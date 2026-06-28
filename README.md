<h1 align="center">coreutils (Windows 增强版)</h1>

<p align="center">UNIX 风格命令工具集，原生 Windows 可执行文件。</p>

<h3 align="center">
  <a href="#install">Install</a>
  <span> · </span>
  <a href="#usage">Usage</a>
  <span> · </span>
  <a href="#added-commands">Added commands</a>
  <span> · </span>
  <a href="#build">Build</a>
  <span> · </span>
  <a href="#windows-caveats">Windows caveats</a>
</h3>

<p align="center">
  <a href="https://github.com/Titor-Z/coreutils/releases"><img src="https://img.shields.io/github/v/release/Titor-Z/coreutils" alt="Latest release"></a>
  <a href="https://github.com/Titor-Z/coreutils/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/Titor-Z/coreutils/release.yml" alt="Build status"></a>
</p>

---

Fork of [microsoft/coreutils](https://github.com/microsoft/coreutils) 基础上扩展，
集成了 [uutils/findutils](https://github.com/uutils/findutils)、[uutils/grep](https://github.com/uutils/grep)、
[uutils/sed](https://github.com/uutils/sed)、[uutils/diffutils](https://github.com/uutils/diffutils)、
以及 [uutils/tar](https://github.com/uutils/tar)，打包成一个多调用二进制文件。在 Windows 上原生运行，
无需 WSL / Cygwin / MSYS2。

共 **88 个命令**（76 `[microsoft]` + 7 `[uutils]` + 5 `[custom]`），覆盖日常 Linux 命令行的绝大部分需求。

<br/>

## Install

从 [GitHub Releases](https://github.com/Titor-Z/coreutils/releases) 下载最新安装包 `coreutils-Titor-Z.exe`（Inno Setup 安装器），安装后自动加入 PATH。

也可以直接下载 `coreutils.exe` 放到任意目录使用。

可用 `coreutils.exe --list` 查看所有命令，`coreutils.exe --list-raw` 供安装程序/脚本使用。

## Usage

```bash
coreutils.exe <command> [args...]    # 直接使用
coreutils.exe --list                  # 列出所有可用命令（含分类标签）
coreutils.exe --list-raw              # 仅命令名，每行一个，供脚本使用
```

安装器会自动创建硬链接，安装后可直接当独立命令使用：

```bash
sed --version
diff -u a.txt b.txt
dfree
```

<br/>

## Added commands

该 fork 在 Microsoft 原版 76 个命令的基础上增加了以下命令：

| Command | Source | Description |
| ------- | ------ | ----------- |
| `find` | [uutils/findutils](https://github.com/uutils/findutils) | 文件搜索，支持 `-name`、`-type`、`-exec` 等标准功能 |
| `grep` | [uutils/grep](https://github.com/uutils/grep) | 文本搜索，支持 `-r`、`-i`、`-E`（ERE）、`-v` 等 |
| `sed` | [uutils/sed](https://github.com/uutils/sed) | 流编辑器，支持 `s/pattern/replace/`、`-n`、地址范围等 |
| `diff` | [uutils/diffutils](https://github.com/uutils/diffutils) | 文件差异比较，支持 unified diff (`-u`) |
| `cmp` | [uutils/diffutils](https://github.com/uutils/diffutils) | 字节级文件比较 |
| `tar` | [uutils/tar](https://github.com/uutils/tar) | 归档工具，支持 create / list / extract |
| `dfree` | 自定义 | TUI 磁盘分析器（ratatui），支持磁盘分组、文件分类统计、大文件浏览、toml 可配置 |
| `winfo` | 自定义 | 系统信息显示（neofetch-like），显示 OS、BIOS、CPU、内存、NIC 等信息 |
| `wtop` | 自定义 | 进程监视器 TUI（ratatui），实时显示进程列表、CPU/内存占用 |
| `which` | 自定义 | PATH 查找，支持 PATHEXT 和 `-a` 参数 |
| `la` | 别名 | 等价于 `ls -A` |

### dfree（TUI 磁盘分析器）

基于 ratatui 的全屏 TUI 工具，支持：
- **磁盘分组**：按物理磁盘分组，展开查看分区详情
- **文件分类**：统计各目录下按类型（文档、图片、视频、代码等）分布
- **大文件浏览**：按大小排序，快速定位占用空间最多的文件
- **toml 配置**：通过 `dfree.toml` 自定义扫描路径和文件类型规则

```bash
dfree                    # 启动 TUI
dfree --help             # 查看所有选项
```

### winfo（系统信息）

类似 neofetch 的系统信息展示，显示：
- OS 版本、内核、运行时间
- BIOS 厂商/版本/日期
- CPU 型号、核心数、频率
- 内存容量、频率
- 网络适配器信息

```bash
winfo
```

### wtop（进程监视器 TUI）

基于 ratatui 的实时进程监视器，类似 htop：

```bash
wtop                     # 启动 TUI
```

### which（PATH 查找）

查找可执行文件在 PATH 中的位置：

```bash
which git                # 查找 git
which -a node            # 列出所有匹配
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
| `which`    |  ✅  |       ⚠️        | PowerShell has `which` as alias for `Get-Command` |
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
