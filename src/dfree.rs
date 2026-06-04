use std::ffi::OsString;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

type BOOL = i32;
type DWORD = u32;
type LPCWSTR = *const u16;

const DRIVE_FIXED: DWORD = 3;

#[allow(non_snake_case, dead_code)]
#[repr(C)]
struct MEMORYSTATUSEX {
    dwLength: DWORD,
    dwMemoryLoad: DWORD,
    ullTotalPhys: u64,
    ullAvailPhys: u64,
    ullTotalPageFile: u64,
    ullAvailPageFile: u64,
    ullTotalVirtual: u64,
    ullAvailVirtual: u64,
    ullAvailExtendedVirtual: u64,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> BOOL;
    fn GetDiskFreeSpaceExW(
        lpDirectoryName: LPCWSTR,
        lpFreeBytesAvailableToCaller: *mut u64,
        lpTotalNumberOfBytes: *mut u64,
        lpTotalNumberOfFreeBytes: *mut u64,
    ) -> BOOL;
    fn GetLogicalDrives() -> DWORD;
    fn GetDriveTypeW(lpRootPathName: LPCWSTR) -> DWORD;
}

fn get_memory() -> (u64, u64, u64, u64, u32) {
    let mut state = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as DWORD,
        dwMemoryLoad: 0,
        ullTotalPhys: 0,
        ullAvailPhys: 0,
        ullTotalPageFile: 0,
        ullAvailPageFile: 0,
        ullTotalVirtual: 0,
        ullAvailVirtual: 0,
        ullAvailExtendedVirtual: 0,
    };
    unsafe {
        GlobalMemoryStatusEx(&mut state);
    }
    (
        state.ullTotalPhys,
        state.ullAvailPhys,
        state.ullTotalPageFile,
        state.ullAvailPageFile,
        state.dwMemoryLoad,
    )
}

struct DiskInfo {
    letter: char,
    total: u64,
    free: u64,
}

fn get_disks() -> Vec<DiskInfo> {
    let drives = unsafe { GetLogicalDrives() };
    let mut disks = Vec::new();
    for i in 0..26 {
        if drives & (1 << i) != 0 {
            let letter = (b'A' + i) as char;
            let path = vec![letter as u16, b':' as u16, b'\\' as u16, 0];

            let drive_type = unsafe { GetDriveTypeW(path.as_ptr()) };
            if drive_type != DRIVE_FIXED {
                continue;
            }

            let mut total = 0u64;
            let mut free = 0u64;
            unsafe {
                GetDiskFreeSpaceExW(path.as_ptr(), std::ptr::null_mut(), &mut total, &mut free);
            }
            if total > 0 {
                disks.push(DiskInfo { letter, total, free });
            }
        }
    }
    disks
}

fn format_size_no_align(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

fn progress_bar(used: u64, total: u64, width: usize) -> String {
    if total == 0 {
        return "░".repeat(width);
    }
    let ratio = used as f64 / total as f64;
    let filled = (ratio * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn color_for_usage(ratio: f64) -> &'static str {
    if ratio > 0.90 {
        "\x1b[91m" // bright red
    } else if ratio > 0.75 {
        "\x1b[93m" // bright yellow
    } else {
        "\x1b[92m" // bright green
    }
}

const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[96m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const WHITE: &str = "\x1b[97m";
const BAR_WIDTH: usize = 25;

fn print_usage() {
    eprintln!("Usage: dfree [-n seconds]");
    eprintln!("Display real-time memory and disk usage.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -n <seconds>    Refresh interval (default: 3)");
    eprintln!("  --help, -h      Show this help");
}

fn draw_frame(label: &str, used: u64, total: u64, pct: f64, bar_color: &str) {
    let bar = progress_bar(used, total, BAR_WIDTH);
    let used_s = format_size_no_align(used);
    let total_s = format_size_no_align(total);

    print!(
        "  {BOLD}{label}{RESET} {bar_color}{bar}{RESET}  {WHITE}{used_s}{RESET} / {total_s}  \
         {bar_color}{pct:>5.1}%{RESET}\n"
    );
}

pub fn uumain(mut raw_args: impl Iterator<Item = OsString>) -> i32 {
    let mut interval = 3u64;

    let _prog = raw_args.next();

    let args: Vec<OsString> = raw_args.collect();
    let mut i = 0;
    while i < args.len() {
        if let Some(s) = args[i].to_str() {
            match s {
                "-n" => {
                    i += 1;
                    if i < args.len() {
                        if let Some(val) = args[i].to_str() {
                            interval = val.parse().unwrap_or(3);
                        }
                    }
                }
                "--help" | "-h" => {
                    print_usage();
                    return 0;
                }
                _ => {
                    print_usage();
                    return 1;
                }
            }
        }
        i += 1;
    }

    let sleep_dur = Duration::from_secs(interval);

    loop {
        print!("\x1b[?25l\x1b[2J\x1b[H");

        let (total_phys, avail_phys, total_page, avail_page, _load) = get_memory();
        let used_phys = total_phys.saturating_sub(avail_phys);
        let used_page = total_page.saturating_sub(avail_page);
        let mem_pct = if total_phys > 0 {
            used_phys as f64 / total_phys as f64 * 100.0
        } else {
            0.0
        };
        let swap_pct = if total_page > 0 {
            used_page as f64 / total_page as f64 * 100.0
        } else {
            0.0
        };

        let mem_color = color_for_usage(mem_pct / 100.0);
        let swap_color = color_for_usage(swap_pct / 100.0);

        println!(
            " {CYAN}{BOLD}┌─────────────────────────────────────────────┐{RESET}"
        );
        println!(
            " {CYAN}{BOLD}│            {WHITE}dfree v1  内存 & 磁盘监控{CYAN}            │{RESET}"
        );
        println!(
            " {CYAN}{BOLD}└─────────────────────────────────────────────┘{RESET}"
        );
        println!();

        draw_frame(" Memory ", used_phys, total_phys, mem_pct, mem_color);
        draw_frame(" Swap   ", used_page, total_page, swap_pct, swap_color);

        println!();
        println!("  {DIM}── Disks ──────────────────────────────────{RESET}");

        let disks = get_disks();
        let disk_colors = ["\x1b[94m", "\x1b[95m", "\x1b[96m", "\x1b[93m"];
        for (idx, disk) in disks.iter().enumerate() {
            let used = disk.total.saturating_sub(disk.free);
            let pct = if disk.total > 0 {
                used as f64 / disk.total as f64 * 100.0
            } else {
                0.0
            };
            let color = disk_colors[idx % disk_colors.len()];
            draw_frame(&format!(" {}: ", disk.letter), used, disk.total, pct, color);
        }

        println!();
        println!(
            " {DIM}──────────────────────────────────────────────────{RESET}"
        );
        println!(
            "  Refresh: {}{}s{}   Ctrl+C to exit",
            WHITE, interval, RESET
        );

        io::stdout().flush().ok();
        thread::sleep(sleep_dur);
    }
}
