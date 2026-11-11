// The page's behaviour, read from its Prism product model (src/Uorc/Page.lex.tex),
// which index.html carries as JSON: the state machine, the operations and their
// fields, the controls each state enables, the examples and the error hints.
// The codec runs in a module worker (uorc.js), never on this thread.

const $ = (id) => document.getElementById(id);
const model = JSON.parse($("page-model").textContent);
const one = (kind) => model[kind][0];
const byOrder = (rows) => [...rows].sort((a, b) => a.order - b.order);
const labels = one("Labels");
const states = new Map(model.UmlState.map((s) => [s.state, s]));
const hints = new Map(model.ErrorHint.map((h) => [h.code, h]));
const operations = new Map(model.Operation.map((o) => [o.id, o]));
const fields = new Map(model.Field.map((f) => [f.id, f]));
const controls = new Map(model.Control.map((c) => [c.id, c]));

let state = model.UmlState.find((s) => s.initial).state;
let operation = one("Workbench").initialOperation;
let worker;
let job;

const fmt = (n) => n.toLocaleString("en-US");
const seconds = (ms) => `${(ms / 1000).toFixed(2)} s`;
const status = (text) => { $("status").textContent = text; };

// Take the modeled transition for `trigger` from the current state.
function fire(trigger, message) {
  const t = model.UmlTransition.find((x) => x.source === state && x.trigger === trigger);
  if (!t) throw new Error(`no ${trigger} transition from ${state}`);
  state = t.target;
  document.body.dataset.state = state;
  status(message ?? states.get(state).status);
  render();
}

const chosen = (id) => [...$(`field-${id}`).files];
// An operation can run when each required field, or the field it may replace,
// has a choice.
function complete(op) {
  return operations.get(op).fields.every((id) => {
    const f = fields.get(id);
    return !f.required || chosen(id).length > 0 || (f.replaces && chosen(f.replaces).length > 0);
  });
}

function enabled(id) {
  return controls.get(id).enabledIn.includes(state);
}

function render() {
  const op = operations.get(operation);
  for (const tab of document.querySelectorAll("[role=tab]")) {
    const on = tab.dataset.operation === operation;
    tab.setAttribute("aria-selected", String(on));
    tab.tabIndex = on ? 0 : -1;
    tab.disabled = !enabled("operation");
  }
  $("panel").setAttribute("aria-labelledby", `tab-${operation}`);
  for (const el of document.querySelectorAll(".operation")) el.hidden = el.dataset.operation !== operation;
  for (const el of document.querySelectorAll(".field")) {
    el.hidden = !op.fields.includes(el.dataset.field);
    el.querySelector("input").disabled = !enabled("field");
  }
  $("run").textContent = op.runLabel;
  $("run").disabled = !(enabled("run") && complete(operation));
  $("cancel").disabled = !enabled("cancel");
  for (const b of document.querySelectorAll("[data-example]")) b.disabled = !enabled("example");
  $("progress").hidden = state !== "running";
}

function describe(files) {
  if (!files.length) return "";
  const bytes = files.reduce((n, f) => n + f.size, 0);
  return files.length === 1 ? `${files[0].name}, ${fmt(bytes)} bytes` : `${fmt(files.length)} files, ${fmt(bytes)} bytes`;
}

function edited() {
  for (const f of fields.values()) $(`chosen-${f.id}`).textContent = describe(chosen(f.id));
  if (state === "done" || state === "failed") {
    clearResult();
    fire("edit");
  } else {
    render();
  }
}

function clearResult() {
  $("result").hidden = true;
  $("result").replaceChildren();
  $("error").hidden = true;
}

// The worker: started at load and again after a cancel or a stop.
function start(onReady) {
  try {
    worker = new Worker(new URL("uorc.js", import.meta.url), { type: "module" });
  } catch (error) {
    unavailable(String(error?.message ?? error));
    return;
  }
  worker.onmessage = ({ data }) => {
    if (data.type === "ready") onReady();
    else if (data.type === "loadFailed") unavailable(data.message);
    else if (data.type === "progress") progress(data.done, data.total);
    else if (data.type === "done") finished(data.result);
    else if (data.type === "failed") {
      failed(data.code, data.message);
      if (data.code === "stopped") restart();
    }
  };
  worker.onerror = (event) => {
    event.preventDefault();
    if (state === "loading") unavailable(event.message);
    else if (state === "running") {
      failed("stopped", event.message);
      restart();
    }
  };
}

function restart(after = () => {}) {
  worker?.terminate();
  start(after);
}

function unavailable(message) {
  const hint = hints.get("load");
  fire("loadFailed", `${hint.title}.`);
  showError(hint, message);
}

function showError(hint, detail) {
  $("error-title").textContent = hint.title;
  $("error-explanation").textContent = detail ? `${hint.explanation} (${detail})` : hint.explanation;
  $("error-recovery").textContent = hint.recovery;
  $("error").hidden = false;
}

function progress(done, total) {
  if (!job) return;
  const p = $("progress");
  p.max = Math.max(total, 1);
  p.value = done;
  const elapsed = performance.now() - job.started;
  const rest = done > 0 && done < total ? `, about ${Math.ceil((elapsed / done) * (total - done) / 1000)} s left` : "";
  status(`${job.label}: ${fmt(done)} of ${fmt(total)} bytes${rest}…`);
}

function run(request, label, onDone) {
  clearResult();
  job = { label, started: performance.now(), onDone, request };
  fire("run", `${label}…`);
  worker.postMessage(request);
}

function finished(result) {
  const { onDone, started } = job;
  job = undefined;
  const took = performance.now() - started;
  fire("finished", onDone(result, took));
}

function failed(code, message) {
  const hint = hints.get(code) ?? hints.get("decode");
  job = undefined;
  fire("failed", `${hint.title}.`);
  showError(hint, code === "stopped" ? message : "");
}

function download(file) {
  const link = document.createElement("a");
  link.href = URL.createObjectURL(new Blob([file.blob], { type: "application/octet-stream" }));
  link.download = file.path.split("/").pop();
  link.textContent = `${labels.download} ${file.path} (${fmt(file.blob.size)} bytes)`;
  return link;
}

function table(kind, rows) {
  const columns = byOrder(model.Column.filter((c) => c.table === kind));
  const t = document.createElement("table");
  const head = t.createTHead().insertRow();
  for (const c of columns) { const th = document.createElement("th"); th.scope = "col"; th.textContent = c.label; head.append(th); }
  const body = t.createTBody();
  rows.forEach((row) => {
    const tr = body.insertRow();
    for (const c of columns) tr.insertCell().textContent = String(row[c.key]);
  });
  const wrap = document.createElement("div");
  wrap.className = "table-wrap";
  wrap.append(t);
  return wrap;
}

function showResult(result, nodes) {
  const memory = document.createElement("p");
  memory.className = "memory";
  memory.textContent = `${labels.memory}: ${fmt(Math.round(result.memory / 1048576))} MiB`;
  $("result").replaceChildren(...nodes, memory);
  $("result").hidden = false;
}

function runOperation() {
  const op = operations.get(operation);
  const files = [...chosen("files"), ...chosen("folder")];
  const [archive] = chosen("archive");
  const [original] = chosen("original");
  const subject = operation === "compress" ? describe(files) : archive.name;
  const request = { op: operation, files, archive, original };
  run(request, `${op.running} ${subject}`, (result, took) => {
    if (result.kind === "archive") {
      const [file] = result.files;
      const share = result.input ? ` (${((100 * file.blob.size) / result.input).toFixed(1)}%)` : "";
      showResult(result, [download(file)]);
      return `${fmt(result.input)} → ${fmt(file.blob.size)} bytes${share} in ${seconds(took)}.`;
    }
    if (result.kind === "files") {
      const list = document.createElement("ul");
      for (const file of result.files) { const li = document.createElement("li"); li.append(download(file)); list.append(li); }
      const head = document.createElement("p");
      head.textContent = `${labels.restored}: ${fmt(result.files.length)}`;
      showResult(result, [head, list]);
      const bytes = result.files.reduce((n, f) => n + f.blob.size, 0);
      return `${fmt(result.input)} → ${fmt(bytes)} bytes in ${fmt(result.files.length)} file(s), in ${seconds(took)}.`;
    }
    const rows = result.rows.map((row, i) => ({ ...row, index: i + 1, integrity_ok: row.integrity_ok ? "intact" : "broken" }));
    showResult(result, [table("inspect", rows)]);
    if (result.kind === "table") {
      const ok = result.rows.every((r) => r.integrity_ok);
      return `${fmt(rows.length)} archive(s), integrity ${ok ? "intact" : "broken"}, in ${seconds(took)}.`;
    }
    return result.compared
      ? `Verified: the archive reproduces ${original.name} exactly, in ${seconds(took)}.`
      : `Verified: the archive decodes and every integrity seal holds, in ${seconds(took)}.`;
  });
}

function runExample(example) {
  const seed = `${one("Workbench").purpose} ${one("Workbench").guidance.join(" ")} `;
  run({ op: "example", input: example.input, bytes: example.bytes, seed }, `${example.label}`, (result) => {
    const row = { label: example.label, bytes: fmt(result.bytes), archive: fmt(result.archive), roundTrip: result.same ? example.expectedResponse : "differs", seconds: result.seconds.toFixed(2) };
    const tr = document.createElement("tr");
    for (const c of byOrder(model.Column.filter((x) => x.table === "examples"))) tr.insertCell().textContent = row[c.key];
    $("example-results").tBodies[0].prepend(tr);
    return `${example.label}: ${fmt(result.bytes)} → ${fmt(result.archive)} bytes, round trip ${row.roundTrip}.`;
  });
}

// Controls.
for (const tab of document.querySelectorAll("[role=tab]")) {
  tab.addEventListener("click", () => { operation = tab.dataset.operation; edited(); });
  tab.addEventListener("keydown", (event) => {
    const order = byOrder(model.Operation).map((o) => o.id);
    const step = { ArrowRight: 1, ArrowLeft: -1 }[event.key];
    if (!step) return;
    operation = order[(order.indexOf(operation) + step + order.length) % order.length];
    edited();
    $(`tab-${operation}`).focus();
  });
}
for (const f of fields.values()) {
  $(`field-${f.id}`).addEventListener("change", () => {
    if (f.replaces && chosen(f.id).length) $(`field-${f.replaces}`).value = "";
    edited();
  });
}
$("run").addEventListener("click", runOperation);
$("cancel").addEventListener("click", () => {
  job = undefined;
  fire("cancel");
  restart(() => fire("restarted", labels.cancelled));
});
for (const button of document.querySelectorAll("[data-example]")) {
  const example = model.Example.find((e) => e.id === button.dataset.example);
  button.addEventListener("click", () => runExample(example));
}

fetch("provenance.json").then((r) => (r.ok ? r.json() : {})).then((p) => {
  if (p.version) $("version").textContent = `${$("version").dataset.label} ${p.version}`;
}).catch(() => {});

render();
start(() => fire("loaded"));
