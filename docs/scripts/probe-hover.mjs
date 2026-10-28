// Drives headless Chrome over CDP to check that hovering a treemap tile
// actually mutates the DOM. Node 22+ has a global WebSocket, so no deps.
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CHROME = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
const URL = process.env.PROBE_URL ?? "http://localhost:5173/coreutils/";
const PORT = 9333;

const profile = mkdtempSync(join(tmpdir(), "cdp-"));
const chrome = spawn(
  CHROME,
  [
    "--headless=new",
    `--remote-debugging-port=${PORT}`,
    `--user-data-dir=${profile}`,
    "--no-first-run",
    "--disable-gpu",
    "--window-size=1440,900",
    "about:blank",
  ],
  { stdio: "ignore" },
);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function getJson(path) {
  for (let i = 0; i < 60; i++) {
    try {
      const r = await fetch(`http://127.0.0.1:${PORT}${path}`);
      if (r.ok) return await r.json();
    } catch {}
    await sleep(250);
  }
  throw new Error("CDP not ready");
}

const targets = await getJson("/json/list");
const page = targets.find((t) => t.type === "page");
const ws = new WebSocket(page.webSocketDebuggerUrl);

let id = 0;
const pending = new Map();
const logs = [];

ws.addEventListener("message", (e) => {
  const msg = JSON.parse(e.data);
  if (msg.id && pending.has(msg.id)) {
    pending.get(msg.id)(msg);
    pending.delete(msg.id);
  }
  if (msg.method === "Runtime.consoleAPICalled") {
    logs.push(msg.params.args.map((a) => a.value ?? a.description).join(" "));
  }
  if (msg.method === "Runtime.exceptionThrown") {
    logs.push("EXCEPTION: " + (msg.params.exceptionDetails.exception?.description ?? ""));
  }
});

await new Promise((r) => ws.addEventListener("open", r));

const send = (method, params = {}) =>
  new Promise((res) => {
    const mid = ++id;
    pending.set(mid, res);
    ws.send(JSON.stringify({ id: mid, method, params }));
  });

const evaluate = async (expr) => {
  const r = await send("Runtime.evaluate", {
    expression: expr,
    returnByValue: true,
    awaitPromise: true,
  });
  if (r.result?.exceptionDetails) {
    return { error: r.result.exceptionDetails.exception?.description };
  }
  return r.result?.result?.value;
};

await send("Page.enable");
await send("Runtime.enable");
await send("Page.navigate", { url: URL });
await sleep(3500);

// --- what did we actually get? ---
const survey = await evaluate(`
(() => {
  const tiles = [...document.querySelectorAll('g.m-tile')];
  const svg = document.querySelector('svg.m-map-svg');
  return {
    tiles: tiles.length,
    svg: !!svg,
    svgBox: svg ? svg.getBoundingClientRect().toJSON() : null,
    firstFill: tiles[0]?.querySelector('rect')?.getAttribute('fill') ?? null,
    labels: tiles.map(t => t.querySelector('.m-tile-label')?.textContent?.trim() ?? ''),
  };
})()
`);
console.log("survey:", JSON.stringify(survey, null, 2));

if (!survey.tiles) {
  console.log("no tiles rendered — abort");
  ws.close();
  chrome.kill();
  process.exit(1);
}

// --- hover the 2nd tile, via real mouse events ---
const boxes = await evaluate(`
(() => [...document.querySelectorAll('g.m-tile')].map(t => {
  const r = t.querySelector('rect').getBoundingClientRect();
  return { cx: r.left + r.width / 2, cy: r.top + r.height / 2, label: t.querySelector('.m-tile-label')?.textContent?.trim() };
}))()
`);

const target = boxes[1] ?? boxes[0];
console.log(`\nhovering: ${target.label} @ (${target.cx.toFixed(0)}, ${target.cy.toFixed(0)})`);

const before = await evaluate(`
(() => [...document.querySelectorAll('g.m-tile')].map(t => ({
  label: t.querySelector('.m-tile-label')?.textContent?.trim(),
  fill: t.querySelector('rect').getAttribute('fill'),
  cls: t.getAttribute('class'),
})))()
`);

await send("Input.dispatchMouseEvent", {
  type: "mouseMoved",
  x: Math.round(target.cx),
  y: Math.round(target.cy),
  buttons: 0,
});
await sleep(500);

const after = await evaluate(`
(() => [...document.querySelectorAll('g.m-tile')].map(t => ({
  label: t.querySelector('.m-tile-label')?.textContent?.trim(),
  fill: t.querySelector('rect').getAttribute('fill'),
  cls: t.getAttribute('class'),
})))()
`);

console.log("\n  label      before     after      class-change");
for (let i = 0; i < before.length; i++) {
  const b = before[i], a = after[i];
  const changed = b.fill !== a.fill;
  console.log(
    `  ${(b.label ?? "").padEnd(9)} ${b.fill.padEnd(10)} ${a.fill.padEnd(10)} ` +
      `${changed ? "FILL CHANGED" : "-"} ${a.cls !== b.cls ? "| class: " + a.cls : ""}`,
  );
}

const changedCount = before.filter((b, i) => b.fill !== after[i].fill).length;
console.log(`\nchanged tiles: ${changedCount} (expect exactly 1)`);

// caption should follow the hovered tile
const caption = await evaluate(
  `document.querySelector('.m-map-caption-title')?.textContent?.trim()`,
);
console.log("caption:", caption);

if (logs.length) console.log("\nconsole:\n" + logs.join("\n"));

ws.close();
chrome.kill();
