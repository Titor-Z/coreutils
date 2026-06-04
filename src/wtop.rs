use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};

use ratatui::{
    DefaultTerminal,
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, Paragraph, Row, Table, TableState},
};

type HANDLE = isize;
type BOOL = i32;
type DWORD = u32;

const TH32CS_SNAPPROCESS: DWORD = 0x00000002;
const PROCESS_QUERY_LIMITED_INFORMATION: DWORD = 0x1000;
const PROCESS_TERMINATE: DWORD = 0x0001;
const INVALID_HANDLE_VALUE: isize = -1;

#[allow(non_snake_case)]
#[repr(C)]
struct FILETIME {
    dwLowDateTime: DWORD,
    dwHighDateTime: DWORD,
}

#[allow(non_snake_case)]
#[repr(C)]
struct PROCESSENTRY32W {
    dwSize: DWORD,
    cntUsage: DWORD,
    th32ProcessID: DWORD,
    th32DefaultHeapID: usize,
    th32ModuleID: DWORD,
    cntThreads: DWORD,
    th32ParentProcessID: DWORD,
    pcPriClassBase: i32,
    dwFlags: DWORD,
    szExeFile: [u16; 260],
}

#[allow(non_snake_case)]
#[repr(C)]
struct SYSTEM_INFO {
    wProcessorArchitecture: u16,
    wReserved: u16,
    dwPageSize: DWORD,
    lpMinimumApplicationAddress: *mut u8,
    lpMaximumApplicationAddress: *mut u8,
    dwActiveProcessorMask: usize,
    dwNumberOfProcessors: DWORD,
    dwProcessorType: DWORD,
    dwAllocationGranularity: DWORD,
    wProcessorLevel: u16,
    wProcessorRevision: u16,
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

#[allow(non_snake_case)]
#[repr(C)]
struct PROCESS_MEMORY_COUNTERS {
    cb: DWORD,
    PageFaultCount: DWORD,
    PeakWorkingSetSize: usize,
    WorkingSetSize: usize,
    QuotaPeakPagedPoolUsage: usize,
    QuotaPagedPoolUsage: usize,
    QuotaPeakNonPagedPoolUsage: usize,
    QuotaNonPagedPoolUsage: usize,
    _pad: usize,
    PagefileUsage: usize,
    PeakPagefileUsage: usize,
}

#[allow(non_snake_case)]
#[repr(C)]
struct IO_COUNTERS {
    ReadOperationCount: u64,
    WriteOperationCount: u64,
    OtherOperationCount: u64,
    ReadTransferCount: u64,
    WriteTransferCount: u64,
    OtherTransferCount: u64,
}

#[allow(dead_code)]
#[allow(non_snake_case)]
#[repr(C)]
struct MIB_TCPROW_OWNER_PID {
    dwState: u32,
    dwLocalAddr: u32,
    dwLocalPort: u32,
    dwRemoteAddr: u32,
    dwRemotePort: u32,
    dwOwningPid: u32,
}

#[allow(dead_code)]
#[allow(non_snake_case)]
#[repr(C)]
struct MIB_IFROW {
    wszName: [u16; 256],
    dwIndex: u32,
    dwType: u32,
    dwMtu: u32,
    dwSpeed: u32,
    dwPhysAddrLen: u32,
    bPhysAddr: [u8; 8],
    dwAdminStatus: u32,
    dwOperStatus: u32,
    dwLastChange: u32,
    dwInOctets: u32,
    dwInUcastPkts: u32,
    dwInNUcastPkts: u32,
    dwInDiscards: u32,
    dwInErrors: u32,
    dwInUnknownProtos: u32,
    dwOutOctets: u32,
    dwOutUcastPkts: u32,
    dwOutNUcastPkts: u32,
    dwOutDiscards: u32,
    dwOutErrors: u32,
    dwOutQLen: u32,
    dwDescrLen: u32,
    bDescr: [u8; 256],
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateToolhelp32Snapshot(dwFlags: DWORD, th32ProcessID: DWORD) -> HANDLE;
    fn Process32FirstW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
    fn Process32NextW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
    fn GetProcessTimes(
        hProcess: HANDLE,
        lpCreationTime: *mut FILETIME,
        lpExitTime: *mut FILETIME,
        lpKernelTime: *mut FILETIME,
        lpUserTime: *mut FILETIME,
    ) -> BOOL;
    fn GetSystemTimes(
        lpIdleTime: *mut FILETIME,
        lpKernelTime: *mut FILETIME,
        lpUserTime: *mut FILETIME,
    ) -> BOOL;
    fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> BOOL;
    fn TerminateProcess(hProcess: HANDLE, uExitCode: u32) -> BOOL;
    fn GetSystemInfo(lpSystemInfo: *mut SYSTEM_INFO);
    fn QueryFullProcessImageNameW(hProcess: HANDLE, dwFlags: DWORD, lpExeName: *mut u16, lpdwSize: *mut DWORD) -> BOOL;
    fn GetLogicalDrives() -> DWORD;
    fn GetDiskFreeSpaceExW(
        lpDirectoryName: *const u16,
        lpFreeBytesAvailable: *mut u64,
        lpTotalNumberOfBytes: *mut u64,
        lpTotalNumberOfFreeBytes: *mut u64,
    ) -> BOOL;
    fn SetConsoleCtrlHandler(handler: Option<unsafe extern "system" fn(DWORD) -> BOOL>, add: BOOL) -> BOOL;
}

#[link(name = "psapi")]
unsafe extern "system" {
    fn GetProcessMemoryInfo(
        hProcess: HANDLE,
        ppsmemCounters: *mut PROCESS_MEMORY_COUNTERS,
        cb: DWORD,
    ) -> BOOL;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryA(lpLibFileName: *const u8) -> HANDLE;
    fn GetProcAddress(hModule: HANDLE, lpProcName: *const u8) -> *mut u8;
}

#[allow(dead_code)]
#[link(name = "iphlpapi")]
unsafe extern "system" {
    fn GetExtendedTcpTable(
        pTcpTable: *mut u8,
        pdwSize: *mut u32,
        bOrder: BOOL,
        ulAf: u32,
        TableClass: u32,
        Reserved: u32,
    ) -> u32;
    fn GetIfTable(
        pIfTable: *mut u8,
        pdwSize: *mut u32,
        bOrder: BOOL,
    ) -> u32;
}

const _AF_INET: u32 = 2;
const _AF_INET6: u32 = 23;
const _TCP_TABLE_OWNER_PID_ALL: u32 = 5;

unsafe extern "system" fn ctrl_handler(_: DWORD) -> BOOL {
    1
}

fn install_ctrl_handler() {
    unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), 1); }
}

fn filetime_to_u64(ft: &FILETIME) -> u64 {
    (ft.dwLowDateTime as u64) | ((ft.dwHighDateTime as u64) << 32)
}

type DiskIoFn = unsafe extern "system" fn(HANDLE, *mut IO_COUNTERS) -> i32;

fn get_disk_io_fn() -> Option<DiskIoFn> {
    unsafe {
        let lib = LoadLibraryA(b"psapi.dll\0".as_ptr() as *const u8);
        if lib == 0 || lib == INVALID_HANDLE_VALUE { return None; }
        let addr = GetProcAddress(lib, b"GetProcessDiskIoCounters\0".as_ptr() as *const u8);
        if addr.is_null() { return None; }
        Some(std::mem::transmute(addr))
    }
}

fn disk_space() -> (u64, u64) {
    let drives = unsafe { GetLogicalDrives() };
    let mut total = 0u64;
    let mut free = 0u64;
    for i in 0..26u32 {
        if drives & (1 << i) != 0 {
            let root: Vec<u16> = format!("{}:\\", (b'A' as u32 + i) as u8 as char).encode_utf16().chain(std::iter::once(0)).collect();
            let (mut free_bytes, mut total_bytes, mut total_free) = (0u64, 0u64, 0u64);
            if unsafe { GetDiskFreeSpaceExW(root.as_ptr(), &mut free_bytes, &mut total_bytes, &mut total_free) != 0 } {
                total += total_bytes;
                free += total_free;
            }
        }
    }
    (total, free)
}

fn num_cpus() -> DWORD {
    let mut info = SYSTEM_INFO {
        wProcessorArchitecture: 0,
        wReserved: 0,
        dwPageSize: 0,
        lpMinimumApplicationAddress: std::ptr::null_mut(),
        lpMaximumApplicationAddress: std::ptr::null_mut(),
        dwActiveProcessorMask: 0,
        dwNumberOfProcessors: 0,
        dwProcessorType: 0,
        dwAllocationGranularity: 0,
        wProcessorLevel: 0,
        wProcessorRevision: 0,
    };
    unsafe { GetSystemInfo(&mut info); }
    info.dwNumberOfProcessors
}

fn memory_status() -> (u64, u64, u64, u64) {
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
    (state.ullTotalPhys, state.ullAvailPhys, state.ullTotalPageFile, state.ullAvailPageFile)
}

fn kill_process(pid: DWORD) -> Result<(), String> {
    let h = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
    if h == 0 || h == INVALID_HANDLE_VALUE {
        return Err(format!("access denied (PID {})", pid));
    }
    let ret = unsafe { TerminateProcess(h, 1) };
    unsafe { CloseHandle(h); }
    if ret == 0 {
        Err(format!("failed to terminate PID {}", pid))
    } else {
        Ok(())
    }
}

struct SysCpuSnap {
    idle: u64,
    kernel: u64,
    user: u64,
}

fn system_cpu_snapshot() -> SysCpuSnap {
    let mut idle = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut kernel = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut user = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user); }
    SysCpuSnap {
        idle: filetime_to_u64(&idle),
        kernel: filetime_to_u64(&kernel),
        user: filetime_to_u64(&user),
    }
}

fn system_cpu_pct(prev: &SysCpuSnap, cur: &SysCpuSnap, ncpus: DWORD) -> f64 {
    let d_idle = cur.idle - prev.idle;
    let d_total = (cur.kernel - prev.kernel) + (cur.user - prev.user);
    if d_total == 0 { return 0.0; }
    let pct = (1.0 - d_idle as f64 / d_total as f64) * 100.0;
    (pct / ncpus as f64).min(100.0)
}

#[derive(Clone)]
struct ProcInfo {
    pid: DWORD,
    name: String,
    mem: u64,
    threads: DWORD,
    raw_total: u64,
    cpu_pct: f64,
    connections: u32,
    ports: String,
}

struct HandleCache {
    handles: HashMap<DWORD, HANDLE>,
}

impl HandleCache {
    fn new() -> Self {
        HandleCache { handles: HashMap::new() }
    }

    fn get(&mut self, pid: DWORD) -> Option<HANDLE> {
        if let Some(&h) = self.handles.get(&pid) {
            return if h != 0 { Some(h) } else { None };
        }
        let h = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if h != 0 && h != INVALID_HANDLE_VALUE {
            self.handles.insert(pid, h);
            Some(h)
        } else {
            self.handles.insert(pid, 0);
            None
        }
    }

    fn retain(&mut self, active_pids: &[DWORD]) {
        let mut dead = Vec::new();
        for (&pid, &h) in &self.handles {
            if h != 0 && !active_pids.contains(&pid) {
                unsafe { CloseHandle(h); }
                dead.push(pid);
            }
        }
        for pid in dead { self.handles.remove(&pid); }
    }
}

impl Drop for HandleCache {
    fn drop(&mut self) {
        for (_, h) in &self.handles {
            if *h != 0 { unsafe { CloseHandle(*h); } }
        }
    }
}

fn get_process_path(h: HANDLE) -> Option<String> {
    let mut buf = [0u16; 260];
    let mut len = 260u32;
    let ret = unsafe { QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len) };
    if ret == 0 { return None; }
    Some(String::from_utf16_lossy(&buf[..len as usize]))
}

fn enumerate_procs(cache: &mut HandleCache) -> Vec<ProcInfo> {
    let mut list = Vec::new();
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snap == INVALID_HANDLE_VALUE { return list; }

    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as DWORD,
        cntUsage: 0,
        th32ProcessID: 0,
        th32DefaultHeapID: 0,
        th32ModuleID: 0,
        cntThreads: 0,
        th32ParentProcessID: 0,
        pcPriClassBase: 0,
        dwFlags: 0,
        szExeFile: [0; 260],
    };

    let mut ok = unsafe { Process32FirstW(snap, &mut entry) != 0 };
    while ok {
        let pid = entry.th32ProcessID;
        let exe_len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(260);
        let exe_name = String::from_utf16_lossy(&entry.szExeFile[..exe_len]);
        let threads = entry.cntThreads;

        if let Some(h) = cache.get(pid) {
            let name = get_process_path(h).unwrap_or(exe_name);
            let mut pmc = PROCESS_MEMORY_COUNTERS {
                cb: size_of::<PROCESS_MEMORY_COUNTERS>() as DWORD,
                PageFaultCount: 0,
                PeakWorkingSetSize: 0,
                WorkingSetSize: 0,
                QuotaPeakPagedPoolUsage: 0,
                QuotaPagedPoolUsage: 0,
                QuotaPeakNonPagedPoolUsage: 0,
                QuotaNonPagedPoolUsage: 0,
                _pad: 0,
                PagefileUsage: 0,
                PeakPagefileUsage: 0,
            };
            unsafe {
                GetProcessMemoryInfo(h, &mut pmc, size_of::<PROCESS_MEMORY_COUNTERS>() as DWORD);
            }

            let mut creation = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
            let mut exit = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
            let mut kernel = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
            let mut user = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
            unsafe { GetProcessTimes(h, &mut creation, &mut exit, &mut kernel, &mut user); }

            list.push(ProcInfo {
                pid,
                name,
                mem: pmc.WorkingSetSize as u64,
                threads,
                raw_total: filetime_to_u64(&kernel) + filetime_to_u64(&user),
                cpu_pct: 0.0,
                connections: 0,
                ports: String::new(),
            });
        } else {
            list.push(ProcInfo {
                pid,
                name: exe_name,
                mem: 0,
                threads,
                raw_total: 0,
                cpu_pct: 0.0,
                connections: 0,
                ports: String::new(),
            });
        }

        ok = unsafe { Process32NextW(snap, &mut entry) != 0 };
    }

    unsafe { CloseHandle(snap); }
    list
}

fn network_io() -> (u64, u64) {
    let mut size = 0u32;
    unsafe { GetIfTable(std::ptr::null_mut(), &mut size, 0); }
    if size < size_of::<MIB_IFROW>() as u32 + 4 { return (0, 0); }
    let mut buf = vec![0u8; size as usize];
    let ret = unsafe { GetIfTable(buf.as_mut_ptr(), &mut size, 0) };
    if ret != 0 { return (0, 0); }
    let num = unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const u32) };
    let mut total_in = 0u64;
    let mut total_out = 0u64;
    for i in 0..num {
        let row_ptr = unsafe {
            (buf.as_ptr() as *const u8).add(4 + i as usize * size_of::<MIB_IFROW>()) as *const MIB_IFROW
        };
        let row = unsafe { std::ptr::read_unaligned(row_ptr) };
        if row.dwType == 24 || row.dwType == 6 || row.dwType == 1 { continue; } // loopback, tunnel, software
        if row.dwOperStatus != 1 { continue; } // 1 = IF_OPER_STATUS_UP
        total_in += row.dwInOctets as u64;
        total_out += row.dwOutOctets as u64;
    }
    (total_in, total_out)
}

fn get_connection_info() -> (HashMap<DWORD, u32>, HashMap<DWORD, String>) {
    let mut counts = HashMap::new();
    let mut port_map: HashMap<DWORD, Vec<u16>> = HashMap::new();
    let mut size = 0u32;
    unsafe {
        GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 0, 2, 5, 0);
    }
    if size == 0 { return (counts, HashMap::new()); }
    let mut buf = vec![0u8; size as usize];
    let ret = unsafe {
        GetExtendedTcpTable(buf.as_mut_ptr(), &mut size, 0, 2, 5, 0)
    };
    if ret != 0 { return (counts, HashMap::new()); }
    let num_entries = unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const DWORD) };
    for i in 0..num_entries {
        let row_ptr = unsafe {
            (buf.as_ptr() as *const u8).add(4 + i as usize * size_of::<MIB_TCPROW_OWNER_PID>()) as *const MIB_TCPROW_OWNER_PID
        };
        let row = unsafe { std::ptr::read_unaligned(row_ptr) };
        let pid = row.dwOwningPid;
        *counts.entry(pid).or_insert(0) += 1;
        let port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);
        let ports = port_map.entry(pid).or_default();
        if !ports.contains(&port) {
            ports.push(port);
        }
    }
    let port_strings: HashMap<DWORD, String> = port_map.into_iter().map(|(pid, mut ports)| {
        ports.sort();
        let s = ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",");
        if s.len() > 18 {
            (pid, format!("{},…", &s[..16]))
        } else {
            (pid, s)
        }
    }).collect();
    (counts, port_strings)
}

fn compute_cpu(procs: &mut [ProcInfo], prev: &HashMap<DWORD, u64>, dt_ms: u64) {
    if dt_ms == 0 { return; }
    let dt_100ns = dt_ms as f64 * 10000.0;
    for p in procs.iter_mut() {
        let prev_total = prev.get(&p.pid).copied().unwrap_or(p.raw_total);
        let d = p.raw_total.saturating_sub(prev_total);
        p.cpu_pct = d as f64 / dt_100ns * 100.0;
    }
}

fn sort_procs(procs: &mut [ProcInfo], sort_by: &str, reverse: bool) {
    let cmp: fn(&ProcInfo, &ProcInfo) -> std::cmp::Ordering = match sort_by {
        "pid" => |a, b| a.pid.cmp(&b.pid),
        "name" => |a, b| a.name.cmp(&b.name),
        "mem" => |a, b| a.mem.cmp(&b.mem),
        "threads" => |a, b| a.threads.cmp(&b.threads),
        "conn" => |a, b| a.connections.cmp(&b.connections),
        _ => |a, b| a.cpu_pct.partial_cmp(&b.cpu_pct).unwrap_or(std::cmp::Ordering::Equal),
    };
    if reverse {
        procs.sort_by(|a, b| cmp(b, a));
    } else {
        procs.sort_by(cmp);
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1}{}", size, UNITS[unit_idx])
}

fn format_rate(bytes_per_sec: f64) -> String {
    const UNITS: &[&str] = &["B/s", "KiB/s", "MiB/s", "GiB/s", "TiB/s"];
    let mut size = bytes_per_sec;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1}{}", size, UNITS[unit_idx])
}

fn build_display(last_procs: &[ProcInfo], filter: &str, max_lines: usize) -> (Vec<ProcInfo>, usize) {
    let mut v: Vec<ProcInfo> = if filter.is_empty() {
        last_procs.to_vec()
    } else {
        let lower = filter.to_ascii_lowercase();
        last_procs.iter().filter(|p| p.name.to_ascii_lowercase().contains(&lower)).cloned().collect()
    };
    let total = v.len();
    if max_lines < v.len() { v.truncate(max_lines); }
    (v, total)
}

fn print_usage() {
    eprintln!("Usage: wtop [options]");
    eprintln!("Display real-time process information.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -n <seconds>       Refresh interval (default: 2)");
    eprintln!("  --sort <field>     Sort field: cpu, mem, pid, name, threads (default: cpu)");
    eprintln!("  -r                 Reverse sort order");
    eprintln!("  -L <lines>         Max lines to show (default: all)");
    eprintln!("  --help, -h         Show this help");
}

fn usage_color(pct: f64) -> Color {
    if pct > 90.0 { Color::Red } else if pct > 75.0 { Color::Yellow } else { Color::Green }
}

fn cpu_cell_color(pct: f64) -> Color {
    if pct > 50.0 { Color::Red } else if pct > 20.0 { Color::Yellow } else { Color::Reset }
}

fn render(frame: &mut Frame, procs: &[ProcInfo], sys_cpu: f64, used_phys: u64, total_phys: u64,
          mem_pct: f64, used_commit: u64, total_commit: u64, commit_pct: f64,
          disk_read_rate: f64, disk_write_rate: f64, disk_total_bytes: u64, disk_free_bytes: u64,
          net_in_rate: f64, net_out_rate: f64,
          interval: u64, sort_by: &str, reverse: bool,
          table_state: &mut TableState, status_msg: Option<&str>,
          filter: &str, filter_mode: bool, total_count: usize) {
    let area = frame.area();

    let block = Block::default()
        .title(" wtop v0.1 ")
        .title_alignment(Alignment::Left)
        .borders(ratatui::widgets::Borders::ALL)
        .border_type(BorderType::Plain);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(0),
    ]).split(inner);

    let used_mb = (used_phys / 1048576) as u64;
    let total_mb = (total_phys / 1048576) as u64;
    let used_commit_mb = (used_commit / 1048576) as u64;
    let total_commit_mb = (total_commit / 1048576) as u64;
    let disk_used_bytes = disk_total_bytes.saturating_sub(disk_free_bytes);

    let sort_indicator = if reverse { "▼" } else { "▲" };

    let status_line = if let Some(msg) = status_msg {
        Line::from(vec![
            Span::styled(format!("  ⚠ {}  ", msg), Style::default().fg(Color::Red).bold()),
        ])
    } else if filter_mode {
        Line::from(vec![
            Span::styled(" /", Style::default().fg(Color::Cyan).bold()),
            Span::styled(format!("{}", filter), Style::default().fg(Color::White).bold()),
            Span::styled("_", Style::default().fg(Color::Cyan).bold()),
            Span::styled("  [Enter] ok  [Esc] cancel", Style::default().fg(Color::DarkGray)),
        ])
    } else if !filter.is_empty() {
        Line::from(vec![
            Span::styled(format!("  [Filter: \"{}\"]", filter), Style::default().fg(Color::Yellow)),
        ])
    } else {
        Line::from(vec![
            Span::styled("  wtop", Style::default().fg(Color::DarkGray)),
        ])
    };

    let procs_count_str = if !filter.is_empty() {
        format!("{}/{}", procs.len(), total_count)
    } else {
        format!("{}", procs.len())
    };

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("CPU:", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" "),
            Span::styled(format!("{:>5.1}%", sys_cpu), Style::default().fg(usage_color(sys_cpu)).bold()),
            Span::raw("  │  "),
            Span::styled("Memory:", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" "),
            Span::styled(format!("{:>4}/{:>3} MiB", used_mb, total_mb), Style::default().fg(usage_color(mem_pct)).bold()),
            Span::styled(format!(" ({:.1}%)", mem_pct), Style::default().fg(usage_color(mem_pct))),
            Span::raw("  │  "),
            Span::styled("Swap:", Style::default().fg(Color::Magenta).bold()),
            Span::raw(" "),
            Span::styled(format!("{:>4}/{:>3} MiB", used_commit_mb, total_commit_mb), Style::default().fg(usage_color(commit_pct)).bold()),
            Span::styled(format!(" ({:.1}%)", commit_pct), Style::default().fg(usage_color(commit_pct))),
        ]),
        Line::from(vec![
            Span::styled("Disk", Style::default().fg(Color::Blue).bold()),
            Span::raw(" R:"),
            Span::styled(format_rate(disk_read_rate), Style::default().fg(Color::Green)),
            Span::raw("  W:"),
            Span::styled(format_rate(disk_write_rate), Style::default().fg(Color::Yellow)),
            Span::raw("  │  "),
            Span::styled(format!("{}", format_size(disk_used_bytes)), Style::default().fg(Color::Cyan)),
            Span::raw("/"),
            Span::styled(format!("{}", format_size(disk_total_bytes)), Style::default().fg(Color::White)),
            Span::styled(format!(" ({:.1}%)", if disk_total_bytes > 0 { disk_used_bytes as f64 / disk_total_bytes as f64 * 100.0 } else { 0.0 }), Style::default().fg(Color::DarkGray)),
            Span::raw("  │  "),
            Span::styled("Net", Style::default().fg(Color::Magenta).bold()),
            Span::raw(" "),
            Span::styled("▼", Style::default().fg(Color::Cyan)),
            Span::styled(format_rate(net_in_rate), Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled("▲", Style::default().fg(Color::Yellow)),
            Span::styled(format_rate(net_out_rate), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw(format!("Procs: {}   {}s", procs_count_str, interval)),
            Span::raw("   "),
            Span::styled(format!("[{} {}]", sort_indicator, sort_by), Style::default().dim()),
            Span::styled(" 排序:", Style::default().fg(Color::DarkGray)),
            Span::styled("[c]", Style::default().fg(Color::Yellow)),
            Span::styled("[m]", Style::default().fg(Color::Cyan)),
            Span::styled("[p]", Style::default().fg(Color::DarkGray)),
            Span::styled("[n]", Style::default().fg(Color::White)),
            Span::styled("[t]", Style::default().fg(Color::Green)),
            Span::styled("[o] ", Style::default().fg(Color::Magenta)),
            Span::styled("[r]反向 ", Style::default().fg(Color::DarkGray)),
            Span::styled("[a]全部 ", Style::default().fg(Color::DarkGray)),
            Span::styled("[/]过滤 ", Style::default().fg(Color::DarkGray)),
            Span::styled("[k]杀 ", Style::default().fg(Color::DarkGray)),
            Span::styled("[q]退出", Style::default().fg(Color::DarkGray)),
        ]),
        status_line,
    ]);

    frame.render_widget(header, chunks[0]);

    let selected_idx = table_state.selected();
    let rows = procs.iter().enumerate().map(|(i, p)| {
        let selected = selected_idx == Some(i);
        let cpu_style = if selected { Style::default() } else { Style::default().fg(cpu_cell_color(p.cpu_pct)) };
        let mem_s = if p.mem > 0 { format_size(p.mem) } else { "-".to_string() };
        let threads_str = format!("{:>5}", p.threads);
        let mut cells = vec![
            Cell::from(Line::from(format!("{}", p.pid)).alignment(Alignment::Right))
                .style(if selected { Style::default() } else { Style::default().fg(Color::DarkGray) }),
            Cell::from(p.name.as_str()),
        ];
        if p.cpu_pct > 0.1 {
            cells.push(
                Cell::from(Line::from(format!("{:.1}%", p.cpu_pct)).alignment(Alignment::Right))
                    .style(cpu_style)
            );
        } else {
            cells.push(Cell::from(Line::from("-").alignment(Alignment::Right)).style(if selected { Style::default() } else { Style::default().dim() }));
        }
        cells.push(
            Cell::from(Line::from(mem_s).alignment(Alignment::Right))
                .style(if selected { Style::default() } else { Style::default().fg(Color::Cyan) })
        );
        cells.push(
            Cell::from(Line::from(threads_str).alignment(Alignment::Right))
                .style(if selected { Style::default() } else { Style::default().fg(Color::Green) })
        );
        cells.push(
            Cell::from(Line::from(if p.ports.is_empty() { "-".to_string() } else { p.ports.clone() }).alignment(Alignment::Right))
                .style(if selected { Style::default() } else { Style::default().fg(Color::Blue) })
        );
        Row::new(cells)
    });

    let widths = [
        Constraint::Length(7),
        Constraint::Fill(1),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(5),
        Constraint::Length(20),
    ];

    let header_cells = [
        Cell::from(Line::from("PID").alignment(Alignment::Right)).style(Style::default().fg(Color::DarkGray).bold()),
        Cell::from("NAME").style(Style::default().fg(Color::White).bold()),
        Cell::from(Line::from("CPU%").alignment(Alignment::Right)).style(Style::default().fg(Color::Yellow).bold()),
        Cell::from(Line::from("MEM").alignment(Alignment::Right)).style(Style::default().fg(Color::Cyan).bold()),
        Cell::from(Line::from("THR").alignment(Alignment::Right)).style(Style::default().fg(Color::Green).bold()),
        Cell::from(Line::from("PORTS").alignment(Alignment::Right)).style(Style::default().fg(Color::Blue).bold()),
    ];
    let table = Table::new(rows, widths)
        .header(Row::new(header_cells.iter().map(|h| h.clone())))
        .row_highlight_style(Style::default().fg(Color::White).bg(Color::Blue));

    frame.render_stateful_widget(table, chunks[1], table_state);
}

fn run(terminal: &mut DefaultTerminal, interval: u64, sort_by: &mut String, reverse: &mut bool, max_lines: &mut usize) -> io::Result<()> {
    let ncpus = num_cpus();
    let sleep_dur = Duration::from_secs(interval);

    let mut handle_cache = HandleCache::new();
    let mut prev_cpu = system_cpu_snapshot();
    let mut prev_procs: HashMap<DWORD, u64> = HashMap::new();
    let mut prev_disk_read = 0u64;
    let mut prev_disk_write = 0u64;
    let mut prev_net_in;
    let mut prev_net_out;
    let disk_io_fn = get_disk_io_fn();
    let init_procs = enumerate_procs(&mut handle_cache);
    for p in &init_procs {
        prev_procs.insert(p.pid, p.raw_total);
    }
    for p in &init_procs {
        if let Some(h) = handle_cache.get(p.pid) {
            if let Some(f) = disk_io_fn {
                let mut io = IO_COUNTERS {
                    ReadOperationCount: 0, WriteOperationCount: 0, OtherOperationCount: 0,
                    ReadTransferCount: 0, WriteTransferCount: 0, OtherTransferCount: 0,
                };
                if unsafe { f(h, &mut io) != 0 } {
                    prev_disk_read += io.ReadTransferCount;
                    prev_disk_write += io.WriteTransferCount;
                }
            }
        }
    }
    let (init_net_in, init_net_out) = network_io();
    prev_net_in = init_net_in;
    prev_net_out = init_net_out;
    std::thread::sleep(sleep_dur);

    let mut last_refresh = Instant::now();
    let mut dt_ms = sleep_dur.as_millis() as u64;

    ratatui::crossterm::execute!(
        io::stdout(),
        ratatui::crossterm::event::EnableMouseCapture
    )?;

    let mut table_state = TableState::default();
    table_state.select(Some(0));

    let mut last_procs: Vec<ProcInfo> = Vec::new();
    let mut last_sys_cpu = 0.0f64;
    let mut last_used_phys = 0u64;
    let mut last_total_phys = 0u64;
    let mut last_mem_pct = 0.0f64;
    let mut last_used_commit = 0u64;
    let mut last_total_commit = 0u64;
    let mut last_commit_pct = 0.0f64;
    let mut last_disk_read_rate = 0.0f64;
    let mut last_disk_write_rate = 0.0f64;
    let mut last_disk_total = 0u64;
    let mut last_disk_free = 0u64;
    let mut last_net_in_rate = 0.0f64;
    let mut last_net_out_rate = 0.0f64;
    let mut status_msg: Option<(String, Instant)> = None;
    let mut filter = String::new();
    let mut filter_mode = false;

    loop {
        if matches!(event::poll(Duration::from_millis(100)), Ok(true)) {
            let mut needs_render = false;
            if let Ok(event) = event::read() {
                match event {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if filter_mode {
                        match key.code {
                            KeyCode::Char(c) if !c.is_control() && c != '\n' && c != '\r' => { filter.push(c); needs_render = true; }
                            KeyCode::Backspace => { filter.pop(); needs_render = true; }
                            KeyCode::Enter => { filter_mode = false; needs_render = true; }
                            KeyCode::Esc => { filter.clear(); filter_mode = false; needs_render = true; }
                            KeyCode::Up => { move_sel(&mut table_state, -1); needs_render = true; }
                            KeyCode::Down => { move_sel(&mut table_state, 1); needs_render = true; }
                            KeyCode::PageUp => { move_sel(&mut table_state, -20); needs_render = true; }
                            KeyCode::PageDown => { move_sel(&mut table_state, 20); needs_render = true; }
                            KeyCode::Home => { table_state.select(Some(0)); needs_render = true; }
                            KeyCode::End => { table_state.select(Some(usize::MAX)); needs_render = true; }
                            _ => {}
                        }
                    } else {
                        match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::NONE => {
                            *sort_by = "cpu".to_string(); needs_render = true;
                        }
                        KeyCode::Char('m') => { *sort_by = "mem".to_string(); needs_render = true; }
                        KeyCode::Char('p') => { *sort_by = "pid".to_string(); needs_render = true; }
                        KeyCode::Char('n') => { *sort_by = "name".to_string(); needs_render = true; }
                        KeyCode::Char('t') => { *sort_by = "threads".to_string(); needs_render = true; }
                        KeyCode::Char('o') => { *sort_by = "conn".to_string(); needs_render = true; }
                        KeyCode::Char('r') => { *reverse = !*reverse; needs_render = true; }
                        KeyCode::Char('a') => { *max_lines = usize::MAX; needs_render = true; }
                        KeyCode::Char('/') => { filter_mode = true; needs_render = true; }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Up => { move_sel(&mut table_state, -1); needs_render = true; }
                        KeyCode::Down => { move_sel(&mut table_state, 1); needs_render = true; }
                        KeyCode::PageUp => { move_sel(&mut table_state, -20); needs_render = true; }
                        KeyCode::PageDown => { move_sel(&mut table_state, 20); needs_render = true; }
                        KeyCode::Home => { table_state.select(Some(0)); needs_render = true; }
                        KeyCode::End => { table_state.select(Some(usize::MAX)); needs_render = true; }
                        KeyCode::Char('k') | KeyCode::Delete => {
                            if let Some(idx) = table_state.selected() {
                                let pid = if filter.is_empty() {
                                    last_procs.get(idx).map(|p| p.pid)
                                } else {
                                    let lower = filter.to_ascii_lowercase();
                                    last_procs.iter()
                                        .filter(|p| p.name.to_ascii_lowercase().contains(&lower))
                                        .nth(idx).map(|p| p.pid)
                                };
                                if let Some(pid) = pid {
                                    let proc_opt = last_procs.iter().find(|p| p.pid == pid);
                                    let name = proc_opt.map(|p| p.name.clone()).unwrap_or_default();
                                    match kill_process(pid) {
                                        Ok(()) => status_msg = Some((format!("Killed {} ({})", pid, name), Instant::now())),
                                        Err(e) => status_msg = Some((format!("{}", e), Instant::now())),
                                    }
                                    needs_render = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::Mouse(m) => {
                    needs_render = true;
                    match m.kind {
                        MouseEventKind::ScrollUp => { move_sel(&mut table_state, -3); }
                        MouseEventKind::ScrollDown => { move_sel(&mut table_state, 3); }
                        MouseEventKind::Down(..) => {
                            if m.row >= 4 {
                                table_state.select(Some((m.row - 4) as usize));
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            }

            if needs_render && !last_procs.is_empty() {
                sort_procs(&mut last_procs, sort_by, *reverse);
                let (display_procs, _) = build_display(&last_procs, &filter, *max_lines);
                let max_idx = display_procs.len().saturating_sub(1);
                if let Some(idx) = table_state.selected() {
                    table_state.select(Some(idx.min(max_idx)));
                } else if !display_procs.is_empty() {
                    table_state.select(Some(0));
                }
                let status = status_msg.as_ref()
                    .and_then(|(s, t)| if t.elapsed() < Duration::from_secs(3) { Some(s.as_str()) } else { None });
                let _ = terminal.draw(|f| {
                    render(f, &display_procs, last_sys_cpu, last_used_phys, last_total_phys, last_mem_pct,
                           last_used_commit, last_total_commit, last_commit_pct,
                           last_disk_read_rate, last_disk_write_rate, last_disk_total, last_disk_free,
                           last_net_in_rate, last_net_out_rate,
                           interval, sort_by, *reverse, &mut table_state, status,
                           &filter, filter_mode, last_procs.len());
                });
            }
        }

        if last_refresh.elapsed() >= sleep_dur {
            let now = Instant::now();

            let cur_cpu = system_cpu_snapshot();
            let sys_cpu = system_cpu_pct(&prev_cpu, &cur_cpu, ncpus);
            prev_cpu = cur_cpu;

            let mut procs = enumerate_procs(&mut handle_cache);
            compute_cpu(&mut procs, &prev_procs, dt_ms);

            prev_procs.clear();
            for p in &procs {
                prev_procs.insert(p.pid, p.raw_total);
            }
            handle_cache.retain(&procs.iter().map(|p| p.pid).collect::<Vec<_>>());

            let (conn_counts, port_strings) = get_connection_info();
            for p in &mut procs {
                p.connections = conn_counts.get(&p.pid).copied().unwrap_or(0);
                p.ports = port_strings.get(&p.pid).cloned().unwrap_or_default();
            }

            sort_procs(&mut procs, sort_by, *reverse);

            let (total_phys, avail_phys, total_page, avail_page) = memory_status();
            let used_phys = total_phys - avail_phys;
            let used_commit = total_page - avail_page;
            let mem_pct = if total_phys > 0 { used_phys as f64 / total_phys as f64 * 100.0 } else { 0.0 };
            let commit_pct = if total_page > 0 { used_commit as f64 / total_page as f64 * 100.0 } else { 0.0 };

            // Disk I/O
            let mut disk_read = 0u64;
            let mut disk_write = 0u64;
            for p in &procs {
                if let Some(h) = handle_cache.get(p.pid) {
                    if let Some(dio) = disk_io_fn {
                        let mut io = IO_COUNTERS {
                            ReadOperationCount: 0, WriteOperationCount: 0, OtherOperationCount: 0,
                            ReadTransferCount: 0, WriteTransferCount: 0, OtherTransferCount: 0,
                        };
                        if unsafe { dio(h, &mut io) != 0 } {
                            disk_read += io.ReadTransferCount;
                            disk_write += io.WriteTransferCount;
                        }
                    }
                }
            }
            let dt_secs = dt_ms as f64 / 1000.0;
            last_disk_read_rate = if prev_disk_read > 0 && dt_secs > 0.0 {
                (disk_read - prev_disk_read) as f64 / dt_secs
            } else { 0.0 };
            last_disk_write_rate = if prev_disk_write > 0 && dt_secs > 0.0 {
                (disk_write - prev_disk_write) as f64 / dt_secs
            } else { 0.0 };
            prev_disk_read = disk_read;
            prev_disk_write = disk_write;

            // Disk space
            let (disk_total, disk_free) = disk_space();
            last_disk_total = disk_total;
            last_disk_free = disk_free;

            // Network IO
            let (net_in, net_out) = network_io();
            last_net_in_rate = if prev_net_in > 0 && dt_secs > 0.0 {
                (net_in.wrapping_sub(prev_net_in)) as f64 / dt_secs
            } else { 0.0 };
            last_net_out_rate = if prev_net_out > 0 && dt_secs > 0.0 {
                (net_out.wrapping_sub(prev_net_out)) as f64 / dt_secs
            } else { 0.0 };
            prev_net_in = net_in;
            prev_net_out = net_out;

            dt_ms = now.duration_since(last_refresh).as_millis() as u64;
            last_refresh = now;

            // Store for immediate re-render on key events
            last_procs = procs;
            last_sys_cpu = sys_cpu;
            last_used_phys = used_phys;
            last_total_phys = total_phys;
            last_mem_pct = mem_pct;
            last_used_commit = used_commit;
            last_total_commit = total_page;
            last_commit_pct = commit_pct;

            let (display_procs, _) = build_display(&last_procs, &filter, *max_lines);

            let max_idx = display_procs.len().saturating_sub(1);
            if let Some(idx) = table_state.selected() {
                table_state.select(Some(idx.min(max_idx)));
            } else if !display_procs.is_empty() {
                table_state.select(Some(0));
            }

            let status = status_msg.as_ref()
                .and_then(|(s, t)| if t.elapsed() < Duration::from_secs(3) { Some(s.as_str()) } else { None });
            let _ = terminal.draw(|f| {
                render(f, &display_procs, last_sys_cpu, last_used_phys, last_total_phys, last_mem_pct,
                       last_used_commit, last_total_commit, last_commit_pct,
                       last_disk_read_rate, last_disk_write_rate, last_disk_total, last_disk_free,
                       last_net_in_rate, last_net_out_rate,
                       interval, sort_by, *reverse, &mut table_state, status,
                       &filter, filter_mode, last_procs.len());
            });
        }
    }

    let _ = ratatui::crossterm::execute!(
        io::stdout(),
        ratatui::crossterm::event::DisableMouseCapture
    );
    Ok(())
}

fn move_sel(state: &mut TableState, delta: isize) {
    let current = state.selected().unwrap_or(0);
    if delta < 0 {
        state.select(Some(current.saturating_sub((-delta) as usize)));
    } else {
        state.select(Some(current.saturating_add(delta as usize)));
    }
}

pub fn uumain(raw_args: impl Iterator<Item = OsString>) -> i32 {
    install_ctrl_handler();
    let args: Vec<String> = raw_args.map(|s| s.to_string_lossy().into_owned()).collect();
    let mut interval = 2u64;
    let mut sort_by = "cpu".to_string();
    let mut reverse = false;
    let mut max_lines = usize::MAX;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => {
                i += 1;
                if i < args.len() { interval = args[i].parse().unwrap_or(2); }
            }
            "--sort" => {
                i += 1;
                if i < args.len() { sort_by = args[i].clone(); }
            }
            "-r" => reverse = true,
            "-L" => {
                i += 1;
                if i < args.len() { max_lines = args[i].parse().unwrap_or(usize::MAX); }
            }
            "--help" | "-h" => { print_usage(); return 0; }
            _ => { print_usage(); return 1; }
        }
        i += 1;
    }

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, interval, &mut sort_by, &mut reverse, &mut max_lines);
    ratatui::restore();
    if let Err(e) = result {
        eprintln!("{}", e);
        1
    } else {
        0
    }
}
