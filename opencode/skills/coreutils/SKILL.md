---
name: coreutils
description: Complete reference for coreutils commands on Windows, including .exe suffix rules, command categories, and TTY behavioral differences
---

# Coreutils 工具集 — 完整参考

## 概述

**coreutils** 是一个用 Rust 编写的、让用户在 Windows 上使用 Linux 风格 bash 命令的工具集。基于 Microsoft/coreutils fork，额外集成了 uutils 生态包和自研 Windows 原生工具。

- **运行环境**: Windows 11，默认 shell 为 PowerShell (pwsh)
- **安装位置**: `C:\Program Files\coreutils`（`bin\` 已在 PATH 中）
- **总命令数**: 83 个

---

## 命令列表

### [microsoft] — Microsoft/coreutils 原生（71 个）

来自 `deps/coreutils/src/uu/*`，标准 Unix 工具：

```
arch       base32     base64     basename   basenc     b2sum      cat
cksum      comm       cp         csplit     cut        date       df
dirname    du         echo       env        expr       factor     false
fmt        fold       head       hostname   join       link       ln
ls         md5sum     mkdir      mktemp     mv         nl         nproc
numfmt     od         paste      pathchk    pr         printenv   printf
ptx        pwd        readlink   realpath   rm         rmdir      seq
sha1sum    sha224sum  sha256sum  sha384sum  sha512sum  shuf       sleep
sort       split      stat       sum        tac        tail       tee
test       touch      tr         true       truncate   tsort      unexpand
uniq       unlink     uptime     wc         yes        [          dir
```

> `dir` 和 `vdir` 是 `ls` 的别名，`[` 是 `test` 的别名

### [uutils] — 额外 uutils 生态包（7 个）

来自 uutils 组织下独立仓库的工具：

| 命令 | 来源 | 说明 |
|------|------|------|
| `cmp` | uutils/diffutils | 逐字节比较两个文件 |
| `diff` | uutils/diffutils | 逐行比较文件差异 |
| `find` | uutils/findutils | 搜索目录层次结构中的文件 |
| `grep` | uutils/grep | 打印与模式匹配的行 |
| `sed` | uutils/sed | 流编辑器，转换文本 |
| `tar` | uutils/tar | 归档工具 |
| `xargs` | uutils/findutils | 从标准输入生成和执行命令行 |

### [custom] — 自研扩展（5 个）

| 命令 | 说明 | 使用方式 |
|------|------|---------|
| `dfree` | TUI 磁盘存储空间分析工具 | `dfree`（需选择磁盘） |
| `la` | `ls -A` 的别名（显示隐藏文件） | `la` |
| `which` | 定位命令在 PATH 中的位置 | `which <cmd>`，`which -a <cmd>` |
| `winfo` | 系统信息 TUI（类似 neofetch） | `winfo` |
| `wtop` | 进程监视器 TUI | `wtop` |

---

## 冲突命令表 — 必须加 `.exe` 后缀

PowerShell 为以下命令设置了内置 alias，直接使用命令名会被 PowerShell 拦截：

| 命令 | PowerShell 拦截对象 | 正确用法 |
|------|--------------------|---------|
| `cat` | `Get-Content` (alias) | `cat.exe` |
| `cp` | `Copy-Item` (alias) | `cp.exe` |
| `echo` | `Write-Output` (alias) | `echo.exe` |
| `ls` | `Get-ChildItem` (alias) | `ls.exe` |
| `mkdir` | `New-Item` (alias) | `mkdir.exe` |
| `mv` | `Move-Item` (alias) | `mv.exe` |
| `pwd` | `Get-Location` (alias) | `pwd.exe` |
| `rm` | `Remove-Item` (alias) | `rm.exe` |
| `rmdir` | `Remove-Item` (alias) | `rmdir.exe` |
| `sleep` | `Start-Sleep` (alias) | `sleep.exe` |
| `sort` | `Sort-Object` (alias) | `sort.exe` |
| `tee` | `Tee-Object` (alias) | `tee.exe` |

**规则**: 上表中 12 个命令，在 opencode 中执行时必须加 `.exe` 后缀。

---

## 无冲突命令 — 可直接使用

以下命令与 PowerShell 无冲突，直接使用短命令名即可：

```
arch         base32       base64       basename     basenc       b2sum
cksum        cmp          comm         csplit       cut          date
df           dfree        diff         dirname      du           env
expr         factor       false        find         fmt          fold
grep         head         hostname     join         la           link
ln           md5sum       mktemp       nl           nproc        numfmt
od           paste        pathchk      pr           printenv     printf
ptx          pwd          readlink     realpath     sed          seq
sha1sum      sha224sum    sha256sum    sha384sum    sha512sum    shuf
split        stat         sum          tac          tail         tar
test         touch        tr           true         truncate     tsort
unexpand     uniq         unlink       uptime       wc           which
winfo        wtop         xargs        yes
```

---

## TTY 执行差异

### 问题

opencode 通过 `pwsh -Command` 执行命令时，其命令解析行为与用户在终端中手动执行不完全一致：

| 场景 | 行为 |
|------|------|
| 用户手动执行 | 加载 PowerShell profile → alias 生效 → `ls` = `Get-ChildItem` |
| opencode `pwsh -Command` | **如果使用 `-NoProfile`**，alias 不加载；否则 alias 会加载 |
| opencode `pwsh -File` | 同样可能受 profile 影响 |

### 解决方案

1. **冲突命令一律加 `.exe`**（见上表），无论 profile 是否加载
2. **不要依赖 `-NoProfile` 来绕开冲突**，因为 opencode 可能在不同模式下运行
3. **所有文件操作优先使用 coreutils 命令**，而非 PowerShell cmdlet

### 禁止

```
Get-ChildItem     ❌  →  ls.exe     ✅
Select-String     ❌  →  grep       ✅
Select-Object     ❌  →  sort.exe   ✅
Where-Object      ❌  →  grep       ✅
ForEach-Object    ❌  →  for/while  ✅
Start-Sleep       ❌  →  sleep.exe  ✅
Write-Output      ❌  →  echo.exe   ✅
Get-Content       ❌  →  cat.exe    ✅
Copy-Item         ❌  →  cp.exe     ✅
Move-Item         ❌  →  mv.exe     ✅
Remove-Item       ❌  →  rm.exe     ✅
```

---

## 常用命令用法速查

### 文件操作

```bash
# 列出目录（含隐藏文件）
ls.exe -la
la

# 查看文件内容
cat.exe file.txt
head -n 20 file.txt
tail -n 10 file.txt
tac file.txt          # 倒序输出

# 复制/移动/删除
cp.exe source dest
mv.exe source dest
rm.exe file

# 查找文件
find . -name "*.rs"
grep -r "pattern" .

# 比较
diff file1 file2
cmp file1 file2        # 逐字节比较
```

### 文本处理

```bash
# 排序
sort.exe file.txt
sort.exe -u file.txt   # 去重排序

# 统计
wc file.txt            # 行数/词数/字符数
wc -l file.txt         # 仅行数

# 转换
tr 'a-z' 'A-Z' < file.txt   # 小写转大写
sed 's/old/new/g' file.txt  # 替换
```

### 系统信息

```bash
# 磁盘空间
df -h                 # 命令行
dfree                 # TUI 全屏分析

# 系统信息
winfo                 # TUI neofetch 风格

# 进程
wtop                  # TUI 进程监视器

# 定位命令
which cmd             # 查找命令路径
which -a cmd          # 查找所有匹配路径
```

### 归档

```bash
tar -cf archive.tar dir/
tar -xf archive.tar
tar -czf archive.tar.gz dir/
```

### Shell 管道

```bash
# 标准管道
cat.exe file.txt | grep "error" | sort.exe | uniq

# xargs（标准输入转参数）
find . -name "*.tmp" | xargs rm.exe

# tee（同时输出到文件）
echo.exe "data" | tee output.txt
```

---

## 关于 command --list

`coreutils.exe --list` 输出所有可用命令及分类标签：

```bash
coreutils.exe --list
# arch            [microsoft]
# cmp             [uutils]
# dfree           [custom]
# ...
```

标签含义：
- `[microsoft]` — Microsoft fork 中的核心工具
- `[uutils]` — 额外引入的 uutils 生态工具
- `[custom]` — 自研 Windows 原生工具
