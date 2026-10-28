#![allow(non_snake_case, non_camel_case_types)]

use std::ffi::OsString;
use std::io::Write;

type BOOL = i32;
type DWORD = u32;

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
}

struct Mem {
    total_phys: u64,
    avail_phys: u64,
    total_page: u64,
    avail_page: u64,
}

fn read_mem() -> Option<Mem> {
    let mut s = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as DWORD,
        dwMemoryLoad: 0,
        ullTotalPhys: 0,
        ullAvailPhys: 0,
        ullTotalPageFile: 0,
        ullAvailPageFile: 0,
        ullTotalVirtual: 0,
        ullAvailVirtual: 0,
        ullAvailExtendedVirtual: 0,
    };
    if unsafe { GlobalMemoryStatusEx(&mut s) } == 0 {
        return None;
    }
    Some(Mem {
        total_phys: s.ullTotalPhys,
        avail_phys: s.ullAvailPhys,
        total_page: s.ullTotalPageFile,
        avail_page: s.ullAvailPageFile,
    })
}

const HELP: &str = "\
Usage: free [option]...
Display the amount of free and used physical and swap memory.

  -b, --bytes          output in bytes
  -k, --kibi           output in KiB (default)
  -m, --mebi           output in MiB
  -g, --gibi           output in GiB
      --tebi           output in TiB
      --pebi           output in PiB
  -h, --human          output in human-readable form (auto-scaled, base 1024)
      --si             use powers of 1000, not 1024
  -t, --total          display a line showing total memory
  -V, --version        output version information and exit
      --help           display this help and exit

Note: on Windows there is no equivalent of the Linux 'shared' and
'buff/cache' fields, so they are reported as 0. 'Swap' maps to the
committed pagefile beyond physical memory.
";

struct Config {
    divisor: u64,
    human: bool,
    human_base: u64,
    total: bool,
}

fn fmt_val(bytes: u64, cfg: &Config) -> String {
    if cfg.human {
        let base = cfg.human_base as f64;
        let units: &[&str] = if cfg.human_base == 1000 {
            &["B", "kB", "MB", "GB", "TB", "PB"]
        } else {
            &["B", "KiB", "MiB", "GiB", "TiB", "PiB"]
        };
        let mut v = bytes as f64;
        let mut idx = 0usize;
        while v >= base && idx + 1 < units.len() {
            v /= base;
            idx += 1;
        }
        if idx == 0 {
            format!("{:.0}{}", v, units[0])
        } else {
            format!("{:.1}{}", v, units[idx])
        }
    } else {
        (bytes / cfg.divisor).to_string()
    }
}

pub fn uumain(raw_args: impl Iterator<Item = OsString>) -> i32 {
    let mut cfg = Config { divisor: 1024, human: false, human_base: 1024, total: false };
    let mut si = false;

    let args: Vec<String> = raw_args.map(|s| s.to_string_lossy().into_owned()).collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-b" | "--bytes" => cfg.divisor = 1,
            "-k" | "--kibi" => cfg.divisor = 1024,
            "-m" | "--mebi" => cfg.divisor = 1024 * 1024,
            "-g" | "--gibi" => cfg.divisor = 1024 * 1024 * 1024,
            "--tebi" => cfg.divisor = 1024u64.pow(4),
            "--pebi" => cfg.divisor = 1024u64.pow(5),
            "-h" | "--human" => cfg.human = true,
            "--si" => si = true,
            "-t" | "--total" => cfg.total = true,
            "-V" | "--version" => {
                println!("free (coreutils) 1.0");
                return 0;
            }
            "--help" => {
                print!("{HELP}");
                return 0;
            }
            other => {
                // Bundled short flags like -th
                if other.len() > 2 && other.starts_with('-') && !other.starts_with("--") {
                    for c in other[1..].chars() {
                        match c {
                            'b' => cfg.divisor = 1,
                            'k' => cfg.divisor = 1024,
                            'm' => cfg.divisor = 1024 * 1024,
                            'g' => cfg.divisor = 1024 * 1024 * 1024,
                            'h' => cfg.human = true,
                            't' => cfg.total = true,
                            's' | 'c' | 'l' | 'w' | 'o' => {} // loop/legacy: accepted, ignored
                            _ => {
                                eprintln!("free: unrecognized option -- '-{c}'");
                                return 1;
                            }
                        }
                    }
                } else {
                    eprintln!("free: unrecognized option -- '{other}'");
                    eprint!("{HELP}");
                    return 1;
                }
            }
        }
        i += 1;
    }

    if si {
        let table: [(u64, u64); 7] = [
            (1, 1),
            (1024, 1000),
            (1024 * 1024, 1000 * 1000),
            (1024u64.pow(3), 1000u64.pow(3)),
            (1024u64.pow(4), 1000u64.pow(4)),
            (1024u64.pow(5), 1000u64.pow(5)),
            (1024u64.pow(6), 1000u64.pow(6)),
        ];
        if let Some(&(_, si_v)) = table.iter().find(|(b, _)| *b == cfg.divisor) {
            cfg.divisor = si_v;
        }
        cfg.human_base = 1000;
    }

    let mem = match read_mem() {
        Some(m) => m,
        None => {
            eprintln!("free: failed to read memory statistics");
            return 1;
        }
    };

    let phys_total = mem.total_phys;
    let phys_avail = mem.avail_phys;
    let phys_used = phys_total.saturating_sub(phys_avail);

    let swap_total = mem.total_page.saturating_sub(phys_total);
    let swap_free = mem.avail_page.saturating_sub(phys_avail);
    let swap_used = swap_total.saturating_sub(swap_free);

    let out = std::io::stdout();
    let mut w = out.lock();

    let unit_hdr = if cfg.human {
        String::new()
    } else {
        let u = match cfg.divisor {
            1 => "bytes",
            1000 => "kB",
            1024 => "KiB",
            1000000 => "MB",
            1048576 => "MiB",
            1000000000 => "GB",
            1073741824 => "GiB",
            1099511627776 => "TiB",
            1125899906842624 => "PiB",
            _ => "",
        };
        format!("[{u}]")
    };

    let w_col = if cfg.human { 12 } else { 13 };
    let field = |bytes: u64| format!("{:>w$}", fmt_val(bytes, &cfg), w = w_col);
    let row = |label: &str, vals: [u64; 6]| {
        let mut s = format!("{:<6}", label);
        for v in vals {
            s.push_str(&field(v));
        }
        s.push('\n');
        s
    };

    let mut hdr = format!("{:<6}", "");
    for name in ["total", "used", "free", "shared", "buff/cache", "available"] {
        hdr.push_str(&format!("{:>w$}", name, w = w_col));
    }
    if !unit_hdr.is_empty() {
        hdr.push(' ');
        hdr.push_str(&unit_hdr);
    }
    hdr.push('\n');

    let _ = w.write_all(hdr.as_bytes());
    let _ = w.write_all(
        row("Mem:", [phys_total, phys_used, phys_avail, 0, 0, phys_avail]).as_bytes(),
    );
    let _ = w.write_all(row("Swap:", [swap_total, swap_used, swap_free, 0, 0, swap_free]).as_bytes());
    if cfg.total {
        let _ = w.write_all(
            row(
                "Total:",
                [
                    phys_total + swap_total,
                    phys_used + swap_used,
                    phys_avail + swap_free,
                    0,
                    0,
                    phys_avail + swap_free,
                ],
            )
            .as_bytes(),
        );
    }
    let _ = w.flush();
    0
}
