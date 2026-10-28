// Checks the 说明 column: coverage, wrapping, and horizontal overflow.
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CHROME = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
const URL = process.env.PROBE_URL ?? "http://localhost:4174/coreutils/commands";
const PORT = 9342;

const profile = mkdtempSync(join(tmpdir(), "cdpa-"));
const chrome = spawn(
  CHROME,
  ["--headless=new", `--remote-debugging-port=${PORT}`, `--user-data-dir=${profile}`,
   "--no-first-run", "--disable-gpu", "--window-size=1440,1000", "about:blank"],
  { stdio: "ignore" },
);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function getJson(p) {
  for (let i = 0; i < 60; i++) {
    try { const r = await fetch(`http://127.0.0.1:${PORT}${p}`); if (r.ok) return await r.json(); } catch {}
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

console.log(JSON.stringify(await ev(`(() => {
  const rows = [...document.querySelectorAll('.r-table tbody tr')];
  const descs = rows.map(r => r.querySelector('.c-desc')?.textContent?.trim() ?? '');
  const cells = [...document.querySelectorAll('.c-desc')];
  return {
    rows: rows.length,
    emptyDescs: descs.filter(d => !d).length,
    uniqueDescs: new Set(descs).size,
    multiLineCells: cells.filter(c => c.getBoundingClientRect().height > 46).length,
    cellHeights: [...new Set(cells.map(c => Math.round(c.getBoundingClientRect().height)))].sort((a, b) => a - b),
    shortRows: rows
      .map(r => ({ n: r.querySelector('.c-name')?.textContent.trim(), h: Math.round(r.getBoundingClientRect().height) }))
      .filter(r => r.h < 36),
    descColW: Math.round(cells[0]?.getBoundingClientRect().width ?? 0),
    tableW: Math.round(document.querySelector('.r-table').getBoundingClientRect().width),
    overflowX: document.documentElement.scrollWidth > window.innerWidth + 1,
    longest3: descs.slice().sort((a, b) => b.length - a.length).slice(0, 3),
  };
})()`), null, 1));

ws.close();
chrome.kill();
