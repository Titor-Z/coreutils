#!/usr/bin/env node
// Generates docs/data/commands.json from the built binary + source tree.
// Run: node docs/scripts/gen-data.mjs

import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, statSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const BIN = join(ROOT, "target", "release", "coreutils.exe");

const exists = (p) => { try { statSync(p); return true; } catch { return false; } };

// Maps a command to the source directory/file that implements it.
function sourceOf(name, category) {
  const ms = join(ROOT, "deps/coreutils/src/uu", name === "[" ? "test" : name, "src");
  if (category === "microsoft") return ms;

  switch (name) {
    case "find":   return join(ROOT, "deps/findutils/src/find");
    case "xargs":  return join(ROOT, "deps/findutils/src/xargs");
    case "grep":   return join(ROOT, "deps/grep/src");
    case "sed":    return join(ROOT, "deps/sed/src/sed");
    case "tar":    return join(ROOT, "deps/tar/src/uu");
    case "cmp":    return join(ROOT, "deps/diffutils/src/cmp.rs");
    case "diff":   return join(ROOT, "deps/diffutils/src");
    case "dfree":  return join(ROOT, "src/dfree");
    case "free":   return join(ROOT, "src/free.rs");
    case "winfo":  return join(ROOT, "src/winfo.rs");
    case "wtop":   return join(ROOT, "src/wtop.rs");
    case "which":  return join(ROOT, "crates/uu_which/src");
    case "was":    return join(ROOT, "src/was.rs");
    case "unwas":  return join(ROOT, "src/was.rs");
    case "la":     return join(ROOT, "deps/coreutils/src/uu/ls/src");
    default:       return exists(ms) ? ms : null;
  }
}

// diff and cmp share one crate; cmp.rs belongs to cmp, the rest to diff.
const DIFFUTILS_SHARED = new Set(["lib.rs", "main.rs", "macros.rs"]);

function countLoc(p, isDiff) {
  let files = 0, lines = 0;
  const walk = (target) => {
    const st = statSync(target);
    if (st.isFile()) {
      if (!target.endsWith(".rs")) return;
      const base = target.split(/[\\/]/).pop();
      if (isDiff && (base === "cmp.rs" || DIFFUTILS_SHARED.has(base))) return;
      files++;
      lines += readFileSync(target, "utf8").split("\n").length;
      return;
    }
    for (const entry of readdirSync(target)) walk(join(target, entry));
  };
  try { walk(p); } catch { /* missing source */ }
  return { files, lines };
}

const raw = execFileSync(BIN, ["--list-raw"], { encoding: "utf8" })
  .split("\n").map((l) => l.trim()).filter(Boolean);

const list = execFileSync(BIN, ["--list"], { encoding: "utf8" });

const categoryOf = new Map();
for (const line of list.split("\n")) {
  const m = line.match(/^(\S+)\s+\[(\w+)\]/);
  if (m) categoryOf.set(m[1], m[2]);
}

const seen = new Map();
// Aliases must not claim a source directory that a canonical command owns.
const ALIASES = new Set(["[", "la", "dir", "vdir"]);
// When several commands share one source file, the one that owns it is picked by
// name rather than by argv order, so `was` (not `unwas`) carries src/was.rs.
const PRIMARY = new Set(["was"]);
const ordered = [...raw].sort(
  (a, b) =>
    Number(ALIASES.has(a)) - Number(ALIASES.has(b)) ||
    Number(!PRIMARY.has(a)) - Number(!PRIMARY.has(b)),
);
const resolved = new Map();
for (const name of ordered) {
  const category = categoryOf.get(name) ?? "microsoft";
  const src = sourceOf(name, category);
  const key = src ?? name;
  if (seen.has(key)) continue;
  const { files, lines } = src ? countLoc(src, name === "diff") : { files: 0, lines: 0 };
  seen.set(key, name);
  resolved.set(name, { category, files, lines, source: src });
}

const commands = raw.map((name) => {
  const own = resolved.get(name);
  const category = categoryOf.get(name) ?? "microsoft";
  if (own) {
    return {
      name,
      category,
      loc: own.lines,
      files: own.files,
      aliasOf: null,
      source: own.source ? relative(ROOT, own.source).split("\\").join("/") : null,
    };
  }
  const src = sourceOf(name, category);
  const owner = src ? seen.get(src) : null;
  return {
    name,
    category,
    loc: 0,
    files: 0,
    aliasOf: owner ?? null,
    source: src ? relative(ROOT, src).split("\\").join("/") : null,
  };
});

const counts = commands.reduce((acc, c) => {
  acc[c.category] = (acc[c.category] ?? 0) + 1;
  return acc;
}, {});

const out = {
  generatedAt: new Date().toISOString().slice(0, 10),
  binaryBytes: statSync(BIN).size,
  total: commands.length,
  counts,
  totalLoc: commands.reduce((s, c) => s + c.loc, 0),
  commands: commands.sort((a, b) => b.loc - a.loc || a.name.localeCompare(b.name)),
};

mkdirSync(join(ROOT, "docs", "data"), { recursive: true });
writeFileSync(join(ROOT, "docs", "data", "commands.json"), JSON.stringify(out, null, 2) + "\n");

console.log(`commands: ${out.total}  (${Object.entries(counts).map(([k, v]) => `${k}=${v}`).join(" ")})`);
console.log(`total LOC: ${out.totalLoc}`);
console.log(`binary: ${(out.binaryBytes / 1048576).toFixed(2)} MiB`);
console.log("by category LOC:");
for (const cat of ["microsoft", "uutils", "custom"]) {
  const loc = commands.filter((c) => c.category === cat).reduce((s, c) => s + c.loc, 0);
  console.log(`  ${cat.padEnd(10)} ${String(loc).padStart(7)}  (${((loc / out.totalLoc) * 100).toFixed(1)}%)`);
}
