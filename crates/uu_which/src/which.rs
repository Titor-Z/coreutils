use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::Path;

use clap::{Arg, ArgAction, Command};

use uucore::error::UResult;
use uucore::{format_usage, show_error, translate};

static OPT_ALL: &str = "all";

static ARG_COMMANDS: &str = "commands";

fn get_pathext_extensions() -> Vec<String> {
    let empty = OsStr::new("");
    let pathext = env::var_os("PATHEXT").unwrap_or_else(|| empty.to_os_string());
    env::split_paths(&pathext)
        .filter_map(|p| p.to_str().map(|s| s.to_lowercase()))
        .collect()
}

fn search_in_path(command: &str, pathext: &[String]) -> Option<String> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let base = dir.join(command);
        if base.is_file() {
            if let Ok(canon) = fs::canonicalize(&base) {
                return canon.to_str().map(|s| s.to_string());
            }
        }
        for ext in pathext {
            let candidate = base.with_extension(ext.trim_start_matches('.'));
            if candidate.is_file() {
                if let Ok(canon) = fs::canonicalize(&candidate) {
                    return canon.to_str().map(|s| s.to_string());
                }
            }
        }
    }
    None
}

fn search_all_in_path(command: &str, pathext: &[String]) -> Vec<String> {
    let mut results = Vec::new();
    let Some(path) = env::var_os("PATH") else {
        return results;
    };
    for dir in env::split_paths(&path) {
        let base = dir.join(command);
        if base.is_file() {
            if let Ok(canon) = fs::canonicalize(&base) {
                if let Some(s) = canon.to_str().map(|s| s.to_string()) {
                    if !results.contains(&s) {
                        results.push(s);
                    }
                }
            }
        }
        for ext in pathext {
            let candidate = base.with_extension(ext.trim_start_matches('.'));
            if candidate.is_file() {
                if let Ok(canon) = fs::canonicalize(&candidate) {
                    if let Some(s) = canon.to_str().map(|s| s.to_string()) {
                        if !results.contains(&s) {
                            results.push(s);
                        }
                    }
                }
            }
        }
    }
    results
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uucore::clap_localization::handle_clap_result_with_exit_code(uu_app(), args, 2)?;

    let all = matches.get_flag(OPT_ALL);
    let commands: Vec<String> = matches
        .get_many::<String>(ARG_COMMANDS)
        .map(|v| v.map(ToString::to_string).collect())
        .unwrap_or_default();

    if commands.is_empty() {
        show_error!("no arguments specified");
        return Err(1.into());
    }

    let pathext = get_pathext_extensions();
    let mut not_found = false;
    let mut stdout = std::io::stdout().lock();

    for cmd in &commands {
        let path = Path::new(cmd);
        if path.is_absolute() || cmd.contains('\\') || cmd.contains('/') {
            if path.is_file() {
                if let Ok(canon) = fs::canonicalize(path) {
                    writeln!(stdout, "{}", canon.display()).ok();
                }
            } else {
                not_found = true;
            }
            continue;
        }

        if all {
            let results = search_all_in_path(cmd, &pathext);
            if results.is_empty() {
                not_found = true;
            } else {
                for r in &results {
                    writeln!(stdout, "{r}").ok();
                }
            }
        } else if let Some(result) = search_in_path(cmd, &pathext) {
            writeln!(stdout, "{result}").ok();
        } else {
            not_found = true;
        }
    }

    if not_found {
        Err(1.into())
    } else {
        Ok(())
    }
}

pub fn uu_app() -> Command {
    let cmd = Command::new("which")
        .version(uucore::crate_version!())
        .about(translate!("which-about"))
        .override_usage(format_usage(&translate!("which-usage")))
        .infer_long_args(true);
    uucore::clap_localization::configure_localized_command(cmd)
        .arg(
            Arg::new(OPT_ALL)
                .short('a')
                .long(OPT_ALL)
                .help(translate!("which-help-all"))
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(ARG_COMMANDS)
                .action(ArgAction::Append)
                .required(true)
                .num_args(1..),
        )
}
