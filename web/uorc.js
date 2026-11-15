// Thin loader for the uorc WebAssembly client (crates/uorc-wasm). Loaded as a
// module worker, it is also the page's worker: see `serve` at the end.
const ERRORS = ["", "no checked archive was found within the step budget",
  "a natural-number operation overflowed 64 bits", "input is not a decodable stream of self-contained UORC archives",
  "the archive does not decode to the given original"];
// The code of each failure, as the page's product model names them.
export const CODES = ["", "budget", "overflow", "decode", "mismatch"];
const failure = (code) => Object.assign(new Error(ERRORS[code] ?? `uorc error ${code}`), { code: CODES[code] ?? "decode" });
const KINDS = ["file", "executable", "directory"];
const fail = () => { throw failure(3); };

// Entries cross into wasm as u32 count, then per entry u8 kind, u32 path length,
// path, u32 data length, data (little-endian).
function packEntries(entries) {
  const enc = new TextEncoder();
  const paths = entries.map((e) => enc.encode(e.path));
  const size = 4 + entries.reduce((n, e, i) => n + 9 + paths[i].length + (e.data?.length ?? 0), 0);
  const out = new Uint8Array(size);
  const view = new DataView(out.buffer);
  let at = 0;
  view.setUint32(at, entries.length, true); at += 4;
  entries.forEach((e, i) => {
    const data = e.data ?? new Uint8Array(0);
    out[at++] = Math.max(0, KINDS.indexOf(e.kind ?? "file"));
    view.setUint32(at, paths[i].length, true); at += 4; out.set(paths[i], at); at += paths[i].length;
    view.setUint32(at, data.length, true); at += 4; out.set(data, at); at += data.length;
  });
  return out;
}

// A tree's listing crosses as u32 count, then per entry u8 kind, u32 path
// length, path, u64 length (little-endian): no file bytes.
function packListing(items) {
  const enc = new TextEncoder();
  const paths = items.map((e) => enc.encode(e.path));
  const out = new Uint8Array(4 + items.reduce((n, e, i) => n + 13 + paths[i].length, 0));
  const view = new DataView(out.buffer);
  let at = 0;
  view.setUint32(at, items.length, true); at += 4;
  items.forEach((e, i) => {
    out[at++] = Math.max(0, KINDS.indexOf(e.kind ?? "file"));
    view.setUint32(at, paths[i].length, true); at += 4; out.set(paths[i], at); at += paths[i].length;
    view.setBigUint64(at, BigInt(e.blob?.size ?? 0), true); at += 8;
  });
  return out;
}

function unpackEntries(bytes) {
  const dec = new TextDecoder();
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let at = 0;
  const count = view.getUint32(at, true); at += 4;
  const entries = [];
  for (let i = 0; i < count; i++) {
    const kind = KINDS[bytes[at++]];
    const pathLength = view.getUint32(at, true); at += 4;
    const path = dec.decode(bytes.subarray(at, at + pathLength)); at += pathLength;
    const dataLength = view.getUint32(at, true); at += 4;
    entries.push({ path, kind, data: bytes.slice(at, at + dataLength) }); at += dataLength;
  }
  return entries;
}

// A coding request for one planned frame (see code_frame in uorc-wasm):
// u32 pieceCount, u64 piece*, u32 window, u64 windowLength*, then the window
// frames and the frame.
function frameRequest(pieces, window, frame) {
  const size = 8 + 8 * pieces.length + 8 * window.length + window.reduce((n, w) => n + w.length, 0) + frame.length;
  const out = new Uint8Array(size);
  const view = new DataView(out.buffer);
  let at = 0;
  view.setUint32(at, pieces.length, true); at += 4;
  for (const p of pieces) { view.setBigUint64(at, p, true); at += 8; }
  view.setUint32(at, window.length, true); at += 4;
  for (const w of window) { view.setBigUint64(at, BigInt(w.length), true); at += 8; }
  for (const w of [...window, frame]) { out.set(w, at); at += w.length; }
  return out;
}

// A frame's coding runs a few passes (the scheduled strategies, a learned
// re-encode, and in small schedules the v0.3 entry), how many is known only
// as they run. After x passes (whole and part) the share done is
// 1 - (passes / (passes + 1))^x: under 1/passes per pass, then smaller
// steps, so the bar keeps moving through any extra pass and never moves back.
export function frameShare(passes = 3) {
  let begun = 0, last = 0, best = 0;
  const left = passes / (passes + 1);
  return (done, total) => {
    if (done < last) begun++;
    last = done;
    best = Math.max(best, Math.min(0.99, 1 - left ** (begun + done / Math.max(total, 1))));
    return best;
  };
}

// `source` is the module's URL, or a WebAssembly.Module already compiled (a
// pool worker receives the page worker's).
export async function load(source) {
  // The module reports each coding pass through uorc.progress (bytes done of
  // the pass's length); `onPass` routes it to the operation running.
  let onPass = () => {};
  const imports = { uorc: { progress: (done, total) => onPass(done, total) } };
  const { module, instance } = source instanceof WebAssembly.Module
    ? { module: source, instance: await WebAssembly.instantiate(source, imports) }
    : await WebAssembly.instantiateStreaming(fetch(source), imports);
  const x = instance.exports;
  x.report_progress?.(1);
  const put = (alloc, bytes) => {
    const at = alloc(bytes.length); // may grow memory, so read the buffer after
    new Uint8Array(x.memory.buffer, at, bytes.length).set(bytes);
  };
  const call = (bytes, op) => {
    put(x.input_buffer, bytes);
    const code = op();
    if (code !== 0) throw failure(code);
    return new Uint8Array(x.memory.buffer, x.output_ptr(), x.output_len()).slice();
  };
  const json = (bytes) => JSON.parse(new TextDecoder().decode(bytes));
  const frame = Number(x.frame_size());
  const bytesOf = async (blob) => new Uint8Array(await blob.arrayBuffer());
  // Length of the archive starting at `at` in `blob`, read from its header.
  const archiveLength = async (blob, at) => {
    for (let want = Number(x.header_size()); ; want *= 2) {
      const head = await bytesOf(blob.slice(at, at + want));
      put(x.input_buffer, head);
      const length = x.frame_length();
      if (length >= 0 && at + length <= blob.size) return length;
      if (length === -1 || head.length < want) throw failure(3);
    }
  };
  // Start decoding a stream: note every archive first (a reference frame
  // names a frame to keep, and each archive how many frames before it it
  // reads), so the decoder keeps exactly the frames later ones read.
  const begin = async (blob) => {
    x.window_reset();
    for (let at = 0; at < blob.size;) {
      const length = await archiveLength(blob, at);
      put(x.input_buffer, await bytesOf(blob.slice(at, at + length)));
      x.note_frame();
      at += length;
    }
  };
  // Code the stream the encoder has been started on (window_reset, or
  // tree_start for a tree): here, a frame at a time, or with a pool of
  // workers (see `workers`) on several cores, a span to a worker, spans in
  // order. Either way the bytes are the
  // library's.
  const codeStream = async (blob, steps, progress, pool) => {
    const total = blob.size;
    // Until two pool workers have loaded, the page's worker codes alone, so
    // the first file does not wait behind their start: the same plan and
    // cuts, each trial coded here, one after another.
    const coder = pool && pool.ready >= 2 ? pool : {
      async code(job, onShare) {
        const parts = await Promise.all([...job.window, job.frame].map(bytesOf));
        const share = frameShare();
        onPass = (done, of) => onShare(share(done, of));
        try {
          return call(frameRequest(job.pieces, parts.slice(0, -1), parts.at(-1)), () => x.code_frame(job.which, BigInt(job.steps)));
        } finally {
          onPass = () => {};
        }
      },
    };
    // Plan here, code each span's trials on the pool.
    const archives = [];
    const shares = new Map();
    let finished = 0;
    const report = () => progress(Math.min(total, finished + [...shares.values()].reduce((n, v) => n + v, 0)), total);
    const settle = (index, length, promise) => {
      archives[index] = promise.then((archive) => { shares.delete(index); finished += length; report(); return archive; });
    };
    // A span is cut as the library cuts it (uorc::compress_span): a part
    // that repeats a frame before it in its span is that frame's reference,
    // a second half that repeats its first is the first's reference (the
    // first one frame); otherwise it is one frame, unless its two halves
    // (the second reading the first while the declared host holds them) are
    // shorter by their planning trials (code_frame with which = strategies)
    // than the part's, each half cut the same way. Then each frame the cut
    // keeps is coded once (code_frame with no strategy), the frames side by
    // side on the pool (and the span whole beside them), so the bytes are
    // the library's. Both halves are cut
    // at once, the second after the first whole, and cut again after the
    // first's frames when the first is cut further.
    const planning = x.strategies();
    const slice = (r) => blob.slice(r.start, r.start + r.length);
    const reads = (n, history) => x.span_reads(BigInt(n), BigInt(history)) === 1;
    const label = async (r) => call(await bytesOf(slice(r)), () => x.frame_label());
    const labels = (frames) => Promise.all(frames.map(label));
    // The reference archive of a part that repeats one of `earlier`, or null.
    const repeat = async (r, pieces, earlier) => {
      const request = frameRequest(pieces, earlier, await bytesOf(slice(r)));
      const reference = call(request, () => x.repeat_frame());
      return reference.length ? reference : null;
    };
    const splitPieces = (pieces, at) => {
      const first = [], rest = [];
      let used = 0n, i = 0;
      for (; i < pieces.length; i++) {
        const take = pieces[i] < at - used ? pieces[i] : at - used;
        first.push(take);
        used += take;
        if (used === at) { rest.push(pieces[i] - take); i++; break; }
      }
      return [first, [...rest, ...pieces.slice(i)]];
    };
    // Progress: each part of a cut owns a share of its span's bar (a meter),
    // split over its trials and its halves' cuts by their bytes. A meter
    // moves with its trials' passes, is credited whole when its part ends,
    // and a part cut again maps its new passes onto what its meter has left,
    // so the bar never moves back and never waits on work it did not count.
    const meter = (weight, add) => {
      let base = 0, got = 0;
      const m = {
        move(f) { const v = base + (weight - base) * Math.min(1, f); if (v > got) { add(v - got); got = v; } },
        again() { base = got; },
        end() { m.move(1); },
        // Meters for parts of `ranges`, sharing this one's weight by bytes.
        split(ranges) {
          const bytes = ranges.reduce((n, r) => n + r.length, 0);
          return ranges.map((r) => meter((weight * r.length) / bytes, (d) => { got += d; add(d); }));
        },
      };
      return m;
    };
    const cut = async (range, pieces, window, earlier, whole, m) => {
      const parts = await cutParts(range, pieces, window, earlier, whole, m);
      m.end();
      return parts;
    };
    // A frame of the cut: its range, pieces and window, its archive when
    // the cut wrote it (a reference), and the length the cut counts for it
    // (that archive's, else its planning trial's).
    const leaf = (range, pieces, window, archive = null, len = 0) => ({ ...range, pieces, window, archive, len: archive ? archive.length : len });
    const cutParts = async (range, pieces, window, earlier, whole, m) => {
      const planned = async (r, p, w, t) => (await coder.code({ which: planning, steps, pieces: p, window: w.map(slice), frame: slice(r) }, (share) => t.move(share))).length;
      if (!whole) {
        const reference = await repeat(range, pieces, earlier);
        if (reference) return [leaf(range, pieces, window, reference)];
      }
      const half = range.length >>> 1;
      if (half === 0) return [leaf(range, pieces, window, null, whole ?? await planned(range, pieces, window, m))];
      const a = { start: range.start, length: half };
      const b = { start: range.start + half, length: range.length - half };
      const [pa, pb] = splitPieces(pieces, BigInt(half));
      const held = window.reduce((n, w) => n + w.length, 0);
      const fits = reads(b.length, held + half);
      const after = (frames) => (fits ? [...window, ...frames] : []);
      const overA = [...earlier, await label(a)];
      // a second half that repeats a frame is its reference, the first half
      // cut by the same rule (the second cut after the first's frames when
      // it repeats the first and the first is cut); the part stays one frame
      // when its planning trial is not longer than that cut
      const known = await repeat(b, pb, earlier);
      const repeated = known ? null : await repeat(b, pb, overA.slice(earlier.length));
      if (known || repeated) {
        const [ra, rb, rf, rw] = m.split([a, b, a, range]);
        const cost = (leaves) => leaves.reduce((n, l) => n + l.len, 0);
        const reference = known ?? repeated;
        let parts = await cut(a, pa, window, earlier, null, ra);
        if (repeated && parts.length > 1) {
          parts = [...parts, ...await cut(b, pb, after(parts), [...earlier, ...await labels(parts)], null, rb)];
          const first = await planned(a, pa, window, rf);
          if (first + reference.length <= cost(parts)) parts = [leaf(a, pa, window, null, first), leaf(b, pb, after([a]), reference)];
        } else {
          parts = [...parts, leaf(b, pb, after([a]), reference)];
        }
        const own = whole ?? await planned(range, pieces, window, rw);
        return own <= cost(parts) ? [leaf(range, pieces, window, null, own)] : parts;
      }
      // the planning trials to run (the whole part unless the caller ran
      // it, then each half), then each half's cut
      const trials = whole ? [a, b] : [range, a, b];
      const meters = m.split([...trials, a, b]);
      const coded = await Promise.all(trials.map((r, i) => planned(r, r === a ? pa : r === b ? pb : pieces, r === b ? after([a]) : window, meters[i])));
      const [w, ca, cb] = whole ? [whole, ...coded] : coded;
      if (ca + cb >= w) return [leaf(range, pieces, window, null, w)];
      const [ma, mb] = meters.slice(trials.length);
      const [firsts, overWhole] = await Promise.all([cut(a, pa, window, earlier, ca, ma), cutParts(b, pb, after([a]), overA, cb, mb)]);
      if (firsts.length === 1) return [...firsts, ...overWhole];
      mb.again();
      return [...firsts, ...await cut(b, pb, after(firsts), [...earlier, ...await labels(firsts)], null, mb)];
    };
    // A span's bar is shared by its cut and its frames' coding, by the
    // bytes each reads (the span's, once each).
    const span = (f) => {
      const range = { start: f.start, length: f.length };
      const [planMeter, codeMeter] = meter(f.length, (d) => { shares.set(f.index, (shares.get(f.index) ?? 0) + d); report(); }).split([range, range]);
      // a span cut by planning trials alone (no reference among its frames)
      // is coded whole beside them, the frames kept only when strictly
      // shorter, as the library keeps them
      const frames = cut(range, f.pieces, [], [], null, planMeter).then(async (leaves) => {
        const coding = leaves.length > 1 && leaves.every((l) => !l.archive) ? [...leaves, leaf(range, f.pieces, [])] : leaves;
        const meters = codeMeter.split(coding);
        const coded = await Promise.all(coding.map((l, i) => l.archive ?? coder.code({ which: -1, steps, pieces: l.pieces, window: l.window.map(slice), frame: slice(l) }, (share) => meters[i].move(share))));
        if (coding === leaves) return coded;
        const whole = coded.pop();
        return whole.length <= coded.reduce((n, a) => n + a.length, 0) ? [whole] : coded;
      });
      settle(f.index, f.length, frames.then((archives) => new Blob(archives)));
    };
    let index = 0;
    let position = 0;
    const plan = (records) => {
      const view = new DataView(records.buffer, records.byteOffset, records.byteLength);
      let p = 0;
      const n = view.getUint32(p, true); p += 4;
      for (let i = 0; i < n; i++) {
        const kind = records[p++];
        const length = Number(view.getBigUint64(p, true)); p += 8;
        const f = { index: index++, start: position, length };
        if (kind === 0) {
          const size = Number(view.getBigUint64(p, true)); p += 8;
          archives[f.index] = Promise.resolve(records.slice(p, p + size)); p += size;
          finished += length;
        } else {
          const count = view.getUint32(p, true); p += 4;
          f.pieces = [];
          for (let k = 0; k < count; k++) { f.pieces.push(view.getBigUint64(p, true)); p += 8; }
          span(f);
        }
        position += length;
      }
    };
    for (let at = 0; at < total; at += frame) {
      plan(call(await bytesOf(blob.slice(at, at + frame)), () => x.plan_frame()));
      report();
    }
    plan(call(new Uint8Array(0), () => x.plan_end()));
    const parts = await Promise.all(archives);
    progress(total, total);
    return new Blob(parts);
  };
  return {
    compress: (bytes, steps = 0n) => call(bytes, () => x.compress(BigInt(steps))),
    decompress: (archive) => call(archive, () => x.decompress()),
    // Frame header and integrity seal of every archive in a stream, without
    // decoding: one description per archive.
    inspect: (archive) => json(call(archive, () => x.inspect())),
    // Stream a File or Blob of any size through frames, a few in wasm memory
    // at a time; the result is a Blob of the concatenated archives, the same
    // bytes the native client writes.
    async compressBlob(blob, steps = 0n, progress = () => {}, pool) {
      x.window_reset();
      return codeStream(blob, steps, progress, pool);
    },
    async decompressBlob(blob, progress = () => {}) {
      const parts = [];
      await begin(blob);
      for (let at = 0; at < blob.size || parts.length === 0;) {
        const length = await archiveLength(blob, at);
        onPass = (done, total) => progress(at + Math.floor((length * done) / Math.max(total, 1)), blob.size);
        parts.push(call(await bytesOf(blob.slice(at, at + length)), () => x.decompress_frame()));
        at += length;
        progress(at, blob.size);
      }
      onPass = () => {};
      return new Blob(parts);
    },
    // Decode in memory and check; with an original, also compare. Returns one
    // description per archive in the stream; throws on any failure.
    // `progress` (done, total) moves through the decode passes, in archive bytes.
    verify: (archive, original, progress = () => {}) => {
      const share = frameShare(1);
      onPass = (done, total) => progress(Math.floor(archive.length * share(done, total)), archive.length);
      try {
        return json(call(archive, () => {
          if (original) put(x.original_buffer, original);
          return x.verify(original ? 1 : 0);
        }));
      } finally {
        onPass = () => {};
      }
    },
    // entries: [{ path: "dir/a.txt", kind: "file" | "executable" | "directory", data: Uint8Array }]
    compressFiles: (entries, steps = 0n) => call(packEntries(entries), () => x.compress_tree(BigInt(steps))),
    decompressFiles: (archive) => unpackEntries(call(archive, () => x.decompress_tree())),
    // Stream a file tree of any size: items are [{ path, kind, blob }] (no
    // blob for a directory). Each file is read a frame at a time; the result
    // is a Blob of the same bytes compressFiles writes.
    async compressTreeBlob(items, steps = 0n, progress = () => {}, pool) {
      x.window_reset();
      const manifest = call(packListing(items), () => x.tree_start());
      return codeStream(new Blob([manifest, ...items.map((e) => e.blob ?? new Blob([]))]), steps, progress, pool);
    },
    // Restore a tree archive a frame at a time: [{ path, kind, blob }], with
    // no blob for a directory. A one-object archive is not a tree.
    async decompressTreeBlob(blob, progress = () => {}) {
      await begin(blob);
      const objects = [];
      for (let at = 0, frames = 0; at < blob.size || frames === 0; frames++) {
        const length = await archiveLength(blob, at);
        const archive = await bytesOf(blob.slice(at, at + length));
        const lengths = json(call(archive, () => x.frame_objects()));
        const output = call(archive, () => x.decompress_frame());
        let from = 0;
        lengths.forEach((n, i) => {
          const piece = output.subarray(from, from + n);
          from += n;
          if (frames > 0 && i === 0) {
            if (!objects.length) throw failure(3);
            objects.at(-1).push(piece);
          } else objects.push([piece]);
        });
        at += length;
        progress(at, blob.size);
      }
      if (objects.length < 2) throw failure(3);
      const manifest = new Uint8Array(await new Blob(objects[0]).arrayBuffer());
      const listed = unpackEntries(call(manifest, () => x.tree_manifest()));
      const files = objects.slice(1);
      const entries = listed.map(({ path, kind }) => (kind === "directory" ? { path, kind } : { path, kind, blob: new Blob(files.shift() ?? fail()) }));
      if (files.length > 1 || (files.length === 1 && (entries.some((e) => e.blob) || files[0].some((p) => p.length)))) throw failure(3);
      return entries;
    },
    // How many objects the stream's first archive holds: one for a file, more
    // for an archive of several files or a folder.
    async objects(blob) {
      const length = await archiveLength(blob, 0);
      const [info] = json(call(await bytesOf(blob.slice(0, length)), () => x.inspect()));
      return info.objects;
    },
    // One planned frame's archive (which -1), or one of its two strategies'
    // (0 or 1; empty when it cannot beat the raw archive), reporting each
    // pass to `pass(done, total)`: what a pool worker runs.
    codeFrame(request, which, steps = 0n, pass = () => {}) {
      onPass = pass;
      try { return call(request, () => x.code_frame(which, BigInt(steps))); } finally { onPass = () => {}; }
    },
    // The compiled module, for other workers to instantiate without a fetch.
    module,
    // Bytes of WebAssembly memory the client holds now.
    memoryBytes: () => x.memory.buffer.byteLength,
  };
}

// The page's worker: one operation at a time on File objects, answering with
// progress, then the result or the failure's code. The page ends a running
// operation by terminating the worker.
const inWorker = typeof WorkerGlobalScope !== "undefined" && self instanceof WorkerGlobalScope;
// A pool worker codes for the page's worker and starts no pool of its own.
const inPool = new URL(import.meta.url).search === "?pool";

const EXAMPLES = {
  text: (n, seed) => { const t = new TextEncoder().encode(seed); return Uint8Array.from({ length: n }, (_, i) => t[i % t.length]); },
  zeros: (n) => new Uint8Array(n),
  counter: (n) => Uint8Array.from({ length: n }, (_, i) => i & 255),
  random: (n) => { const out = new Uint8Array(n); for (let at = 0; at < n; at += 65536) crypto.getRandomValues(out.subarray(at, Math.min(n, at + 65536))); return out; },
  empty: () => new Uint8Array(0),
};

async function serve() {
  let uorc;
  let host;
  try {
    // A pool worker instantiates the module its page worker sends first.
    // The page worker's first message is the page's host rule (Uorc.Page::pageWorker).
    const sent = () => new Promise((resolve) => { self.onmessage = ({ data }) => resolve(data); });
    const first = sent();
    if (inPool) uorc = await load((await first).module);
    else { uorc = await load("./uorc_bg.wasm"); host = (await first).host; }
  } catch (error) {
    self.postMessage({ type: "loadFailed", message: String(error?.message ?? error) });
    return;
  }
  const bytesOf = async (file) => new Uint8Array(await file.arrayBuffer());
  const progress = (done, total) => self.postMessage({ type: "progress", done, total });
  // Requests from the worker that started this one, when it is a pool worker:
  // code one planned frame or strategy (see `pool`).
  const helped = async ({ id, which, steps, pieces, window, frame }) => {
    try {
      const parts = await Promise.all([...window, frame].map(bytesOf));
      const share = frameShare(which < 0 ? 3 : which === 0 ? 2 : 1);
      const archive = uorc.codeFrame(frameRequest(pieces, parts.slice(0, -1), parts.at(-1)), which, steps,
        (done, total) => self.postMessage({ id, type: "share", share: share(done, total) }));
      self.postMessage({ id, type: "coded", archive, memory: uorc.memoryBytes() }, [archive.buffer]);
    } catch (error) {
      self.postMessage({ id, type: "failed", code: error?.code ?? "stopped", message: String(error?.message ?? error) });
      if (!error?.code) self.close();
    }
  };
  // A pool worker codes a little text once, so that its first frame runs on
  // optimized code rather than the engine's first tier.
  if (inPool) uorc.compress(EXAMPLES.text(1 << 16, "uorc pool worker warm-up "));
  const pool = inPool ? { size: 0, memory: 0, ready: 0, warm() {} } : workers(uorc.module, host);
  const run = {
    async compress({ files }) {
      const tree = files.length > 1 || files.some((f) => f.webkitRelativePath);
      if (!tree) {
        const [file] = files;
        const archive = await uorc.compressBlob(file, 0n, progress, pool);
        return { kind: "archive", input: file.size, files: [{ path: `${file.name}.uorc`, blob: archive }] };
      }
      const dirs = new Set();
      const entries = files.map((file) => {
        const path = file.webkitRelativePath || file.name;
        const parts = path.split("/");
        for (let i = 1; i < parts.length; i++) dirs.add(parts.slice(0, i).join("/"));
        return { path, kind: "file", blob: file };
      });
      const total = files.reduce((n, f) => n + f.size, 0);
      const all = [...[...dirs].sort().map((path) => ({ path, kind: "directory" })), ...entries];
      const root = files[0].webkitRelativePath ? files[0].webkitRelativePath.split("/")[0] : "files";
      const archive = await uorc.compressTreeBlob(all, 0n, progress, pool);
      return { kind: "archive", input: total, count: entries.length, files: [{ path: `${root}.uorc`, blob: archive }] };
    },
    async decompress({ archive }) {
      if ((await uorc.objects(archive)) === 1) {
        const blob = await uorc.decompressBlob(archive, progress);
        const name = archive.name.endsWith(".uorc") && archive.name.length > 5 ? archive.name.slice(0, -5) : `${archive.name}.out`;
        return { kind: "files", input: archive.size, files: [{ path: name, blob }] };
      }
      const entries = await uorc.decompressTreeBlob(archive, progress);
      const files = entries.filter((e) => e.blob).map((e) => ({ path: e.path, blob: e.blob }));
      return { kind: "files", input: archive.size, files };
    },
    async inspect({ archive }) {
      return { kind: "table", input: archive.size, rows: uorc.inspect(await bytesOf(archive)) };
    },
    async verify({ archive, original }) {
      const rows = uorc.verify(await bytesOf(archive), original ? await bytesOf(original) : undefined, progress);
      return { kind: "verified", input: archive.size, compared: Boolean(original), rows };
    },
    async example({ input, bytes, seed }) {
      const data = EXAMPLES[input](bytes, seed);
      const started = performance.now();
      const archive = uorc.compress(data);
      const back = uorc.decompress(archive);
      const same = back.length === data.length && back.every((b, i) => b === data[i]);
      return { kind: "example", bytes: data.length, archive: archive.length, same, seconds: (performance.now() - started) / 1000 };
    },
  };
  self.onmessage = async ({ data }) => {
    if (data.id !== undefined) return helped(data);
    try {
      const result = await run[data.op](data);
      self.postMessage({ type: "done", result: { ...result, memory: Math.max(uorc.memoryBytes(), pool.memory), workers: pool.size } });
    } catch (error) {
      // Anything but a uorc failure (a trap, exhausted memory) leaves the
      // module unusable: report the stop and let the page start a new worker.
      self.postMessage({ type: "failed", code: error?.code ?? "stopped", message: String(error?.message ?? error) });
      if (!error?.code) self.close();
    }
  };
  self.postMessage({ type: "ready" });
  pool.warm();
}

// The page worker's pool: one more uorc worker per core it may use, started
// once the page's worker is ready. Each holds its own module, so the count is
// the page's host rule (Uorc.Page::pageWorker) applied to the device the
// browser reports: at most host.coreLimit workers and host.workersPerGiB per
// GiB of device memory (host.unreportedGiB when the browser reports none).
// With fewer than two cores there is no pool.
function workers(module, host) {
  const memory = self.navigator?.deviceMemory;
  const cores = self.navigator?.hardwareConcurrency ?? 1;
  const size = Math.max(0, Math.min(cores, host.coreLimit, Math.floor((memory || host.unreportedGiB) * host.workersPerGiB)));
  const idle = [];
  const waiting = [];
  const jobs = new Map();
  let started = false;
  let next = 0;
  const state = { size: size < 2 ? 0 : size, memory: 0, ready: 0 };
  const start = () => {
    started = true;
    for (let i = 0; i < state.size; i++) {
      const w = new Worker(new URL("uorc.js?pool", import.meta.url), { type: "module" });
      w.postMessage({ module });
      w.onmessage = ({ data }) => {
        if (data.type === "ready") { state.ready++; idle.push(w); pump(); return; }
        if (data.type === "loadFailed") { fail(w, Object.assign(new Error(data.message), { code: undefined })); return; }
        const job = jobs.get(data.id);
        if (!job) return;
        if (data.type === "share") job.onShare(data.share);
        else {
          jobs.delete(data.id);
          if (data.type === "coded") { state.memory = Math.max(state.memory, data.memory); job.resolve(data.archive); }
          else job.reject(Object.assign(new Error(data.message), data.code === "stopped" ? {} : { code: data.code }));
          idle.push(w);
          pump();
        }
      };
      w.onerror = (event) => { event.preventDefault(); fail(w, new Error(event.message || "a pool worker stopped")); };
    }
  };
  const fail = (w, error) => {
    for (const [id, job] of jobs) if (job.worker === w) { jobs.delete(id); job.reject(error); }
    while (waiting.length) waiting.shift().reject(error);
  };
  const pump = () => {
    while (idle.length && waiting.length) {
      const w = idle.shift();
      const job = waiting.shift();
      job.worker = w;
      jobs.set(job.id, job);
      w.postMessage({ id: job.id, ...job.request });
    }
  };
  // Started with the page's worker, so the first operation finds them loaded.
  state.warm = () => { if (!started && state.size) start(); };
  state.code = (request, onShare) => new Promise((resolve, reject) => {
    if (!started) start();
    waiting.push({ id: next++, request, onShare, resolve, reject });
    pump();
  });
  return state;
}

if (inWorker) serve();
