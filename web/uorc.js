// Thin loader for the uorc WebAssembly client (crates/uorc-wasm).
const ERRORS = ["", "no checked archive was found within the step budget",
  "a natural-number operation overflowed 64 bits", "input is not a decodable self-contained UORC archive",
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
  return {
    compress: (bytes, steps = 0n) => call(bytes, () => x.compress(BigInt(steps))),
    decompress: (archive) => call(archive, () => x.decompress()),
    // Frame header and integrity seal of one archive, without decoding.
    inspect: (archive) => json(call(archive, () => x.inspect())),
    // Decode in memory and check; with an original, also compare. Returns one
    // description per archive in the stream; throws on any failure.
    verify: (archive, original) => json(call(archive, () => {
      if (original) put(x.original_buffer, original);
      return x.verify(original ? 1 : 0);
    })),
    // entries: [{ path: "dir/a.txt", kind: "file" | "executable" | "directory", data: Uint8Array }]
    compressFiles: (entries, steps = 0n) => call(packEntries(entries), () => x.compress_tree(BigInt(steps))),
    decompressFiles: (archive) => unpackEntries(call(archive, () => x.decompress_tree())),
  };
}
