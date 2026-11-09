// Thin loader for the uorc WebAssembly client (crates/uorc-wasm).
const ERRORS = ["", "no checked archive was found within the step budget",
  "a natural-number operation overflowed 64 bits", "input is not a decodable stream of self-contained UORC archives",
  "the archive does not decode to the given original"];
const KINDS = ["file", "executable", "directory"];

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
    if (code !== 0) throw new Error(ERRORS[code] ?? `uorc error ${code}`);
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
      if (length === -1 || head.length < want) throw new Error(ERRORS[3]);
    }
  };
  return {
    compress: (bytes, steps = 0n, best = false) => call(bytes, () => x.compress(BigInt(steps), best ? 1 : 0)),
    decompress: (archive) => call(archive, () => x.decompress()),
    // Frame header and integrity seal of every archive in a stream, without
    // decoding: one description per archive.
    inspect: (archive) => json(call(archive, () => x.inspect())),
    // Stream a File or Blob of any size through frames, one in wasm memory at
    // a time; the result is a Blob of the concatenated outputs.
    async compressBlob(blob, steps = 0n, progress = () => {}, best = false) {
      const parts = [];
      for (let at = 0; at < blob.size || at === 0; at += frame) {
        const piece = await bytesOf(blob.slice(at, at + frame));
        parts.push(call(piece, () => x.compress_frame(BigInt(steps), best ? 1 : 0)));
        progress(Math.min(at + frame, blob.size), blob.size);
        if (blob.size === 0) break;
      }
      return new Blob(parts);
    },
    async decompressBlob(blob, progress = () => {}) {
      const parts = [];
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
    compressFiles: (entries, steps = 0n, best = false) => call(packEntries(entries), () => x.compress_tree(BigInt(steps), best ? 1 : 0)),
    decompressFiles: (archive) => unpackEntries(call(archive, () => x.decompress_tree())),
  };
}
