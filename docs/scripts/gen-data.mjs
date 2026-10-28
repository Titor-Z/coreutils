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

// One-line Chinese descriptions. Kept here rather than scraped from `--help`
// because that output is English, absent on several commands, and outright
// broken on others (`diff` rejects --help, `which` prints a placeholder).
// gen-data warns about any command missing an entry, so new commands cannot
// silently ship without one.
const DESCRIPTIONS = {
  "[": "test 的方括号形式",
  arch: "显示机器硬件架构",
  b2sum: "计算 BLAKE2 校验和",
  base32: "base32 编解码",
  base64: "base64 编解码",
  basename: "去掉目录前缀，只保留文件名",
  basenc: "按指定 base 做编解码",
  cat: "连接文件并输出到标准输出",
  cksum: "计算 CRC 校验和与字节数",
  cmp: "逐字节比较两个文件",
  comm: "比较两个已排序文件的行",
  cp: "复制文件或目录",
  csplit: "按模式把文件切分成多段",
  cut: "按字段或字符位置截取每行",
  date: "显示或设置日期时间",
  df: "显示文件系统磁盘占用",
  dfree: "磁盘占用交互分析器（TUI）",
  diff: "比较文件差异",
  dirname: "去掉文件名，只保留目录部分",
  du: "统计目录占用空间",
  echo: "输出一行文本",
  env: "在修改后的环境中运行程序",
  expr: "求值表达式",
  factor: "质因数分解",
  false: "总是返回失败",
  find: "按条件递归查找文件",
  fmt: "把段落重排为指定宽度",
  fold: "按宽度折行",
  free: "显示内存与交换区用量",
  grep: "按模式搜索文本",
  head: "输出文件开头部分",
  hostname: "显示主机名",
  join: "按关键字段连接两个已排序文件的行",
  la: "ls -A 的别名",
  link: "为文件创建硬链接",
  ln: "创建硬链接或符号链接",
  ls: "列出目录内容",
  md5sum: "计算 MD5 校验和",
  mkdir: "创建目录",
  mktemp: "创建临时文件或目录",
  mv: "移动或重命名文件",
  nl: "给行加编号后输出",
  nproc: "显示可用处理器数量",
  numfmt: "数字与人类可读单位互转",
  od: "以八进制等格式转储二进制",
  paste: "按列合并多个文件的行",
  pathchk: "检查路径名是否有效且可移植",
  pr: "分页排版后输出",
  printenv: "打印环境变量",
  printf: "按格式输出",
  ptx: "生成排列索引",
  pwd: "显示当前工作目录",
  readlink: "打印符号链接指向的路径",
  realpath: "解析为绝对规范路径",
  rm: "删除文件或目录",
  rmdir: "删除空目录",
  sed: "流编辑器，按脚本转换文本",
  seq: "生成数字序列",
  sha1sum: "计算 SHA-1 校验和",
  sha224sum: "计算 SHA-224 校验和",
  sha256sum: "计算 SHA-256 校验和",
  sha384sum: "计算 SHA-384 校验和",
  sha512sum: "计算 SHA-512 校验和",
  shuf: "随机打乱行的顺序",
  sleep: "延时指定时长",
  sort: "对行排序",
  split: "把文件切分成多个片段",
  stat: "显示文件元数据",
  sum: "计算 BSD 校验和",
  tac: "逆序输出行",
  tail: "输出文件末尾部分",
  tar: "打包与解包归档",
  tee: "同时写入文件与标准输出",
  test: "求值条件表达式",
  touch: "更新时间戳或创建空文件",
  tr: "转换或删除字符",
  true: "总是返回成功",
  truncate: "把文件截断或扩展到指定长度",
  tsort: "拓扑排序",
  unexpand: "把空格转为制表符",
  uniq: "去除相邻重复行",
  unlink: "删除单个文件",
  unwas: "删除 was 管理的别名",
  uptime: "显示开机时长与平均负载",
  was: "管理 PowerShell 别名",
  wc: "统计行数、词数与字节数",
  which: "在 PATH 中定位可执行文件",
  winfo: "系统信息一览",
  wtop: "进程监视器（TUI）",
  xargs: "由标准输入构造并执行命令",
  yes: "不停重复输出指定字符串",
};
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
  const desc = DESCRIPTIONS[name] ?? null;
  if (own) {
    return {
      name,
      category,
      desc,
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
    desc,
    loc: 0,
    files: 0,
    aliasOf: owner ?? null,
    source: src ? relative(ROOT, src).split("\\").join("/") : null,
  };
});

const missingDesc = commands.filter((c) => !c.desc).map((c) => c.name);

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

if (missingDesc.length) {
  console.warn(`\nWARN: ${missingDesc.length} command(s) 缺少一句话说明：`);
  console.warn("  " + missingDesc.join(", "));
  console.warn("  请在 scripts/gen-data.mjs 的 DESCRIPTIONS 中补上。");
  process.exitCode = 1;
}
