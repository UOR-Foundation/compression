// Thin loader for the uorc WebAssembly client (crates/uorc-wasm).
const ERRORS = ["", "no checked archive was found within the step budget",
  "a natural-number operation overflowed 64 bits", "input is not a decodable self-contained UORC archive"];

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
  return {
    compress: (bytes, steps = 10_000_000n) => call(bytes, () => x.compress(BigInt(steps))),
    decompress: (archive) => call(archive, () => x.decompress()),
  };
}
