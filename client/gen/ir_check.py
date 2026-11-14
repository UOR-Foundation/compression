# The ncm coder's IR model (ncm_model.py, and ncm_streams.py where it exists) against the native
# coder (crates/uorc/src/ncm.rs, ncm/streams.rs): the same input coded both ways must give the same
# bytes, and the IR decoder must give back the input from the native payload.
#
#   python3 client/gen/ir_check.py [--k 1,2,3,4] [--n 400] [--file F] [--h H] [--epoch E]
#                                  [--jobs 4] [--no-serial] [--keep DIR]
#
# Input: bytes [h, h+n) of F, coded after the history F[0:h], under the FULL profile.
# Cases (each its own process, --jobs at a time; the IR Machine is slow, ~3 min per 400 bytes):
#   serial-encode    ncm_model.py py_encode      against ncm::encode_profile
#   serial-decode    ncm_model.py py_decode      of the native payload, against the input
#   serial-learn     ncm_model.py py_learn       against ncm::encode_learn (payload and profile)
#   serial-*-p16     the same at 16-bit probabilities (frame word probability field 16, ncm::encode_bits_below)
#   streams-encode-k ncm_streams.py py_encode_streams, against ncm::streams::encode (k streams, epoch E)
#   streams-decode-k ncm_streams.py py_decode_streams of the native payload, against the input
# The streams cases run only when client/gen/ncm_streams.py and the native ncm::streams API exist.
# The native side is a throwaway cargo package written under target/ir-check (its own workspace,
# the workspace's Cargo.lock, the workspace's target directory), so no tracked crate carries it.
# Exit status 1 on any difference. A developer check: no gate runs it.
import argparse
import multiprocessing
import os
import shutil
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, '..', '..'))
FULL = (1 << 20) - 1

MANIFEST = '''[package]
name = "uorc-ir-dump"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
uorc = {{ path = "{uorc}" }}

[workspace]
'''

# The native payloads of one input, written to OUT: encode.bin, learn.bin and learn.mask (the
# learned profile, decimal), and streams-<k>.bin for each k. Every payload is also decoded natively.
MAIN_SERIAL = '''//! The native ncm payloads client/gen/ir_check.py compares with the IR model's.
use uorc::ncm;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let file = std::fs::read(&a[1]).expect("read the input file");
    let h: usize = a[2].parse().expect("h");
    let n: usize = a[3].parse().expect("n");
    let out = std::path::Path::new(&a[4]);
    let ks: Vec<usize> = a[5].split(',').filter(|s| !s.is_empty()).map(|s| s.parse().expect("k")).collect();
    let epoch: usize = a[6].parse().expect("epoch");
    let (history, data) = (&file[..h], &file[h..h + n]);
    let p = ncm::encode_profile(history, data, ncm::FULL).expect("native encode");
    let back = ncm::decode_profile(history, &p, u64::MAX, ncm::FULL).expect("native decode");
    assert_eq!(back, data, "native serial round trip");
    std::fs::write(out.join("encode.bin"), &p).unwrap();
    let w = ncm::encode_bits_below(history, data, ncm::FULL, 16, usize::MAX).expect("native 16-bit encode");
    let back = ncm::decode_canonical_window_bits(&[history], &w, n as u64, ncm::FULL, 16).expect("native 16-bit decode");
    assert_eq!(back, data, "native 16-bit round trip");
    std::fs::write(out.join("encode16.bin"), &w).unwrap();
    let (l, mask) = ncm::encode_learn(history, data, ncm::FULL).expect("native learn");
    std::fs::write(out.join("learn.bin"), &l).unwrap();
    std::fs::write(out.join("learn.mask"), mask.to_string()).unwrap();
    streams(history, data, &ks, epoch, out);
}
'''

MAIN_NO_STREAMS = '''
fn streams(_: &[u8], _: &[u8], ks: &[usize], _: usize, _: &std::path::Path) {
    assert!(ks.is_empty(), "no native streams API");
}
'''

MAIN_STREAMS = '''
fn streams(history: &[u8], data: &[u8], ks: &[usize], epoch: usize, out: &std::path::Path) {
    use uorc::ncm::streams;
    for &k in ks {
        let plan = streams::Plan::of(k, epoch);
        let p = streams::encode(history, data, ncm::FULL, plan, 1).expect("native streams encode");
        let back = streams::decode(&[history], &p, u64::MAX, ncm::FULL, plan, 1, true)
            .expect("native streams decode");
        assert_eq!(back, data, "native streams round trip");
        std::fs::write(out.join(format!("streams-{k}.bin")), &p).unwrap();
    }
}
'''


def streams_available():
    """Why the streams cases cannot run, or None when they can."""
    py = os.path.join(HERE, 'ncm_streams.py')
    rs = os.path.join(ROOT, 'crates', 'uorc', 'src', 'ncm', 'streams.rs')
    if not os.path.exists(py) or 'def py_encode_streams' not in open(py).read():
        return 'client/gen/ncm_streams.py (py_encode_streams) is absent'
    if not os.path.exists(rs) or 'pub mod streams' not in open(os.path.join(ROOT, 'crates', 'uorc', 'src', 'ncm.rs')).read():
        return 'the native ncm::streams API (crates/uorc/src/ncm/streams.rs) is absent'
    text = open(rs).read()
    if 'pub struct Plan' not in text or 'pub fn encode(' not in text or 'pub fn decode(' not in text:
        return 'crates/uorc/src/ncm/streams.rs has no Plan/encode/decode'
    return None


def native(args, ks, work):
    crate = os.path.join(ROOT, 'target', 'ir-check', 'dump')
    os.makedirs(os.path.join(crate, 'src'), exist_ok=True)
    files = {
        'Cargo.toml': MANIFEST.format(uorc=os.path.join(ROOT, 'crates', 'uorc')),
        os.path.join('src', 'main.rs'): MAIN_SERIAL + (MAIN_STREAMS if ks else MAIN_NO_STREAMS),
    }
    for name, text in files.items():
        p = os.path.join(crate, name)
        if not os.path.exists(p) or open(p).read() != text:   # unchanged files keep cargo's cache
            open(p, 'w').write(text)
    lock = os.path.join(crate, 'Cargo.lock')
    if not os.path.exists(lock):
        shutil.copy(os.path.join(ROOT, 'Cargo.lock'), lock)   # the workspace's resolved versions
    t = time.time()
    build = ['cargo', 'build', '-q', '--manifest-path', os.path.join(crate, 'Cargo.toml'),
             '--target-dir', os.path.join(ROOT, 'target')]
    if args.release:
        build.append('--release')
    print(f'ir-check: building the native dumper ({"release" if args.release else "dev"})', flush=True)
    subprocess.run(build, check=True)
    exe = os.path.join(ROOT, 'target', 'release' if args.release else 'debug', 'uorc-ir-dump')
    subprocess.run([exe, args.file, str(args.h), str(args.n), work, ','.join(map(str, ks)), str(args.epoch)],
                   check=True)
    print(f'ir-check: native payloads in {time.time() - t:.1f}s', flush=True)


def run_case(job):
    """One case in its own process: (name, native bytes, IR bytes, verdict, seconds)."""
    name, kind, k, a = job
    t = time.time()
    sys.path.insert(0, HERE)
    import ncm_model as nm
    raw = open(a['file'], 'rb').read()
    history, data = raw[:a['h']], raw[a['h']:a['h'] + a['n']]
    work = a['work']

    def nat(f):
        return open(os.path.join(work, f), 'rb').read()

    note = ''
    if kind == 'serial-encode':
        want, got = nat('encode.bin'), nm.py_encode(data, history, FULL)
    elif kind == 'serial-decode':
        want, got = data, nm.py_decode(nat('encode.bin'), history, FULL)
    elif kind == 'serial-encode-p16':
        want, got = nat('encode16.bin'), nm.py_encode(data, history, FULL | 16 << 32)
    elif kind == 'serial-decode-p16':
        want, got = data, nm.py_decode(nat('encode16.bin'), history, FULL | 16 << 32)
    elif kind == 'serial-learn':
        want, mask = nat('learn.bin'), int(nat('learn.mask'))
        got, gmask = nm.py_learn(data, history, FULL)
        note = f'profile native {mask:#x} IR {gmask:#x}'
        if mask != gmask:
            return name, len(want), len(got), 'DIFFERENT learned profile; ' + note, time.time() - t
    else:
        import ncm_streams as ns
        if kind == 'streams-encode':
            want, got = nat(f'streams-{k}.bin'), ns.py_encode_streams(data, history, FULL, k, a['epoch'])
        else:
            want, got = data, ns.py_decode_streams(nat(f'streams-{k}.bin'), history, FULL, k, a['epoch'])
    if want == got:
        verdict = 'identical'
    else:
        off = next((i for i, (x, y) in enumerate(zip(want, got)) if x != y), min(len(want), len(got)))
        verdict = f'DIFFERENT at offset {off}'
    if note:
        verdict += '; ' + note
    return name, len(want), len(got), verdict, time.time() - t


def main():
    ap = argparse.ArgumentParser(description='ncm IR model (client/gen) against the native coder, byte for byte')
    ap.add_argument('--k', default='1,2,3,4', help='stream counts for the streams cases ("" for none)')
    ap.add_argument('--n', type=int, default=400, help='bytes coded')
    ap.add_argument('--h', type=int, default=0, help='history bytes before them')
    ap.add_argument('--file', default=os.path.join(ROOT, 'LICENSE-APACHE'))
    ap.add_argument('--epoch', type=int, default=64, help='streams epoch (a power of two)')
    ap.add_argument('--jobs', type=int, default=4, help='cases run at once')
    ap.add_argument('--no-serial', action='store_true', help='skip the serial cases')
    ap.add_argument('--release', action='store_true', help='build the native dumper in release')
    ap.add_argument('--keep', help='write the native payloads here (default: a directory under target/ir-check)')
    args = ap.parse_args()
    args.file = os.path.abspath(args.file)
    size = os.path.getsize(args.file)
    if args.h + args.n > size:
        sys.exit(f'ir-check: {args.file} has {size} bytes, fewer than h + n = {args.h + args.n}')
    if args.epoch < 1 or args.epoch & (args.epoch - 1):
        sys.exit('ir-check: --epoch must be a power of two')
    ks = [int(s) for s in args.k.split(',') if s.strip()]
    if ks:
        why = streams_available()
        if why:
            print(f'ir-check: SKIP streams cases (k={args.k}): {why}', flush=True)
            ks = []
    work = os.path.abspath(args.keep or os.path.join(ROOT, 'target', 'ir-check', 'out'))
    os.makedirs(work, exist_ok=True)
    native(args, ks, work)

    a = {'file': args.file, 'h': args.h, 'n': args.n, 'epoch': args.epoch, 'work': work}
    jobs = []
    if not args.no_serial:
        jobs += [(c, c, 0, a) for c in ('serial-encode', 'serial-decode', 'serial-learn', 'serial-encode-p16', 'serial-decode-p16')]
    for k in ks:
        jobs += [(f'streams-encode-{k}', 'streams-encode', k, a), (f'streams-decode-{k}', 'streams-decode', k, a)]
    if not jobs:
        sys.exit('ir-check: no cases to run')
    print(f'ir-check: {len(jobs)} cases, {args.jobs} at a time; input {os.path.relpath(args.file, ROOT)} '
          f'[{args.h}, {args.h + args.n}), epoch {args.epoch}', flush=True)
    t = time.time()
    rows = []
    with multiprocessing.get_context('fork').Pool(min(args.jobs, len(jobs))) as pool:
        for r in pool.imap_unordered(run_case, jobs):
            print(f'  done {r[0]} ({r[4]:.0f}s): {r[3]}', flush=True)
            rows.append(r)
    rows.sort(key=lambda r: [j[0] for j in jobs].index(r[0]))
    w = max(len(r[0]) for r in rows)
    print(f'\n{"case":<{w}}  {"native":>7}  {"IR":>7}  result')
    for name, nb, ib, verdict, _ in rows:
        print(f'{name:<{w}}  {nb:>7}  {ib:>7}  {verdict}')
    bad = sum(not r[3].startswith('identical') for r in rows)
    print(f'\nir-check: {len(rows) - bad}/{len(rows)} identical in {time.time() - t:.0f}s')
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
