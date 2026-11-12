# The kappa-named model states of the standard basis (writing aid). A state is a basis root: bytes the ncm coder
# codes first (ncm_model decodeWarm), so a frame is coded as their continuation. A state's bytes are fixed
# forever once an archive may name it, so each is built from a named git tree and checked against its kappa.
#
# State 1 (256 KiB) consolidates this repository's own text, code and configuration at tree 99c4290 (the v0.4.0
# release): 4 KiB of Markdown, 4 KiB of Rust/Python/JavaScript and 2 KiB of JSON/TOML/HTML/YAML in turn, each
# drawn round-robin over the files of its kind (sorted by path), the next 4 KiB of each file per round. Generated
# outputs, vendored files and frozen basis fixtures are left out.
#
#   python3 client/gen/gen_state.py           writes crates/uorc/src/state-*.bin
#   python3 client/gen/gen_state.py --check   checks it against its kappa
import hashlib
import os
import subprocess
import sys
HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, '..', '..')
TREE = '99c4290'


def git(*args):
    return subprocess.check_output(['git', '-C', ROOT] + list(args))


def state1():
    names = [f for f in git('ls-tree', '-r', '--name-only', TREE).decode().split('\n') if f]
    names = [f for f in names if not f.startswith(('generated/', 'vendor/', 'client/gen/v0', 'crates/uorc/tests/compat'))]
    blob = {}

    def take(exts, budget, per):
        sel = []
        for f in sorted(n for n in names if n.endswith(exts)):
            d = blob.setdefault(f, git('show', f'{TREE}:{f}'))
            if len(d) > 500:
                sel.append(f)
        out, i = b'', 0
        while len(out) < budget and sel:
            f = sel[i % len(sel)]
            d = blob[f]
            k = (i // len(sel)) * per
            if k >= len(d):
                sel.remove(f)
                continue
            out += d[k:k + per]
            i += 1
        return out[:budget]
    size = 256 * 1024
    t = take(('.md',), size, 4096)
    c = take(('.rs', '.py', '.mjs'), size, 4096)
    m = take(('.json', '.toml', '.html', '.yml'), size, 4096)
    mix = b''
    for i in range(0, size, 4096):
        mix += t[i:i + 4096] + c[i:i + 4096] + m[i:i + 2048]
    return mix[:size]


def state2():
    # The last 64 KiB of state 1: about 93% of its gain on small text at a quarter of the warm-up (the model
    # codes a state before every frame that continues it, natively once per process).
    return state1()[-64 * 1024:]


STATES = {1: (state1, 'state-1.bin'), 2: (state2, 'state-2.bin')}

if __name__ == '__main__':
    for k, (build, name) in STATES.items():
        data = build()
        path = os.path.join(ROOT, 'crates', 'uorc', 'src', name)
        if '--check' in sys.argv:
            assert open(path, 'rb').read() == data, name
        else:
            open(path, 'wb').write(data)
        print(k, name, len(data), 'kappa', hashlib.sha256(data).hexdigest())
