// Verifies the /commands report: row counts, category filter, search, stacked bar.
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CHROME = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
const URL = process.env.PROBE_URL ?? "http://localhost:5173/coreutils/commands";
const PORT = 9337;

const profile = mkdtempSync(join(tmpdir(), "cdp5-"));
const chrome = spawn(
  CHROME,
  ["--headless=new", `--remote-debugging-port=${PORT}`, `--user-data-dir=${profile}`,
   "--no-first-run", "--disable-gpu", "--window-size=1440,1000", "about:blank"],
  { stdio: "ignore" },
);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function getJson(path) {
  for (let i = 0; i < 60; i++) {
    try { const r = await fetch(`http://127.0.0.1:${PORT}${path}`); if (r.ok) return await r.json(); } catch {}
    await sleep(250);
  }
  throw new Error("no cdp");
}

const targets = await getJson("/json/list");
const ws = new WebSocket(targets.find((t) => t.type === "page").webSocketDebuggerUrl);
let id = 0;
const pending = new Map();
ws.addEventListener("message", (e) => {
  const m = JSON.parse(e.data);
  if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); }
});
await new Promise((r) => ws.addEventListener("open", r));
const send = (method, params = {}) =>
  new Promise((res) => { const mid = ++id; pending.set(mid, res); ws.send(JSON.stringify({ id: mid, method, params })); });
const ev = async (expr) => {
  const r = await send("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true });
  return r.result?.exceptionDetails ? { error: r.result.exceptionDetails.exception?.description } : r.result?.result?.value;
};

await send("Page.enable");
await send("Runtime.enable");
await send("Page.navigate", { url: URL });
await sleep(3000);

const initial = await ev(`(() => ({
  rows: document.querySelectorAll('.c-name').length,
  aliases: [...document.querySelectorAll('.r-alias code')].map(e => e.textContent.trim()),
  count: document.querySelector('.r-count')?.textContent.trim().replace(/\\s+/g, ' '),
}))()`);
console.log("initial:", JSON.stringify(initial, null, 1));

await ev(`[...document.querySelectorAll('.r-tab')].find(b => b.textContent.includes('本项目自研')).click()`);
await sleep(400);
const custom = await ev(`(() => ({
  rows: document.querySelectorAll('.c-name').length,
  names: [...document.querySelectorAll('.c-name')].map(e => e.textContent.trim()),
  on: [...document.querySelectorAll('.r-tab.on')].map(b => b.textContent.trim()),
  count: document.querySelector('.r-count')?.textContent.trim().replace(/\\s+/g, ' '),
}))()`);
console.log("\nfilter=custom:", JSON.stringify(custom, null, 1));

await ev(`[...document.querySelectorAll('.r-tab')].find(b => b.textContent.includes('uutils')).click()`);
await sleep(400);
const uu = await ev(`(() => ({
  rows: document.querySelectorAll('.c-name').length,
  names: [...document.querySelectorAll('.c-name')].map(e => e.textContent.trim()),
}))()`);
console.log("\nfilter=uutils:", JSON.stringify(uu));

await ev(`[...document.querySelectorAll('.r-tab')].find(b => b.textContent.trim() === '全部').click()`);
await sleep(200);
await ev(`(() => { const i = document.querySelector('.r-search'); i.value = 'sum'; i.dispatchEvent(new Event('input', { bubbles: true })); })()`);
await sleep(400);
const searched = await ev(`(() => ({
  rows: document.querySelectorAll('.c-name').length,
  names: [...document.querySelectorAll('.c-name')].map(e => e.textContent.trim()),
}))()`);
console.log("\nsearch='sum':", JSON.stringify(searched));

const bar = await ev(`(() => [...document.querySelectorAll('.r-stack-cell')].map(e => ({
  pct: e.textContent.trim(),
  w: Math.round(e.getBoundingClientRect().width),
  bg: getComputedStyle(e).backgroundColor,
})))()`);
console.log("\nstacked bar:", JSON.stringify(bar));

const legend = await ev(`(() => [...document.querySelectorAll('.r-legend-row')].map(r => ({
  name: r.querySelector('.r-legend-name')?.textContent.trim(),
  pct: r.querySelector('.r-legend-num')?.textContent.trim(),
  meta: r.querySelector('.r-legend-meta')?.textContent.trim().replace(/\\s+/g,' '),
  fillPct: r.querySelector('.r-legend-fill')?.style.width,
})))()`);
console.log("\nlegend:", JSON.stringify(legend, null, 1));

ws.close();
chrome.kill();
