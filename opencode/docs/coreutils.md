# CoreUtils 工具集 — 使用规范

## 环境
- Windows 11，默认 shell 为 PowerShell (pwsh)
- coreutils 已安装于 `C:\Program Files\coreutils`，`bin\` 已在 PATH 中

## 命令调用规则

### 必须加 `.exe` 后缀（12 个，被 PowerShell alias 覆盖）
```
cat.exe  cp.exe  echo.exe  ls.exe  mkdir.exe  mv.exe
pwd.exe  rm.exe  rmdir.exe  sleep.exe  sort.exe  tee.exe
```

### 可直接使用短命令（无冲突）
```
arch  base32  base64  basename  basenc  b2sum  cksum  cmp  comm
csplit  cut  date  df  dfree  diff  dirname  du  env  expr  factor
false  find  fmt  fold  grep  head  hostname  join  la  link  ln
md5sum  mktemp  nl  nproc  numfmt  od  paste  pathchk  pr  printenv
printf  ptx  pwd  readlink  realpath  sed  seq  sha1sum  sha224sum
sha256sum  sha384sum  sha512sum  shuf  split  stat  sum  tac  tail
tar  test  touch  tr  true  truncate  tsort  unexpand  uniq  unlink
uptime  wc  which  winfo  wtop  xargs  yes
```

## 禁止
- ❌ 不要使用 PowerShell cmdlet（`Get-ChildItem`、`Select-String`、`Start-Sleep` 等）
- ❌ 不要使用 PowerShell alias（`gci`、`sls` 等）
- ✅ 所有文件操作优先使用 coreutils 命令

## TTY 差异
opencode 通过 `pwsh -Command` 执行命令，与手动执行行为不一致。**所有冲突命令一律加 `.exe`**，不依赖 `-NoProfile`。

## 详细参考
完整命令列表和用法示例见项目 `opencode/skills/coreutils/SKILL.md`。
