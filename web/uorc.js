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

export async function load(url) {
  const response = await fetch(url);
  const { instance } = await WebAssembly.instantiateStreaming(response, {});
  const x = instance.exports;
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
  // Start decoding a stream: read the headers first and note every reference
  // frame, so the decoder keeps the frames they name (and only those).
  const begin = async (blob) => {
    x.window_reset();
    for (let at = 0; at < blob.size;) {
      const length = await archiveLength(blob, at);
      if (x.may_be_reference()) {
        put(x.input_buffer, await bytesOf(blob.slice(at, at + length)));
        x.note_frame();
      }
      at += length;
    }
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
    async compressBlob(blob, steps = 0n, progress = () => {}) {
      const parts = [];
      x.window_reset();
      for (let at = 0; at < blob.size; at += frame) {
        const piece = await bytesOf(blob.slice(at, at + frame));
        parts.push(call(piece, () => x.compress_frame(BigInt(steps))));
        progress(Math.min(at + frame, blob.size), blob.size);
      }
      parts.push(call(new Uint8Array(0), () => x.compress_end(BigInt(steps))));
      progress(blob.size, blob.size);
      return new Blob(parts);
    },
    async decompressBlob(blob, progress = () => {}) {
      const parts = [];
      await begin(blob);
      for (let at = 0; at < blob.size || parts.length === 0;) {
        const length = await archiveLength(blob, at);
        parts.push(call(await bytesOf(blob.slice(at, at + length)), () => x.decompress_frame()));
        at += length;
        progress(at, blob.size);
      }
      return new Blob(parts);
    },
    // Decode in memory and check; with an original, also compare. Returns one
    // description per archive in the stream; throws on any failure.
    verify: (archive, original) => json(call(archive, () => {
      if (original) put(x.original_buffer, original);
      return x.verify(original ? 1 : 0);
    })),
    // entries: [{ path: "dir/a.txt", kind: "file" | "executable" | "directory", data: Uint8Array }]
    compressFiles: (entries, steps = 0n) => call(packEntries(entries), () => x.compress_tree(BigInt(steps))),
    decompressFiles: (archive) => unpackEntries(call(archive, () => x.decompress_tree())),
    // Stream a file tree of any size: items are [{ path, kind, blob }] (no
    // blob for a directory). Each file is read a frame at a time; the result
    // is a Blob of the same bytes compressFiles writes.
    async compressTreeBlob(items, steps = 0n, progress = () => {}) {
      x.window_reset();
      const manifest = call(packListing(items), () => x.tree_start());
      const parts = [call(manifest, () => x.compress_frame(BigInt(steps)))];
      const total = items.reduce((n, e) => n + (e.blob?.size ?? 0), 0);
      let done = 0;
      for (const { blob } of items) {
        for (let at = 0; blob && at < blob.size; at += frame) {
          const piece = await bytesOf(blob.slice(at, at + frame));
          parts.push(call(piece, () => x.compress_frame(BigInt(steps))));
          done += piece.length;
          progress(done, total);
        }
      }
      parts.push(call(new Uint8Array(0), () => x.compress_end(BigInt(steps))));
      progress(total, total);
      return new Blob(parts);
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
    // Bytes of WebAssembly memory the client holds now.
    memoryBytes: () => x.memory.buffer.byteLength,
  };
}

// The page's worker: one operation at a time on File objects, answering with
// progress, then the result or the failure's code. The page ends a running
// operation by terminating the worker.
const inWorker = typeof WorkerGlobalScope !== "undefined" && self instanceof WorkerGlobalScope;

const EXAMPLES = {
  text: (n, seed) => { const t = new TextEncoder().encode(seed); return Uint8Array.from({ length: n }, (_, i) => t[i % t.length]); },
  zeros: (n) => new Uint8Array(n),
  counter: (n) => Uint8Array.from({ length: n }, (_, i) => i & 255),
  random: (n) => { const out = new Uint8Array(n); for (let at = 0; at < n; at += 65536) crypto.getRandomValues(out.subarray(at, Math.min(n, at + 65536))); return out; },
  empty: () => new Uint8Array(0),
};

async function serve() {
  let uorc;
  try {
    uorc = await load("./uorc_bg.wasm");
  } catch (error) {
    self.postMessage({ type: "loadFailed", message: String(error?.message ?? error) });
    return;
  }
  const bytesOf = async (file) => new Uint8Array(await file.arrayBuffer());
  const progress = (done, total) => self.postMessage({ type: "progress", done, total });
  const run = {
    async compress({ files }) {
      const tree = files.length > 1 || files.some((f) => f.webkitRelativePath);
      if (!tree) {
        const [file] = files;
        const archive = await uorc.compressBlob(file, 0n, progress);
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
      const archive = await uorc.compressTreeBlob(all, 0n, progress);
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
      const rows = uorc.verify(await bytesOf(archive), original ? await bytesOf(original) : undefined);
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
    try {
      const result = await run[data.op](data);
      self.postMessage({ type: "done", result: { ...result, memory: uorc.memoryBytes() } });
    } catch (error) {
      // Anything but a uorc failure (a trap, exhausted memory) leaves the
      // module unusable: report the stop and let the page start a new worker.
      self.postMessage({ type: "failed", code: error?.code ?? "stopped", message: String(error?.message ?? error) });
      if (!error?.code) self.close();
    }
  };
  self.postMessage({ type: "ready" });
}

if (inWorker) serve();
