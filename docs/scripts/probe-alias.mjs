// Verifies the 共用实现 list is a 3-column aligned grid: equal x per column,
// increasing y per row, nothing wrapping.
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CHROME = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
const URL = process.env.PROBE_URL ?? "http://localhost:4174/coreutils/commands";
const PORT = 9344;

const profile = mkdtempSync(join(tmpdir(), "cdpc-"));
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

const m = await ev(`(() => {
  const ul = document.querySelector('.r-alias ul');
  const rows = [...ul.querySelectorAll('.a-name')].map(name => {
    const li = name.closest('li');
    const arrow = li.querySelector('.a-arrow');
    const target = li.querySelector('.a-target');
    const b = el => { const r = el.getBoundingClientRect(); return { x: Math.round(r.x), y: Math.round(r.y), w: Math.round(r.width) }; };
    return { name: name.textContent.trim(), name: b(name), arrow: b(arrow), target: b(target) };
  });
  const colX = {
    name: [...new Set(rows.map(r => r.name.x))],
    arrow: [...new Set(rows.map(r => r.arrow.x))],
    target: [...new Set(rows.map(r => r.target.x))],
  };
  return {
    rows,
    colX,
    aligned: colX.name.length === 1 && colX.arrow.length === 1 && colX.target.length === 1,
    ulDisplay: getComputedStyle(ul).display,
  };
})()`);
console.log(JSON.stringify(m, null, 1));

ws.close();
chrome.kill();
