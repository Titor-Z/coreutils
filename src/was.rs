// was — Windows Alias System
//
// 把 PowerShell $PROFILE 里的一段标记区域当作别名数据库来维护：
//
//   # >>> was managed <<<
//   function global:ll { & ls -l $args }
//   # <<< was managed >>>
//
// 标记区域之外的内容原样保留，所以可以和安全地和其他 $PROFILE 内容共存。
// `was` 负责列出/查询/写入，`unwas` 负责删除；两者共用同一份 ProfileManager。

use std::ffi::OsString;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

// ============================================================
// 常量
// ============================================================

/// $PROFILE 中标记区域起始行
const MANAGED_START: &str = "# >>> was managed <<<";
/// $PROFILE 中标记区域结束行
const MANAGED_END: &str = "# <<< was managed >>>";

// ============================================================
// Alias — 单个别名对象
// ============================================================

struct Alias {
    name: String,
    command: String,
}

impl Alias {
    /// 从一行函数定义中解析别名
    /// 格式：`function global:<name> { & <command> $args }`
    fn parse_line(line: &str) -> Option<Alias> {
        let line = line.trim();
        let prefix = "function global:";
        if !line.starts_with(prefix) {
            return None;
        }
        let body = &line[prefix.len()..];
        if let Some(split) = body.find(" { & ") {
            let name = &body[..split];
            let cmd_start = split + 5;
            let suffix = " $args }";
            if body.ends_with(suffix) {
                let cmd_end = cmd_start + (body.len() - suffix.len() - split - 5);
                let command = &body[cmd_start..cmd_end];
                if is_valid_name(name) {
                    return Some(Alias {
                        name: name.to_string(),
                        command: command.to_string(),
                    });
                }
            }
        }
        None
    }

    /// 格式化为 PowerShell 函数定义行
    fn format(&self) -> String {
        format!("function global:{} {{ & {} $args }}", self.name, self.command)
    }

    /// 格式化为 human-readable 显示
    fn display(&self) -> String {
        format!("{} = {}", self.name, self.command)
    }
}

/// 别名只能是字母、数字和下划线：其它字符在 PowerShell 函数名里不可靠
fn is_valid_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

// ============================================================
// ProfileManager — 管理 $PROFILE 文件中的别名标记区域
// ============================================================

struct ProfileManager {
    path: PathBuf,
}

impl ProfileManager {
    /// 获取默认的 $PROFILE 路径（pwsh 7）
    fn default_path() -> Result<PathBuf, String> {
        let home = home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
        Ok(home
            .join("Documents")
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"))
    }

    /// 读取 $PROFILE 文件全部内容
    fn read_raw(&self) -> io::Result<Vec<String>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(&self.path)?;
        io::BufReader::new(file).lines().collect()
    }

    /// 从文件内容中解析标记区域内的别名
    fn parse_aliases(&self, lines: &[String]) -> Vec<Alias> {
        let mut in_section = false;
        let mut aliases = Vec::new();
        for line in lines {
            let trimmed = line.trim();
            if trimmed == MANAGED_START {
                in_section = true;
                continue;
            }
            if trimmed == MANAGED_END {
                break;
            }
            if in_section {
                if let Some(alias) = Alias::parse_line(line) {
                    aliases.push(alias);
                }
            }
        }
        aliases
    }

    /// 保存别名列表到 $PROFILE，标记区域外内容保持不变
    fn save(&self, aliases: &[Alias]) -> io::Result<()> {
        let original = self.read_raw()?;
        let mut output: Vec<String> = Vec::new();
        let mut in_section = false;
        let mut section_written = false;

        for line in &original {
            let trimmed = line.trim();
            if trimmed == MANAGED_START {
                in_section = true;
                output.push(MANAGED_START.to_string());
                for alias in aliases {
                    output.push(alias.format());
                }
                section_written = true;
                continue;
            }
            if trimmed == MANAGED_END {
                in_section = false;
                output.push(MANAGED_END.to_string());
                continue;
            }
            if !in_section {
                output.push(line.clone());
            }
        }

        if !section_written {
            if let Some(last) = output.last() {
                if !last.is_empty() {
                    output.push(String::new());
                }
            }
            output.push(MANAGED_START.to_string());
            for alias in aliases {
                output.push(alias.format());
            }
            output.push(MANAGED_END.to_string());
        }

        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut file = fs::File::create(&self.path)?;
        for line in &output {
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }
}

/// 用户主目录。Windows 上 `USERPROFILE` 通常可用，退回 `HOMEDRIVE` + `HOMEPATH`。
fn home_dir() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("USERPROFILE") {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    let drive = std::env::var_os("HOMEDRIVE")?;
    let path = std::env::var_os("HOMEPATH")?;
    let mut joined = drive;
    joined.push(path);
    Some(PathBuf::from(joined))
}

// ============================================================
// 入口
// ============================================================

pub fn was_uumain(raw_args: impl Iterator<Item = OsString>) -> i32 {
    run(raw_args, false)
}

pub fn unwas_uumain(raw_args: impl Iterator<Item = OsString>) -> i32 {
    run(raw_args, true)
}

fn run(raw_args: impl Iterator<Item = OsString>, unwas: bool) -> i32 {
    // 第一个参数是程序名（多调用分发时可能是 was.exe / unwas.exe）
    let args: Vec<String> = raw_args.skip(1).map(|s| s.to_string_lossy().into_owned()).collect();
    if unwas {
        run_unwas(&args)
    } else {
        run_was(&args)
    }
}

// ============================================================
// was
// ============================================================

enum WasCmd {
    List,
    Get(String),
    Set(String, String),
    Help,
    Version,
}

/// 解析 was 的命令行参数
/// 一旦遇到 name=，后续所有参数（即使是 -xxx）都视为命令值
fn parse_was_args(args: &[String]) -> Result<WasCmd, String> {
    let mut i = 0;
    let mut set_name: Option<String> = None;
    let mut command_parts: Vec<String> = Vec::new();

    while i < args.len() {
        let arg = &args[i];

        match &set_name {
            None => match arg.as_str() {
                "--help" | "-?" => return Ok(WasCmd::Help),
                "--version" | "-V" => return Ok(WasCmd::Version),
                _ if arg.starts_with('-') => return Err(format!("未知选项：{}", arg)),
                _ => {
                    if let Some((n, v)) = arg.split_once('=') {
                        if n.is_empty() {
                            return Err("别名名称不能为空".to_string());
                        }
                        set_name = Some(n.to_string());
                        if !v.is_empty() {
                            command_parts.push(v.to_string());
                        }
                    } else {
                        // 没有 =，必须是查询模式，且不能有多余参数
                        if i + 1 < args.len() {
                            return Err("参数过多".to_string());
                        }
                        return Ok(WasCmd::Get(arg.to_string()));
                    }
                }
            },
            // 已进入设置模式，所有剩余参数都是命令值的一部分
            Some(_) => command_parts.push(arg.clone()),
        }
        i += 1;
    }

    match set_name {
        None => Ok(WasCmd::List),
        Some(name) => {
            if command_parts.is_empty() {
                return Err("删除别名请使用 `unwas <name>`".to_string());
            }
            Ok(WasCmd::Set(name, command_parts.join(" ")))
        }
    }
}

fn print_was_help() {
    println!("用法：was [选项] [<name>] [<name>=<command>]");
    println!();
    println!("  was                    列出所有别名");
    println!("  was <name>            查看指定别名");
    println!("  was <name>=<command>  创建或更新别名");
    println!("  -?, --help            显示此帮助");
    println!("  -V, --version         显示版本号");
}

fn run_was(args: &[String]) -> i32 {
    let cmd = match parse_was_args(args) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("was: {msg}");
            return 1;
        }
    };

    match cmd {
        WasCmd::Help => {
            print_was_help();
            return 0;
        }
        WasCmd::Version => {
            println!("was v{}", env!("CARGO_PKG_VERSION"));
            return 0;
        }
        _ => {}
    }

    let manager = match ProfileManager::default_path() {
        Ok(p) => ProfileManager { path: p },
        Err(e) => {
            eprintln!("was: {e}");
            return 1;
        }
    };

    let lines = match manager.read_raw() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("was: 无法读取 $PROFILE：{e}");
            return 1;
        }
    };
    let mut aliases = manager.parse_aliases(&lines);

    match cmd {
        WasCmd::List => {
            if aliases.is_empty() {
                println!("（没有已定义的别名）");
            } else {
                for alias in &aliases {
                    println!("{}", alias.display());
                }
            }
            0
        }
        WasCmd::Get(name) => {
            if !is_valid_name(&name) {
                eprintln!("was: 别名名称只能包含字母、数字和下划线");
                return 1;
            }
            match aliases.iter().find(|a| a.name == name) {
                Some(alias) => {
                    println!("{}", alias.display());
                    0
                }
                None => {
                    eprintln!("was: 别名 '{name}' 未定义");
                    1
                }
            }
        }
        WasCmd::Set(name, command) => {
            if !is_valid_name(&name) {
                eprintln!("was: 别名名称只能包含字母、数字和下划线");
                return 1;
            }
            if let Some(existing) = aliases.iter_mut().find(|a| a.name == name) {
                existing.command = command.clone();
            } else {
                aliases.push(Alias {
                    name: name.clone(),
                    command: command.clone(),
                });
            }
            if let Err(e) = manager.save(&aliases) {
                eprintln!("was: 写入 $PROFILE 失败：{e}");
                return 1;
            }
            println!("已设置别名：{} = {}", name, command);
            println!("已保存到：{}", manager.path.display());
            0
        }
        WasCmd::Help | WasCmd::Version => 0,
    }
}

// ============================================================
// unwas
// ============================================================

fn print_unwas_help() {
    println!("用法：unwas [选项] <name>");
    println!();
    println!("  unwas <name>  删除指定别名");
    println!("  -?, --help    显示此帮助");
    println!("  -V, --version 显示版本号");
}

fn run_unwas(args: &[String]) -> i32 {
    let mut positional: Vec<&str> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--help" | "-?" => {
                print_unwas_help();
                return 0;
            }
            "--version" | "-V" => {
                println!("unwas v{}", env!("CARGO_PKG_VERSION"));
                return 0;
            }
            _ if arg.starts_with('-') => {
                eprintln!("unwas: 未知选项：{arg}");
                return 1;
            }
            _ => positional.push(arg.as_str()),
        }
    }

    let name = match positional.len() {
        0 => {
            eprintln!("unwas: 用法：unwas <name>");
            return 1;
        }
        1 => {
            if positional[0].contains('=') {
                eprintln!("unwas: 设置别名请使用 `was <name>=<command>`");
                return 1;
            }
            positional[0].to_string()
        }
        _ => {
            eprintln!("unwas: 参数过多");
            return 1;
        }
    };

    if !is_valid_name(&name) {
        eprintln!("unwas: 别名名称只能包含字母、数字和下划线");
        return 1;
    }

    let manager = match ProfileManager::default_path() {
        Ok(p) => ProfileManager { path: p },
        Err(e) => {
            eprintln!("unwas: {e}");
            return 1;
        }
    };

    let lines = match manager.read_raw() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("unwas: 无法读取 $PROFILE：{e}");
            return 1;
        }
    };
    let mut aliases = manager.parse_aliases(&lines);
    let before = aliases.len();
    aliases.retain(|a| a.name != name);

    if aliases.len() == before {
        eprintln!("unwas: 别名 '{name}' 未定义");
        return 1;
    }

    if let Err(e) = manager.save(&aliases) {
        eprintln!("unwas: 写入 $PROFILE 失败：{e}");
        return 1;
    }
    println!("已删除别名：{name}");
    println!("已保存到：{}", manager.path.display());
    0
}
