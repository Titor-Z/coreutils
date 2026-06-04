<h1 align="center">coreutils (Windows 增强版)</h1>

<p align="center">UNIX 风格命令工具集，原生 Windows 可执行文件。</p>

<h3 align="center">
  <a href="#usage">Usage</a>
  <span> · </span>
  <a href="#added-commands">Added commands</a>
  <span> · </span>
  <a href="#build">Build</a>
  <span> · </span>
  <a href="#windows-caveats">Windows caveats</a>
</h3>

---

Fork of [microsoft/coreutils](https://github.com/microsoft/coreutils) 基础上扩展，
集成了 [uutils/coreutils](https://github.com/uutils/coreutils)、
[findutils](https://github.com/uutils/findutils)、[grep](https://github.com/uutils/grep)、
[sed](https://github.com/uutils/sed)、[diffutils](https://github.com/uutils/diffutils)、
以及 [tar](https://github.com/uutils/tar)，打包成一个多调用二进制文件。在 Windows 上原生运行，
无需 WSL / Cygwin / MSYS2。

共 **85 个命令**，覆盖日常 Linux 命令行的绝大部分需求。

**自用 fork，不提供官方安装包。**

<br/>

## Usage

```bash
coreutils.exe <command> [args...]    # 直接使用
coreutils.exe --list                  # 列出所有可用命令
```

创建硬链接后可以直接当独立命令使用：

```bash
# 在 target\debug\ 目录下创建硬链接
New-Item -ItemType HardLink -Path "sed.exe" -Target "coreutils.exe"
New-Item -ItemType HardLink -Path "diff.exe" -Target "coreutils.exe"
New-Item -ItemType HardLink -Path "tar.exe" -Target "coreutils.exe"
New-Item -ItemType HardLink -Path "dfree.exe" -Target "coreutils.exe"
```

<br/>

## Added commands

该 fork 在 Microsoft 原版 72 个命令的基础上增加了以下命令：

| Command | Source | Description |
| ------- | ------ | ----------- |
| `sed` | [uutils/sed](https://github.com/uutils/sed) | 流编辑器，支持 `s/pattern/replace/`、`-n`、`/address/` 等标准功能 |
| `diff` | [uutils/diffutils](https://github.com/uutils/diffutils) | 文件差异比较，支持 unified diff (`-u`) |
| `cmp` | [uutils/diffutils](https://github.com/uutils/diffutils) | 字节级文件比较 |
| `tar` | [uutils/tar](https://github.com/uutils/tar) | 归档工具，支持 create / list / extract |
| `dfree` | 自定义 | 实时内存 & 磁盘使用率监控，3 秒刷新，彩色进度条显示 |
| `winfo` | 自定义 | 系统信息显示，类似 neofetch，显示 OS、内核、运行时间、Shell、CPU、内存、磁盘 |

### dfree

实时显示物理内存、虚拟内存（Swap）和各硬盘分区的使用情况：

```
 ┌─────────────────────────────────────────────┐
 │            dfree v1  内存 & 磁盘监控            │
 └─────────────────────────────────────────────┘

  Memory  ███████████████████░░░░░  2.9 GiB / 3.9 GiB  74.5%
  Swap    ████████████████░░░░░░░░  7.0 GiB / 10.7 GiB  65.6%

  ── Disks ──────────────────────────────────
  C: █████████████░░░░░░░░░░░░  61.1 GiB / 118.3 GiB  51.7%
  D: ██████░░░░░░░░░░░░░░░░░░░  60.0 GiB / 200.0 GiB  30.0%

 ──────────────────────────────────────────────────
  Refresh: 3s   Ctrl+C to exit
```

```bash
coreutils.exe dfree          # 默认 3 秒刷新
coreutils.exe dfree -n 5     # 5 秒刷新
dfree.exe                    # 硬链接后直接跑
```

<br/>

## Build

需要 Rust 工具链（nightly）和 MinGW-w64 GCC（用于编译 `sort` 命令的 C 代码）。

```powershell
# 子模块初始化（coreutils、findutils、grep、sed）
git submodule update --init --recursive

# 编译（需要 RUSTC_BOOTSTRAP=1 环境变量）
$env:RUSTC_BOOTSTRAP=1
cargo build --release
```

编译产物在 `target/release/coreutils.exe`。

<br/>

## Shell conflicts

> [!NOTE]
> Any command not mentioned is included in this suite. The following only lists conflicts.

> [!WARNING]
> PowerShell 7.4 or newer is required. Older PowerShell versions aren't supported.

Several commands share names with built-ins in CMD and PowerShell. Whether the Coreutils
version runs depends on the shell, the PATH order, and (for PowerShell) the alias table.

Legend: ✅ ships and works · ⚠️ ships but conflicts with a built-in · 🛑 not shipped

| Command    | CMD  | PowerShell 7.4+ | Notes |
| ---------- | :--: | :-------------: | ----- |
| `cat`      |  ✅  |       ⚠️        | |
| `cp`       |  ✅  |       ⚠️        | |
| `date`     |  ⚠️  |       ⚠️        | |
| `dir`      |  🛑  |       🛑        | Conflicts with the built-in DOS command |
| `echo`     |  ⚠️  |       ⚠️        | |
| `expand`   |  🛑  |       🛑        | Conflicts with the built-in DOS command |
| `find`     |  ✅  |       ✅        | Integrated port of the original DOS command |
| `hostname` |  ✅  |       ✅        | Superset of the Windows built-in |
| `kill`     |  🛑  |       🛑        | Unavailable due to lack of signals on Windows; Implementing a form of SIGTERM/SIGKILL may be possible in the future however |
| `ls`       |  ✅  |       ⚠️        | |
| `mkdir`    |  ⚠️  |       ⚠️        | |
| `more`     |  🛑  |       🛑        | Conflicts with the built-in DOS command (consider `edit` as an alternative) |
| `mv`       |  ✅  |       ⚠️        | |
| `pwd`      |  ✅  |       ⚠️        | |
| `rm`       |  ✅  |       ⚠️        | |
| `rmdir`    |  ⚠️  |       ⚠️        | |
| `sleep`    |  ✅  |       ⚠️        | |
| `sed`      |  ✅  |       ⚠️        | |
| `sort`     |  ✅  |       ✅        | Integrated port of the original DOS command |
| `tar`      |  ✅  |       ✅        | |
| `tee`      |  ✅  |       ⚠️        | |
| `timeout`  |  🛑  |       🛑        | Relies on `kill`'s functionality |
| `uptime`   |  ✅  |       ⚠️        | |
| `whoami`   |  🛑  |       🛑        | Conflicts with the built-in Windows command |

<br/>

## Windows caveats

| Difference            | Detail |
| --------------------- | ------ |
| **CRLF line endings** | Windows text files often use CRLF (`\r\n`). Most utilities handle this transparently, but pattern matching with `$` and exact byte counts can be affected. |
| **No `/dev/null`**    | Use `NUL` instead, for example `find . -name "*.log" > NUL` |
| **No POSIX signals**  | Signals such as `SIGHUP`, `SIGPIPE`, and `SIGUSR` aren't available. `Ctrl+C` (`SIGINT`) works as expected. |
| **Path separators**   | Both `/` and `\` are accepted. Some utilities produce `\`-separated output, which can affect downstream piping. |
| **File permissions**  | Windows uses ACLs, not POSIX permission bits. Permission-based predicates (for example `find -perm`) may behave differently or be unavailable. |
| **Symbolic links**    | Reading existing symbolic links works without elevation. Creating new symbolic links requires Developer Mode ([**Settings > System > Advanced**](https://learn.microsoft.com/windows/advanced-settings)) or an elevated terminal. |

### PowerShell Command Parsing

The installer integrates itself with interactive PowerShell sessions via `PSReadLine`.
It ensures that quoted expression behave somewhat like they do under UNIX shells or CMD:
`echo *.txt` will then print a number of file names, while `echo '*.txt'` will print "*.txt" literally.

There are two shortcomings, however:
* PowerShell's escape character is still <code>\`</code>, not <code>\\</code><br>
  While you may write `find . \( -foo -bar \)` with Bash, you still need to write ``find . `( -foo -bar `)`` in PowerShell.
* `Get-Command ls`, `Get-Help ls`, etc., will still show `ls`, etc., as builtin commands<br>
  Due to limitations around `PSNativeCommandPreserveBytePipe` we cannot integrate ourselves in a more robust way with PowerShell.

### Not shipped

Commands available in source but not compiled:

| Command | Reason |
| ------- | ------ |
| `dd`, `shred`, `dircolors`, `sync`, `uname` | Limited usefulness on Windows |
| `chcon`, `chgrp`, `chmod`, `chown`, `chroot`, `groups` | POSIX permission concepts unavailable |
| `hostid`, `id`, `logname`, `pinky`, `who`, `users` | POSIX user/group concepts |
| `install` | Requires POSIX permission bits |
| `kill`, `nice`, `nohup`, `stdbuf` | No POSIX signals on Windows |
| `mkfifo`, `mknod` | Device/node concepts unavailable |
| `more`, `expand` | Conflict with built-in DOS commands |
| `runcon`, `stty`, `tty`, `pathchk` | POSIX-only terminal / path concepts |

<br/>

## License

该项目基于以下组件的上游仓库，均为宽松许可证：

| Component | License |
| --------- | ------- |
| [microsoft/coreutils](https://github.com/microsoft/coreutils) | MIT |
| [uutils/coreutils](https://github.com/uutils/coreutils) | MIT |
| [uutils/findutils](https://github.com/uutils/findutils) | MIT |
| [uutils/grep](https://github.com/uutils/grep) | MIT |
| [uutils/sed](https://github.com/uutils/sed) | MIT |
| [uutils/diffutils](https://github.com/uutils/diffutils) | MIT / Apache-2.0 |
| [uutils/tar](https://github.com/uutils/tar) | MIT |
