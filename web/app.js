import { load } from "./uorc.js";

const $ = (id) => document.getElementById(id);
const status = $("status");
const examples = [
  ["200 zero bytes", () => new Uint8Array(200)],
  ["Repeating text", () => new TextEncoder().encode("abcd".repeat(64))],
  ["Counter bytes 0..255", () => Uint8Array.from({ length: 256 }, (_, i) => i)],
  ["Empty input", () => new Uint8Array(0)],
];

function offer(blob, name) {
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = name;
  link.textContent = `Download ${name} (${blob.size} bytes)`;
  $("download").replaceChildren(link);
}

const steps = () => BigInt(Math.max(0, Math.floor(Number($("steps").value) || 0)));
const tick = () => new Promise((resolve) => setTimeout(resolve, 20));

try {
  const uorc = await load("./uorc.wasm");
  fetch("version.txt").then((r) => r.ok ? r.text() : "").then((v) => { if (v) $("version").textContent = `Release ${v.trim()}`; });
  $("compress").disabled = $("decompress").disabled = false;
  status.textContent = "Ready. Choose a file, or run an example below.";

  for (const id of ["compress", "decompress"]) {
    $(id).addEventListener("click", async () => {
      const file = $("file").files[0];
      if (!file) { status.textContent = "Choose a file first."; return; }
      const verb = id === "compress" ? "Compressing" : "Decompressing";
      status.textContent = `${verb} ${file.name}…`;
      await tick();
      const started = performance.now();
      const progress = (done, total) => { status.textContent = `${verb} ${file.name}: ${done} of ${total} bytes…`; };
      try {
        const output = id === "compress" ? await uorc.compressBlob(file, steps(), progress) : await uorc.decompressBlob(file, progress);
        const seconds = ((performance.now() - started) / 1000).toFixed(2);
        status.textContent = `${file.size} → ${output.size} bytes in ${seconds} s.`;
        offer(output, id === "compress" ? `${file.name}.uorc` : file.name.replace(/\.uorc$/, "") || "output");
      } catch (error) {
        status.textContent = `Failed: ${error.message}.`;
      }
    });
  }

  const body = $("example-results").querySelector("tbody");
  for (const [label, make] of examples) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "secondary";
    button.textContent = label;
    button.addEventListener("click", async () => {
      button.disabled = true;
      status.textContent = `Running “${label}”…`;
      await tick();
      const input = make();
      const started = performance.now();
      const row = document.createElement("tr");
      try {
        const archive = uorc.compress(input, steps());
        const back = uorc.decompress(archive);
        const same = back.length === input.length && back.every((b, i) => b === input[i]);
        const cells = [label, input.length, archive.length, same ? "identical" : "MISMATCH", `${((performance.now() - started) / 1000).toFixed(2)} s`];
        for (const c of cells) { const td = document.createElement("td"); td.textContent = String(c); row.append(td); }
        status.textContent = `“${label}” done.`;
      } catch (error) {
        const td = document.createElement("td"); td.colSpan = 5; td.textContent = `${label}: ${error.message}`; row.append(td);
        status.textContent = `“${label}” failed.`;
      }
      body.prepend(row);
      button.disabled = false;
    });
    $("example-buttons").append(button);
  }
} catch (error) {
  status.textContent = `Could not load the WebAssembly client: ${error.message}`;
}
