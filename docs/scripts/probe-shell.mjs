// Compares the header on both pages: same height, same palette, same nav items,
// and checks that every nav link resolves (no 404).
import { spawn } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CHROME = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
const BASE = process.env.BASE ?? "http://localhost:4174/coreutils";
const PORT = 9345;

const profile = mkdtempSync(join(tmpdir(), "cdpd-"));
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

const probe = `(() => {
  const bar = document.querySelector('.m-bar');
  if (!bar) return { error: 'no .m-bar' };
  const cs = getComputedStyle(bar);
  const r = bar.getBoundingClientRect();
  return {
    path: location.pathname + location.search,
    barH: Math.round(r.height),
    barX: Math.round(r.x),
    borderBottom: cs.borderBottomWidth + ' ' + cs.borderBottomColor,
    bg: cs.backgroundColor,
    sticky: cs.position,
    brand: bar.querySelector('.m-brand-name')?.textContent.trim(),
    nav: [...bar.querySelectorAll('.m-nav a')].map(a => a.textContent.trim()),
    active: [...bar.querySelectorAll('.m-nav a.on')].map(a => a.textContent.trim()),
    navHrefs: [...bar.querySelectorAll('.m-nav a')].map(a => a.getAttribute('href')),
    viteChrome: document.querySelectorAll('.VPNav,.VPSidebar,.VPDoc,.VPLocalNav').length,
    hasTable: !!document.querySelector('.r-table'),
    hasHero: !!document.querySelector('.m-hero'),
  };
})()`;

const out = {};
for (const [name, url] of [["home", `${BASE}/`], ["report", `${BASE}/commands`]]) {
  await send("Page.navigate", { url });
  await sleep(2600);
  out[name] = await ev(probe);
}
console.log(JSON.stringify(out, null, 1));

// do the nav targets actually resolve?
const hrefs = [...new Set(out.home.navHrefs.filter((h) => h && !h.startsWith("http")))];
const status = {};
for (const h of hrefs) {
  await send("Page.navigate", { url: `http://localhost:4174${h}` });
  await sleep(900);
  status[h] = await ev(`(() => ({ title: document.title, headings: document.querySelectorAll('h1,h2').length }))()`);
}
console.log("\nnav targets:", JSON.stringify(status, null, 1));

ws.close();
chrome.kill();
