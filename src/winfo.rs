use std::ffi::OsString;

type BOOL = i32;
type DWORD = u32;
type LSTATUS = i32;
type HKEY = usize;
type LPCWSTR = *const u16;

const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize;
const RRF_RT_REG_SZ: DWORD = 0x0000_0002;
const ERROR_SUCCESS: LSTATUS = 0;

#[allow(non_snake_case)]
#[repr(C)]
struct RTL_OSVERSIONINFOW {
    dwOSVersionInfoSize: DWORD,
    dwMajorVersion: DWORD,
    dwMinorVersion: DWORD,
    dwBuildNumber: DWORD,
    dwPlatformId: DWORD,
    szCSDVersion: [u16; 128],
}

#[allow(non_snake_case)]
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

#[link(name = "ntdll")]
unsafe extern "system" {
    fn RtlGetVersion(lpVersionInformation: *mut RTL_OSVERSIONINFOW) -> LSTATUS;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetComputerNameExW(
        NameType: DWORD,
        lpBuffer: *mut u16,
        lpnSize: *mut DWORD,
    ) -> BOOL;
    fn GetUserNameW(lpBuffer: *mut u16, lpnSize: *mut DWORD) -> BOOL;
    fn GetTickCount64() -> u64;
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

#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegGetValueW(
        hkey: HKEY,
        lpSubKey: LPCWSTR,
        lpValue: LPCWSTR,
        dwFlags: DWORD,
        pdwType: *mut DWORD,
        pvData: *mut u8,
        pcbData: *mut DWORD,
    ) -> LSTATUS;
}

fn wstr(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn read_reg_string(subkey: &str, value: &str) -> Option<String> {
    let sk = wstr(subkey);
    let val = wstr(value);
    let mut buf = [0u16; 1024];
    let mut size = (buf.len() * 2) as DWORD;
    unsafe {
        let ret = RegGetValueW(
            HKEY_LOCAL_MACHINE,
            sk.as_ptr(),
            val.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut u8,
            &mut size,
        );
        if ret == ERROR_SUCCESS && size > 0 {
            let len = (size as usize / 2).saturating_sub(1);
            String::from_utf16(&buf[..len]).ok()
        } else {
            None
        }
    }
}

fn get_os_info() -> (String, u32, u32) {
    let mut ver = RTL_OSVERSIONINFOW {
        dwOSVersionInfoSize: size_of::<RTL_OSVERSIONINFOW>() as DWORD,
        dwMajorVersion: 0,
        dwMinorVersion: 0,
        dwBuildNumber: 0,
        dwPlatformId: 0,
        szCSDVersion: [0; 128],
    };
    unsafe { RtlGetVersion(&mut ver); }

    let product = read_reg_string(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
        "ProductName",
    ).unwrap_or_default();

    let display = read_reg_string(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
        "DisplayVersion",
    );

    let os_name = if product.contains("11") {
        let ver = display.as_deref().unwrap_or("");
        format!("Windows 11 {ver}")
    } else if product.contains("10") {
        let ver = display.as_deref().unwrap_or("");
        format!("Windows 10 {ver}")
    } else {
        product
    };

    (os_name, ver.dwBuildNumber, ver.dwMajorVersion)
}

fn get_hostname() -> String {
    let mut buf = [0u16; 260];
    let mut size = buf.len() as DWORD;
    unsafe {
        GetComputerNameExW(1, buf.as_mut_ptr(), &mut size);
    }
    let len = size as usize;
    String::from_utf16_lossy(&buf[..len])
}

fn get_username() -> String {
    let mut buf = [0u16; 260];
    let mut size = buf.len() as DWORD;
    unsafe { GetUserNameW(buf.as_mut_ptr(), &mut size); }
    let len = (size as usize).saturating_sub(1);
    String::from_utf16_lossy(&buf[..len])
}

fn get_uptime() -> (u64, u64, u64) {
    let ms = unsafe { GetTickCount64() };
    let days = ms / 86400_000;
    let hours = (ms % 86400_000) / 3600_000;
    let mins = (ms % 3600_000) / 60_000;
    (days, hours, mins)
}

fn get_cpu_name() -> String {
    read_reg_string(
        r"HARDWARE\DESCRIPTION\System\CentralProcessor\0",
        "ProcessorNameString",
    ).unwrap_or_default().trim().to_string()
}

fn get_shell() -> String {
    if let Ok(s) = std::env::var("SHELL") {
        return s;
    }
    if std::env::var("PSModulePath").is_ok() {
        "PowerShell".to_string()
    } else {
        "cmd.exe".to_string()
    }
}

fn get_memory() -> (u64, u64) {
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
    unsafe { GlobalMemoryStatusEx(&mut state); }
    (state.ullTotalPhys, state.ullAvailPhys)
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
            let dt = unsafe { GetDriveTypeW(path.as_ptr()) };
            if dt != 3 { continue; }
            let mut total = 0u64;
            let mut free = 0u64;
            unsafe { GetDiskFreeSpaceExW(path.as_ptr(), std::ptr::null_mut(), &mut total, &mut free); }
            if total > 0 {
                disks.push(DiskInfo { letter, total, free });
            }
        }
    }
    disks
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

fn print_usage() {
    eprintln!("Usage: winfo");
    eprintln!("Display system information.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --help, -h      Show this help");
}

const A: &str = "  ▄██████████████▄";
const B: &str = "  █              █";
const C: &str = "  █  ▄▄▄  ▄▄▄   █";
const D: &str = "  █  ▀▀▀  ▀▀▀   █";
const E: &str = "  ▀██████████████▀";

pub fn uumain(mut raw_args: impl Iterator<Item = OsString>) -> i32 {
    let _prog = raw_args.next();
    for arg in raw_args {
        if let Some(s) = arg.to_str() {
            if s == "--help" || s == "-h" {
                print_usage();
                return 0;
            }
            print_usage();
            return 1;
        }
    }

    #[allow(non_upper_case_globals)]
    const BOLD: &str = "\x1b[1m";
    #[allow(non_upper_case_globals)]
    const DIM: &str = "\x1b[2m";
    #[allow(non_upper_case_globals)]
    const RESET: &str = "\x1b[0m";

    let user = get_username();
    let host = get_hostname();
    let (os_name, build, nt_ver) = get_os_info();
    let (days, hours, mins) = get_uptime();
    let shell = get_shell();
    let cpu = get_cpu_name();
    let (total_phys, avail_phys) = get_memory();
    let used_phys = total_phys.saturating_sub(avail_phys);
    let disks = get_disks();

    println!();
    println!("{A}  {BOLD}{user}@{host}{RESET}");
    println!("{B}  {DIM}{dashes}{RESET}", dashes = "-".repeat(user.len() + host.len() + 1));
    println!("{C}  {BOLD}OS:{RESET}      {os_name} (build {build})");
    println!("{D}  {BOLD}Kernel:{RESET}  {nt_ver}.{build}");
    println!("{B}  {BOLD}Uptime:{RESET}  {days}d {hours}h {mins}m");
    println!("{C}  {BOLD}Shell:{RESET}   {shell}");
    println!("{D}  {BOLD}CPU:{RESET}     {cpu}");
    println!("{B}  {BOLD}Memory:{RESET}  {} / {}", format_size(used_phys), format_size(total_phys));

    if !disks.is_empty() {
        println!("{B}  {BOLD}Disks:{RESET}");
        for d in &disks {
            let used = d.total.saturating_sub(d.free);
            let pct = if d.total > 0 { used as f64 / d.total as f64 * 100.0 } else { 0.0 };
            println!("{C}           {}: {} / {}  ({:.0}%)", d.letter, format_size(used), format_size(d.total), pct);
        }
    }

    println!("{E}");
    println!();
    0
}
