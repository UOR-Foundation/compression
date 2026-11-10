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
  for (const id of ["compress", "decompress", "inspect", "verify"]) $(id).disabled = false;
  status.textContent = "Ready. Choose a file, or run an example below.";

  const bytesOf = async (file) => new Uint8Array(await file.arrayBuffer());
  const report = (text) => { $("report").hidden = !text; $("report").textContent = text ?? ""; };
  const picked = () => {
    const folder = [...$("folder").files];
    return folder.length ? folder : [...$("file").files];
  };
  const timed = async (label, work) => {
    status.textContent = `${label}…`;
    report();
    $("download").replaceChildren();
    await tick();
    const started = performance.now();
    try {
      const said = await work();
      status.textContent = `${said} in ${((performance.now() - started) / 1000).toFixed(2)} s.`;
    } catch (error) {
      status.textContent = `Failed: ${error.message}.`;
    }
  };

  $("compress").addEventListener("click", async () => {
    const files = picked();
    if (!files.length) { status.textContent = "Choose files or a folder first."; return; }
    const tree = files.length > 1 || $("folder").files.length > 0;
    await timed(`Compressing ${files.length} file(s)`, async () => {
      if (!tree) {
        const file = files[0];
        const progress = (done, total) => { status.textContent = `Compressing ${file.name}: ${done} of ${total} bytes…`; };
        const output = await uorc.compressBlob(file, steps(), progress);
        offer(output, `${file.name}.uorc`);
        return `${file.size} → ${output.size} bytes`;
      }
      const entries = [];
      const dirs = new Set();
      for (const file of files) {
        const path = file.webkitRelativePath || file.name;
        const parts = path.split("/");
        for (let i = 1; i < parts.length; i++) dirs.add(parts.slice(0, i).join("/"));
        entries.push({ path, data: await bytesOf(file) });
      }
      const all = [...[...dirs].sort().map((path) => ({ path, kind: "directory" })), ...entries];
      const output = uorc.compressFiles(all, steps());
      const root = files[0].webkitRelativePath ? files[0].webkitRelativePath.split("/")[0] : "files";
      offer(new Blob([output]), `${root}.uorc`);
      return `${entries.length} files, ${entries.reduce((n, e) => n + e.data.length, 0)} → ${output.length} bytes`;
    });
  });

  $("decompress").addEventListener("click", async () => {
    const file = $("file").files[0];
    if (!file) { status.textContent = "Choose an archive first."; return; }
    await timed(`Decompressing ${file.name}`, async () => {
      // A tree's first archive holds several objects; a plain stream's hold one.
      const first = new Uint8Array(await file.slice(0, 64).arrayBuffer());
      const objects = first.length > 8 ? first[8] : 1;
      if (objects === 1) {
        const progress = (done, total) => { status.textContent = `Decompressing ${file.name}: ${done} of ${total} bytes…`; };
        const output = await uorc.decompressBlob(file, progress);
        offer(output, file.name.replace(/\.uorc$/, "") || "output");
        return `${file.size} → ${output.size} bytes`;
      }
      const archive = await bytesOf(file);
      const entries = uorc.decompressFiles(archive);
      const links = entries.filter((e) => e.kind !== "directory").map((e) => {
        const link = document.createElement("a");
        link.href = URL.createObjectURL(new Blob([e.data]));
        link.download = e.path.split("/").pop();
        link.textContent = `${e.path} (${e.data.length} bytes)`;
        const item = document.createElement("li");
        item.append(link);
        return item;
      });
      const list = document.createElement("ul");
      list.append(...links);
      $("download").replaceChildren(list);
      return `${entries.length} entries restored`;
    });
  });

  $("inspect").addEventListener("click", async () => {
    const file = $("file").files[0];
    if (!file) { status.textContent = "Choose an archive first."; return; }
    await timed(`Inspecting ${file.name}`, async () => {
      const infos = uorc.inspect(await bytesOf(file));
      report(JSON.stringify(infos, null, 2));
      const ok = infos.every((info) => info.integrity_ok);
      return `${infos.length} archive(s) read, integrity ${ok ? "ok" : "MISMATCH"}`;
    });
  });

  $("verify").addEventListener("click", async () => {
    const file = $("file").files[0];
    if (!file) { status.textContent = "Choose an archive first."; return; }
    const original = $("original").files[0];
    await timed(`Verifying ${file.name}`, async () => {
      const infos = uorc.verify(await bytesOf(file), original ? await bytesOf(original) : undefined);
      report(JSON.stringify(infos, null, 2));
      return original ? `Verified: identical to ${original.name}` : "Verified: decodes and integrity holds";
    });
  });

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
