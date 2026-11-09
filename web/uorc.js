// Thin loader for the uorc WebAssembly client (crates/uorc-wasm).
const ERRORS = ["", "no checked archive was found within the step budget",
  "a natural-number operation overflowed 64 bits", "input is not a decodable stream of self-contained UORC archives"];

export async function load(url) {
  const response = await fetch(url);
  const { instance } = await WebAssembly.instantiateStreaming(response, {});
  const x = instance.exports;
  const call = (bytes, op) => {
    const at = x.input_buffer(bytes.length);
    new Uint8Array(x.memory.buffer, at, bytes.length).set(bytes);
    const code = op();
    if (code !== 0) throw new Error(ERRORS[code] ?? `uorc error ${code}`);
    return new Uint8Array(x.memory.buffer, x.output_ptr(), x.output_len()).slice();
  };
  const frame = Number(x.frame_size());
  const header = Number(x.header_size());
  const bytesOf = async (blob) => new Uint8Array(await blob.arrayBuffer());
  return {
    compress: (bytes, steps = 0n, best = false) => call(bytes, () => x.compress(BigInt(steps), best ? 1 : 0)),
    decompress: (archive) => call(archive, () => x.decompress()),
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
        const head = await bytesOf(blob.slice(at, at + header));
        new Uint8Array(x.memory.buffer, x.input_buffer(head.length), head.length).set(head);
        const length = x.frame_length();
        if (length < 0 || at + length > blob.size) throw new Error(ERRORS[3]);
        parts.push(call(await bytesOf(blob.slice(at, at + length)), () => x.decompress_frame()));
        at += length;
        progress(at, blob.size);
      }
      return new Blob(parts);
    },
  };
}
