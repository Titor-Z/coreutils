#![allow(non_snake_case, non_camel_case_types, unused)]
mod config;

use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Gauge, Paragraph, Row, Table, TableState},
};
use clap::Command;

use config::Config;

type HANDLE = isize;
type BOOL = i32;
type DWORD = u32;
type LPCWSTR = *const u16;

const INVALID_HANDLE_VALUE: isize = -1;

#[repr(C)]
struct FILETIME {
    dwLowDateTime: DWORD,
    dwHighDateTime: DWORD,
}

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

#[repr(C)]
struct IO_COUNTERS {
    ReadOperationCount: u64,
    WriteOperationCount: u64,
    OtherOperationCount: u64,
    ReadTransferCount: u64,
    WriteTransferCount: u64,
    OtherTransferCount: u64,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetConsoleCtrlHandler(handler: Option<unsafe extern "system" fn(DWORD) -> BOOL>, add: BOOL) -> BOOL;
    fn GetLogicalDrives() -> DWORD;
    fn GetDiskFreeSpaceExW(lpDirectoryName: LPCWSTR, lpFreeBytesAvailable: *mut u64, lpTotalNumberOfBytes: *mut u64, lpTotalNumberOfFreeBytes: *mut u64) -> BOOL;
    fn GetDriveTypeW(lpRootPathName: LPCWSTR) -> DWORD;
    fn GetSystemTimes(lpIdleTime: *mut FILETIME, lpKernelTime: *mut FILETIME, lpUserTime: *mut FILETIME) -> BOOL;
    fn GetVolumeInformationW(lpRootPathName: LPCWSTR, lpVolumeNameBuffer: *mut u16, nVolumeNameSize: DWORD, lpVolumeSerialNumber: *mut DWORD, lpMaximumComponentLength: *mut DWORD, lpFileSystemFlags: *mut DWORD, lpFileSystemNameBuffer: *mut u16, nFileSystemNameSize: DWORD) -> BOOL;
    fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> BOOL;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn CreateFileW(lpFileName: LPCWSTR, dwDesiredAccess: DWORD, dwShareMode: DWORD, lpSecurityAttributes: *mut std::ffi::c_void, dwCreationDisposition: DWORD, dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE) -> HANDLE;
    fn DeviceIoControl(hDevice: HANDLE, dwIoControlCode: DWORD, lpInBuffer: *mut std::ffi::c_void, nInBufferSize: DWORD, lpOutBuffer: *mut std::ffi::c_void, nOutBufferSize: DWORD, lpBytesReturned: *mut DWORD, lpOverlapped: *mut std::ffi::c_void) -> BOOL;
    fn GetCurrentProcess() -> HANDLE;
    fn GetProcessIoCounters(hProcess: HANDLE, lpIoCounters: *mut IO_COUNTERS) -> BOOL;
}

const GENERIC_READ: DWORD = 0x80000000;
const FILE_SHARE_READ: DWORD = 0x00000001;
const FILE_SHARE_WRITE: DWORD = 0x00000002;
const OPEN_EXISTING: DWORD = 3;
#[repr(C)]
struct DISK_EXTENT {
    disk_number: DWORD,
    starting_offset: i64,
    extent_length: i64,
}

#[repr(C)]
struct VOLUME_DISK_EXTENTS {
    number_of_disk_extents: DWORD,
    extents: [DISK_EXTENT; 1],
}

fn get_disk_number(letter: char) -> Option<u32> {
    let path: Vec<u16> = format!("\\\\.\\{}:", letter).encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let h = CreateFileW(path.as_ptr(), GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(), OPEN_EXISTING, 0, INVALID_HANDLE_VALUE);
        if h == INVALID_HANDLE_VALUE || h == 0 { return None; }
        let mut extents: VOLUME_DISK_EXTENTS = std::mem::zeroed();
        let mut ret: DWORD = 0;
        let ok = DeviceIoControl(h, 0x00560000, std::ptr::null_mut(), 0,
            &mut extents as *mut _ as *mut std::ffi::c_void, size_of::<VOLUME_DISK_EXTENTS>() as DWORD,
            &mut ret, std::ptr::null_mut());
        CloseHandle(h);
        if ok == 0 { return None; }
        Some(extents.extents[0].disk_number)
    }
}

fn current_process_io() -> (u64, u64) {
    unsafe {
        let h = GetCurrentProcess();
        let mut counters = IO_COUNTERS {
            ReadOperationCount: 0, WriteOperationCount: 0,
            OtherOperationCount: 0, ReadTransferCount: 0,
            WriteTransferCount: 0, OtherTransferCount: 0,
        };
        if GetProcessIoCounters(h, &mut counters) == 0 { return (0, 0); }
        (counters.ReadTransferCount, counters.WriteTransferCount)
    }
}

unsafe extern "system" fn ctrl_handler(_: DWORD) -> BOOL { 1 }

fn install_ctrl_handler() {
    unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), 1); }
}

fn filetime_to_u64(ft: &FILETIME) -> u64 {
    (ft.dwLowDateTime as u64) | ((ft.dwHighDateTime as u64) << 32)
}

fn system_cpu_snapshot() -> (u64, u64, u64) {
    let mut idle = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut kernel = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut user = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user); }
    (filetime_to_u64(&idle), filetime_to_u64(&kernel), filetime_to_u64(&user))
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{:.0}{}", size, UNITS[unit_idx])
    } else {
        format!("{:.1}{}", size, UNITS[unit_idx])
    }
}

fn format_rate(bytes_per_sec: f64) -> String {
    const UNITS: &[&str] = &["B/s", "KiB/s", "MiB/s", "GiB/s"];
    let mut rate = bytes_per_sec;
    let mut unit_idx = 0;
    while rate >= 1024.0 && unit_idx < UNITS.len() - 1 {
        rate /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1}{}", rate, UNITS[unit_idx])
}

fn usage_color(pct: f64, palette: &config::Palette) -> Color {
    palette.usage_color(pct)
}

#[derive(Clone)]
struct VolumeInfo {
    letter: char,
    label: String,
    fs_type: String,
    total: u64,
    free: u64,
    disk_number: Option<u32>,
}

fn get_volumes() -> Vec<VolumeInfo> {
    let mut vols = Vec::new();
    unsafe {
        let drives = GetLogicalDrives();
        for i in 0..26 {
            if drives & (1 << i) == 0 { continue; }
            let letter = (b'A' + i) as char;
            let root = format!("{}:\\", letter);
            let root_w: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
            let dt = GetDriveTypeW(root_w.as_ptr());
            if dt != 3 { continue; }
            let mut free_bytes: u64 = 0;
            let mut total_bytes: u64 = 0;
            let ret = GetDiskFreeSpaceExW(root_w.as_ptr(), &mut free_bytes, &mut total_bytes, std::ptr::null_mut());
            if ret == 0 { continue; }
            let mut vol_name = [0u16; 256];
            let mut fs_name = [0u16; 256];
            let mut _sn: DWORD = 0;
            let mut _max_component: DWORD = 0;
            let mut _fs_flags: DWORD = 0;
            let _ = GetVolumeInformationW(
                root_w.as_ptr(), vol_name.as_mut_ptr(), 256, &mut _sn,
                &mut _max_component, &mut _fs_flags, fs_name.as_mut_ptr(), 256,
            );
            let label = String::from_utf16_lossy(&vol_name).trim_end_matches(char::from(0)).to_string();
            let fs_type = String::from_utf16_lossy(&fs_name).trim_end_matches(char::from(0)).to_string();
            let disk_number = get_disk_number(letter);
            vols.push(VolumeInfo { letter, label, fs_type, total: total_bytes, free: free_bytes, disk_number });
        }
    }
    vols.sort_by(|a, b| a.letter.cmp(&b.letter));
    vols
}

fn disk_collect_all() -> (u64, u64) {
    let vols = get_volumes();
    let mut total = 0u64;
    let mut free = 0u64;
    for v in &vols {
        total = total.saturating_add(v.total);
        free = free.saturating_add(v.free);
    }
    (total, free)
}

#[derive(Default, Clone)]
struct Categories {
    documents: u64,
    pictures: u64,
    audio: u64,
    video: u64,
    other: u64,
    applications: u64,
    system: u64,
    cache: u64,
}

#[derive(Clone)]
struct LargeFile {
    path: String,
    size: u64,
    category: u8,
    mtime: Option<u64>,
}

enum SortField { Size, Name, Time }

fn classify_by_ext(lower: &str) -> u8 {
    let ext = std::path::Path::new(lower).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" | "doc" | "docx" | "txt" | "md" | "xls" | "xlsx" | "ppt" | "pptx" | "csv" | "rtf" | "epub" => 0,
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" | "heic" | "avif" | "tiff" | "tif" | "raw" => 1,
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" | "wv" | "ape" => 2,
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "ts" | "mts" => 3,
        _ => 4,
    }
}

fn categorize_path(lower: &str) -> u8 {
    if lower.contains("\\temp") || lower.contains("\\prefetch") || lower.contains("\\inetcache")
        || lower.contains("\\softwaredistribution\\download")
    {
        return 7;
    }
    if lower.contains("\\program files") || lower.contains("\\program files (x86)") {
        return 5;
    }
    if lower.contains("\\windows") {
        return 6;
    }
    classify_by_ext(lower)
}

fn scan_dir(dir: &str, fixed_cat: Option<u8>, thresholds: &config::Thresholds, cats: &Arc<Mutex<Categories>>, files: &Arc<Mutex<Vec<LargeFile>>>, dirs_scanned: &Arc<AtomicU64>, cancelled: &Arc<AtomicBool>) {
    let mut stack = vec![dir.to_string()];
    while let Some(d) = stack.pop() {
        if cancelled.load(Ordering::Relaxed) { return; }
        match std::fs::read_dir(&d) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    if cancelled.load(Ordering::Relaxed) { return; }
                    let path = entry.path();
                    let lower = path.to_string_lossy().to_lowercase();
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        stack.push(path.to_string_lossy().to_string());
                    } else if let Ok(meta) = entry.metadata() {
                        let size = meta.len();
                        let file_cat = match fixed_cat { Some(c) => c, None => categorize_path(&lower) };
                        let mut cats_guard = cats.lock().unwrap();
                        match file_cat {
                            0 => cats_guard.documents = cats_guard.documents.saturating_add(size),
                            1 => cats_guard.pictures = cats_guard.pictures.saturating_add(size),
                            2 => cats_guard.audio = cats_guard.audio.saturating_add(size),
                            3 => cats_guard.video = cats_guard.video.saturating_add(size),
                            4 => cats_guard.other = cats_guard.other.saturating_add(size),
                            5 => cats_guard.applications = cats_guard.applications.saturating_add(size),
                            6 => cats_guard.system = cats_guard.system.saturating_add(size),
                            _ => cats_guard.cache = cats_guard.cache.saturating_add(size),
                        }
                        drop(cats_guard);
                        if size >= thresholds.get(file_cat) {
                            let mut files_guard = files.lock().unwrap();
                            let mtime = meta.modified().ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs());
                            files_guard.push(LargeFile { path: path.to_string_lossy().to_string(), size, category: file_cat, mtime });
                            files_guard.sort_by(|a, b| b.size.cmp(&a.size));
                            if files_guard.len() > 50000 { files_guard.truncate(50000); }
                        }
                    }
                }
                dirs_scanned.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {}
        }
    }
}

fn scan_drive(drive: char, thresholds: Arc<config::Thresholds>, cats: Arc<Mutex<Categories>>, files: Arc<Mutex<Vec<LargeFile>>>, dirs_scanned: Arc<AtomicU64>, cancelled: Arc<AtomicBool>, scan_done: Arc<AtomicBool>) {
    let user = format!("{}:\\Users", drive);
    let programs = format!("{}:\\Program Files", drive);
    let programs_x86 = format!("{}:\\Program Files (x86)", drive);
    let windows = format!("{}:\\Windows", drive);
    let root = format!("{}:\\", drive);

    let skip_dirs: Vec<String> = vec![
        format!("{}:\\System Volume Information", drive).to_lowercase(),
        format!("{}:\\$Recycle.Bin", drive).to_lowercase(),
        format!("{}:\\Windows\\SoftwareDistribution\\Download", drive).to_lowercase(),
        format!("{}:\\Windows\\Temp", drive).to_lowercase(),
        format!("{}:\\Windows\\Prefetch", drive).to_lowercase(),
    ];

    if std::path::Path::new(&user).exists() {
        scan_dir(&user, None, &thresholds, &cats, &files, &dirs_scanned, &cancelled);
    }
    if std::path::Path::new(&programs).exists() {
        scan_dir(&programs, Some(5), &thresholds, &cats, &files, &dirs_scanned, &cancelled);
    }
    if std::path::Path::new(&programs_x86).exists() {
        scan_dir(&programs_x86, Some(5), &thresholds, &cats, &files, &dirs_scanned, &cancelled);
    }
    if std::path::Path::new(&windows).exists() {
        scan_dir(&windows, Some(6), &thresholds, &cats, &files, &dirs_scanned, &cancelled);
    }
    match std::fs::read_dir(&root) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if cancelled.load(Ordering::Relaxed) { return; }
                let path = entry.path();
                let lower = path.to_string_lossy().to_lowercase();
                if skip_dirs.iter().any(|s| lower.starts_with(s)) { continue; }
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if lower.starts_with(&format!("{}:\\program files", drive).to_lowercase())
                        || lower.starts_with(&format!("{}:\\windows", drive).to_lowercase())
                        || lower.starts_with(&format!("{}:\\users", drive).to_lowercase())
                        || lower.starts_with(&format!("{}:\\system volume information", drive).to_lowercase())
                        || lower.starts_with(&format!("{}:\\$recycle.bin", drive).to_lowercase())
                    { continue; }
                    scan_dir(&path.to_string_lossy(), None, &thresholds, &cats, &files, &dirs_scanned, &cancelled);
                } else if let Ok(meta) = entry.metadata() {
                    let size = meta.len();
                    let cat = classify_by_ext(&lower);
                    let mut cats_guard = cats.lock().unwrap();
                    match cat {
                        0 => cats_guard.documents = cats_guard.documents.saturating_add(size),
                        1 => cats_guard.pictures = cats_guard.pictures.saturating_add(size),
                        2 => cats_guard.audio = cats_guard.audio.saturating_add(size),
                        3 => cats_guard.video = cats_guard.video.saturating_add(size),
                        _ => cats_guard.other = cats_guard.other.saturating_add(size),
                    }
                    drop(cats_guard);
                    if size >= thresholds.get(cat) {
                        let mut files_guard = files.lock().unwrap();
                        let mtime = meta.modified().ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs());
                        files_guard.push(LargeFile { path: path.to_string_lossy().to_string(), size, category: cat, mtime });
                        files_guard.sort_by(|a, b| b.size.cmp(&a.size));
                        if files_guard.len() > 50000 { files_guard.truncate(50000); }
                    }
                }
            }
        }
        Err(_) => {}
    }
    scan_done.store(true, Ordering::Relaxed);
}

enum Mode {
    Normal,
    Analysis { drive: char, cats: Arc<Mutex<Categories>>, files: Arc<Mutex<Vec<LargeFile>>>, dirs_scanned: Arc<AtomicU64>, cancelled: Arc<AtomicBool>, scan_done: Arc<AtomicBool>, selected_category: usize },
    CategoryFiles { drive: char, cat_index: u8, cat_name: String, cat_color: Color, parent_cats: Arc<Mutex<Categories>>, parent_files: Arc<Mutex<Vec<LargeFile>>>, parent_dirs_scanned: Arc<AtomicU64>, parent_cancelled: Arc<AtomicBool>, parent_scan_done: Arc<AtomicBool>, files: Vec<LargeFile>, sort_by: SortField, sort_desc: bool, selected: usize, detail: Option<usize> },
    Detail { drive: char, label: String, fs_type: String, total: u64, free: u64 },
}

struct App {
    config: Config,
    show_help: bool,
    mode: Mode,
    vols: Vec<VolumeInfo>,
    selected: usize,
    cpu: f64,
    mem_used: u64,
    mem_total: u64,
    mem_pct: f64,
    swap_used: u64,
    swap_total: u64,
    swap_pct: f64,
    disk_read_rate: f64,
    disk_write_rate: f64,
    disk_total: u64,
    disk_free: u64,
    interval: u64,
    prev_disk_read: u64,
    prev_disk_write: u64,
    prev_cpu: (u64, u64, u64),
    scan_cache: HashMap<char, (Categories, Vec<LargeFile>, u64)>,
}

impl App {
    fn new(interval: u64) -> Self {
        let prev_cpu = system_cpu_snapshot();
        let prev_io = current_process_io();
        Self {
            config: Config::load(),
            show_help: false,
            mode: Mode::Normal,
            vols: get_volumes(),
            selected: 0,
            cpu: 0.0,
            mem_used: 0, mem_total: 0, mem_pct: 0.0,
            swap_used: 0, swap_total: 0, swap_pct: 0.0,
            disk_read_rate: 0.0, disk_write_rate: 0.0,
            disk_total: 0, disk_free: 0,
            interval,
            prev_disk_read: prev_io.0,
            prev_disk_write: prev_io.1,
            prev_cpu,
            scan_cache: HashMap::new(),
        }
    }

    fn refresh(&mut self) {
        self.vols = get_volumes();
        if self.selected >= self.vols.len() {
            self.selected = self.vols.len().saturating_sub(1);
        }

        let (total_phys, avail_phys, total_page, avail_page, _) = self.get_memory_info();
        self.mem_total = total_phys;
        self.mem_used = total_phys.saturating_sub(avail_phys);
        self.mem_pct = if total_phys > 0 { self.mem_used as f64 / total_phys as f64 * 100.0 } else { 0.0 };
        self.swap_total = total_page;
        self.swap_used = total_page.saturating_sub(avail_page);
        self.swap_pct = if total_page > 0 { self.swap_used as f64 / total_page as f64 * 100.0 } else { 0.0 };

        let cur_cpu = system_cpu_snapshot();
        let idle_delta = cur_cpu.0.saturating_sub(self.prev_cpu.0);
        let total_delta = (cur_cpu.1 + cur_cpu.2).saturating_sub(self.prev_cpu.1 + self.prev_cpu.2);
        self.cpu = if total_delta > 0 {
            (1.0 - idle_delta as f64 / total_delta as f64) * 100.0
        } else { 0.0 };
        self.prev_cpu = cur_cpu;

        let cur_io = current_process_io();
        let dt = self.interval as f64;
        self.disk_read_rate = if self.prev_disk_read > 0 && dt > 0.0 {
            cur_io.0.saturating_sub(self.prev_disk_read) as f64 / dt
        } else { 0.0 };
        self.disk_write_rate = if self.prev_disk_write > 0 && dt > 0.0 {
            cur_io.1.saturating_sub(self.prev_disk_write) as f64 / dt
        } else { 0.0 };
        self.prev_disk_read = cur_io.0;
        self.prev_disk_write = cur_io.1;

        let (dt, df) = disk_collect_all();
        self.disk_total = dt;
        self.disk_free = df;
    }

    fn get_memory_info(&self) -> (u64, u64, u64, u64, u32) {
        let mut state = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as DWORD,
            dwMemoryLoad: 0, ullTotalPhys: 0, ullAvailPhys: 0,
            ullTotalPageFile: 0, ullAvailPageFile: 0,
            ullTotalVirtual: 0, ullAvailVirtual: 0, ullAvailExtendedVirtual: 0,
        };
        unsafe { GlobalMemoryStatusEx(&mut state); }
        (state.ullTotalPhys, state.ullAvailPhys, state.ullTotalPageFile, state.ullAvailPageFile, state.dwMemoryLoad)
    }

    fn disk_used(&self) -> u64 { self.disk_total.saturating_sub(self.disk_free) }
}

fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let cfg = &app.config;
    let p = &cfg.palette;
    let s = &cfg.spacing;
    let L = &cfg.labels;

    let title_pad_l = " ".repeat(s.title.left as usize);
    let title_pad_r = " ".repeat(s.title.right as usize);
    let block = Block::default()
        .title(format!("{}dfree{}", title_pad_l, title_pad_r))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .style(Style::default().fg(p.title_border));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(5),
        Constraint::Min(3),
        Constraint::Length(1),
    ]).split(inner);

    let sh = &s.sys_header;
    let lvg = " ".repeat(sh.label_value_gap as usize);
    let blank_line = if sh.line_gap > 0 { vec![Line::from(Span::raw(""))] } else { vec![] };

    let mut sys_lines = vec![
        Line::from(vec![
            Span::styled(format!("{}:", L.cpu), Style::default().fg(p.label_cpu)),
            Span::raw(&lvg),
            Span::styled(format!("{:>5.1}%", app.cpu), Style::default().fg(usage_color(app.cpu, p)).bold()),
            Span::raw(&sh.separator),
            Span::styled(format!("{}:", L.memory), Style::default().fg(p.label_memory)),
            Span::raw(&lvg),
            Span::styled(format!("{:>4}/{:>3}", format_size(app.mem_used), format_size(app.mem_total)), Style::default().fg(usage_color(app.mem_pct, p)).bold()),
            Span::raw(&lvg),
            Span::styled(format!("({:.1}%)", app.mem_pct), Style::default().fg(usage_color(app.mem_pct, p))),
            Span::raw(&sh.separator),
            Span::styled(format!("{}:", L.swap), Style::default().fg(p.label_swap)),
            Span::raw(&lvg),
            Span::styled(format!("{:>4}/{:>3}", format_size(app.swap_used), format_size(app.swap_total)), Style::default().fg(usage_color(app.swap_pct, p)).bold()),
            Span::raw(&lvg),
            Span::styled(format!("({:.1}%)", app.swap_pct), Style::default().fg(usage_color(app.swap_pct, p))),
        ]),
    ];
    sys_lines.extend(blank_line);
    sys_lines.push(Line::from(vec![
        Span::styled(&L.disk_io, Style::default().fg(p.label_disk_io)),
        Span::raw(" R:"),
        Span::styled(format_rate(app.disk_read_rate), Style::default().fg(p.gauge_ok)),
        Span::raw("  W:"),
        Span::styled(format_rate(app.disk_write_rate), Style::default().fg(p.gauge_warn)),
        Span::raw(&sh.separator),
        Span::styled(&L.total, Style::default().fg(p.label_total)),
        Span::raw(&lvg),
        Span::styled(format!("{}", format_size(app.disk_used())), Style::default().fg(p.text_primary)),
        Span::raw("/"),
        Span::styled(format!("{}", format_size(app.disk_total)), Style::default().fg(p.text_highlight)),
        Span::raw(&lvg),
        Span::styled(format!("({:.1}%)", if app.disk_total > 0 { app.disk_used() as f64 / app.disk_total as f64 * 100.0 } else { 0.0 }), Style::default().fg(p.text_secondary)),
    ]));
    f.render_widget(Paragraph::new(sys_lines), chunks[0]);

    let selected_idx = app.selected;
    let vs = &s.volume_table;
    let row_pad = " ".repeat(vs.row_prefix as usize);
    let mut disk_groups: Vec<(u32, Vec<usize>, u64)> = Vec::new();
    let mut unknown: Vec<usize> = Vec::new();
    for (i, v) in app.vols.iter().enumerate() {
        match v.disk_number {
            Some(d) => {
                if let Some(g) = disk_groups.iter_mut().find(|g| g.0 == d) {
                    g.1.push(i);
                    g.2 += v.total;
                } else {
                    disk_groups.push((d, vec![i], v.total));
                }
            }
            None => { unknown.push(i); }
        }
    }
    disk_groups.sort_by_key(|g| g.0);

    let mut table_rows: Vec<Row> = Vec::new();
    let mut sel_row: Option<usize> = None;
    for (disk, indices, total_cap) in &disk_groups {
        table_rows.push(Row::new(vec![Cell::from(Line::from(Span::styled(
            format!("{} Disk {}  ({})", row_pad, disk, format_size(*total_cap)),
            Style::default().fg(p.table_header),
        )))]));
        for &vi in indices {
            let v = &app.vols[vi];
            let used = v.total.saturating_sub(v.free);
            let pct = if v.total > 0 { used as f64 / v.total as f64 * 100.0 } else { 0.0 };
            let label = if v.label.is_empty() { format!("{}:", v.letter) } else { format!("{}: ({})", v.letter, v.label) };
            let gauge_str = format!("  {}/{} ({:.1}%)", format_size(used), format_size(v.total), pct);
            let is_sel = vi == selected_idx;
            let prefix = if is_sel { "▸ " } else { "  " };
            let style = if is_sel { Style::default().fg(p.text_highlight).bg(p.table_selected_bg) } else { Style::default().fg(p.table_row) };
            table_rows.push(Row::new(vec![Cell::from(Line::from(Span::styled(
                format!("{}{}  {}{}", row_pad, prefix, label, gauge_str), style,
            )))]));
            if is_sel { sel_row = Some(table_rows.len() - 1); }
        }
    }
    for &vi in &unknown {
        let v = &app.vols[vi];
        let used = v.total.saturating_sub(v.free);
        let pct = if v.total > 0 { used as f64 / v.total as f64 * 100.0 } else { 0.0 };
        let label = if v.label.is_empty() { format!("{}:", v.letter) } else { format!("{}: ({})", v.letter, v.label) };
        let gauge_str = format!("  {}/{} ({:.1}%)", format_size(used), format_size(v.total), pct);
        let is_sel = vi == selected_idx;
        let prefix = if is_sel { "▸ " } else { "  " };
        let style = if is_sel { Style::default().fg(p.text_highlight).bg(p.table_selected_bg) } else { Style::default().fg(p.table_row) };
        table_rows.push(Row::new(vec![Cell::from(Line::from(Span::styled(
            format!("{}{}  {}{}", row_pad, prefix, label, gauge_str), style,
        )))]));
        if is_sel { sel_row = Some(table_rows.len() - 1); }
    }
    let vol_table = Table::new(table_rows, [Constraint::Fill(1)])
        .row_highlight_style(Style::default().fg(p.text_highlight).bg(p.table_selected_bg))
        .highlight_symbol("");
    if let Some(row) = sel_row {
        let mut state = TableState::new().with_selected(Some(row));
        f.render_stateful_widget(vol_table, chunks[1], &mut state);
    } else {
        f.render_widget(vol_table, chunks[1]);
    }

    let ft = &s.footer;
    let f_pad = " ".repeat(ft.prefix as usize);
    let kd_gap = " ".repeat(ft.key_desc_gap as usize);
    let grp_gap = " ".repeat(ft.group_gap as usize);
    let footer = Paragraph::new(Line::from(vec![
        Span::raw(format!("{}{}s{}", f_pad, app.interval, grp_gap)),
        Span::styled("[↑↓]", Style::default().fg(p.key_binding)),
        Span::raw(&kd_gap),
        Span::styled("选择", Style::default().fg(p.key_desc)),
        Span::raw(&grp_gap),
        Span::styled("[Enter]", Style::default().fg(p.key_binding)),
        Span::raw(&kd_gap),
        Span::styled("分析", Style::default().fg(p.key_desc)),
        Span::raw(&grp_gap),
        Span::styled("[d]", Style::default().fg(p.key_binding)),
        Span::raw(&kd_gap),
        Span::styled("详情", Style::default().fg(p.key_desc)),
        Span::raw(&grp_gap),
        Span::styled("[m]", Style::default().fg(p.key_binding)),
        Span::raw(&kd_gap),
        Span::styled("磁盘管理", Style::default().fg(p.key_desc)),
        Span::raw(&grp_gap),
        Span::styled("[?]", Style::default().fg(p.key_binding)),
        Span::raw(&kd_gap),
        Span::styled("帮助", Style::default().fg(p.key_desc)),
        Span::raw(&grp_gap),
        Span::styled("[q]", Style::default().fg(p.key_binding)),
        Span::raw(&kd_gap),
        Span::styled("退出", Style::default().fg(p.key_desc)),
    ]));
    f.render_widget(footer, chunks[2]);

    match &app.mode {
        Mode::Normal => {}
        Mode::Analysis { drive, cats, files: _, dirs_scanned, cancelled: _, scan_done, selected_category } => {
            render_analysis_popup(f, area, *drive, cats, dirs_scanned, scan_done, *selected_category, cfg);
        }
        Mode::CategoryFiles { drive, cat_index: _, cat_name, cat_color, parent_cats: _, parent_files: _, parent_dirs_scanned: _, parent_cancelled: _, parent_scan_done: _, files, sort_by, sort_desc, selected, detail } => {
            render_category_files_popup(f, area, *drive, cat_name, *cat_color, files, sort_by, *sort_desc, *selected, *detail, cfg);
        }
        Mode::Detail { drive, label, fs_type, total, free } => {
            render_detail_popup(f, area, *drive, label, fs_type, *total, *free, cfg);
        }
    }

    if app.show_help {
        render_help_popup(f, area, cfg);
    }
}

fn centered_rect(r: Rect, pct_x: u16, pct_y: u16) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length((r.height * pct_y / 100).min(r.height.saturating_sub(2))),
        Constraint::Fill(1),
    ]).split(r);
    Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length((r.width * pct_x / 100).min(r.width.saturating_sub(4))),
        Constraint::Fill(1),
    ]).split(popup_layout[1])[1]
}

fn render_analysis_popup(f: &mut Frame, area: Rect, drive: char, cats: &Arc<Mutex<Categories>>, dirs_scanned: &Arc<AtomicU64>, scan_done: &Arc<AtomicBool>, selected_category: usize, cfg: &Config) {
    let p = &cfg.palette;
    let s = &cfg.spacing;
    let L = &cfg.labels;
    let popup = centered_rect(area, 60, 55);
    f.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .style(Style::default().bg(p.popup_bg).fg(p.title_text))
        .border_style(Style::default().fg(p.popup_bg));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let cl = s.popup.content_left;
    let ct = s.popup.content_top;
    let cb = s.popup.content_bottom;
    let fg = s.popup.footer_gap_top;

    let cats_guard = cats.lock().unwrap();
    let scanned = dirs_scanned.load(Ordering::Relaxed);
    let total_size = cats_guard.documents + cats_guard.pictures + cats_guard.audio + cats_guard.video + cats_guard.other + cats_guard.applications + cats_guard.system + cats_guard.cache;
    let done = scan_done.load(Ordering::Relaxed);

    let top_pad = if ct > 0 { vec![Constraint::Length(ct)] } else { vec![] };
    let bot_pad = if cb > 0 { vec![Constraint::Length(cb)] } else { vec![] };
    let gauge_h = s.analysis.gauge_height;
    let gauge_gap = s.analysis.gauge_gap;
    let mut gauge_layout = vec![Constraint::Length(gauge_h)];
    if gauge_gap > 0 { gauge_layout.push(Constraint::Length(gauge_gap)); }
    if s.analysis.gauge_margin_top > 0 { gauge_layout.insert(0, Constraint::Length(s.analysis.gauge_margin_top)); }
    if s.analysis.gauge_margin_bottom > 0 { gauge_layout.push(Constraint::Length(s.analysis.gauge_margin_bottom)); }
    let gauge_layout_len = gauge_layout.len();

    let title_chunks: [Constraint; 2] = [Constraint::Length(1), Constraint::Length(1)];
    let mut inner_chunks = vec![];
    inner_chunks.extend_from_slice(&title_chunks);
    inner_chunks.extend(top_pad);
    let gauge_start_idx = inner_chunks.len();
    inner_chunks.extend(gauge_layout);
    inner_chunks.push(Constraint::Fill(1));
    inner_chunks.extend(bot_pad);
    let chunks = Layout::vertical(inner_chunks).split(inner);

    let title = Paragraph::new(Line::from(Span::styled(
        format!("{}: {}", drive, L.storage_analysis),
        Style::default().fg(p.title_text).bold(),
    ))).alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let gauge_pct = if done { 100 } else if scanned > 0 { (scanned % 101) as u16 } else { 0 };
    let gauge_label = if done {
        format!(" {} — {} {} ", L.scan_complete, scanned, L.dirs_scanned)
    } else if scanned == 0 {
        format!(" {} ", L.scan_starting)
    } else {
        format!(" {} — {} {} ", L.scan_scanning, scanned, L.dirs_scanned)
    };
    let gauge_color = if done { p.gauge_done } else { p.gauge_scanning };
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color))
        .percent(gauge_pct)
        .label(gauge_label);
    let gauge_area = chunks[gauge_start_idx];
    let gauge_inner = Rect { x: gauge_area.x + cl, width: gauge_area.width.saturating_sub(cl * 2), ..gauge_area };
    f.render_widget(gauge, gauge_inner);

    let content_idx = gauge_start_idx + gauge_layout_len;

    if total_size == 0 && scanned == 0 && !done {
        let lines = Paragraph::new(Line::from(Span::styled(" Waiting for data...", Style::default().fg(p.text_secondary))));
        f.render_widget(lines, chunks[content_idx]);
        return;
    }

    let total_gb = total_size as f64 / 1_073_741_824.0;
    let cat_names = [&L.documents, &L.pictures, &L.audio, &L.video, &L.other, &L.applications, &L.system, &L.cache];
    let cat_sizes = [cats_guard.documents, cats_guard.pictures, cats_guard.audio, cats_guard.video, cats_guard.other, cats_guard.applications, cats_guard.system, cats_guard.cache];
    let cat_colors = [p.cat_documents, p.cat_pictures, p.cat_audio, p.cat_video, p.cat_other, p.cat_applications, p.cat_system, p.cat_cache];

    let content_parts = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ]).split(chunks[content_idx]);

    f.render_widget(Paragraph::new(Line::from(Span::styled(
        format!(" {} {:.1} GiB", L.cat_total, total_gb),
        Style::default().fg(p.text_highlight).bold(),
    ))), content_parts[0]);

    let rows: Vec<Row> = (0..8).map(|i| {
        let pct = if total_size > 0 { cat_sizes[i] as f64 / total_size as f64 * 100.0 } else { 0.0 };
        let is_sel = i == selected_category;
        let row_style = if is_sel { Style::default().fg(p.text_highlight).bg(p.table_selected_bg) } else { Style::default() };
        Row::new(vec![
            Cell::from(Line::from(Span::styled(format!(" {:<22}", cat_names[i]), Style::default().fg(cat_colors[i])))),
            Cell::from(Line::from(Span::styled(format!("{:>10}", format_size(cat_sizes[i])), Style::default().fg(if is_sel { p.text_highlight } else { p.text_primary })))),
            Cell::from(Line::from(Span::styled(format!("{:>5.1}%", pct), Style::default().fg(cat_colors[i])))),
        ]).style(row_style)
    }).collect();

    let header = Row::new(vec![
        Cell::from(Line::from(Span::styled(format!(" {:<22}", L.category), Style::default().fg(p.text_secondary)))),
        Cell::from(Line::from(Span::styled(format!("{:>10}", L.size), Style::default().fg(p.text_secondary)))),
        Cell::from(Line::from(Span::styled(format!("{:>5}", L.pct), Style::default().fg(p.text_secondary)))),
    ]);

    let table = Table::new(rows, [Constraint::Length(24), Constraint::Length(12), Constraint::Fill(1)])
        .header(header);
    f.render_widget(table, content_parts[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("[↑↓]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled("选择", Style::default().fg(p.key_desc)),
        Span::raw("  "),
        Span::styled("[Enter]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled("展开", Style::default().fg(p.key_desc)),
        Span::raw("  "),
        Span::styled("[Esc]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled(&L.back, Style::default().fg(p.key_desc)),
        Span::raw("  "),
        Span::styled("[?]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled("帮助", Style::default().fg(p.key_desc)),
    ]));
    f.render_widget(footer, content_parts[3]);
}

fn format_mtime(ts: Option<u64>) -> String {
    match ts {
        Some(secs) => {
            let days = secs / 86400;
            let mut y = 1970i64;
            let mut d = days as i64;
            loop {
                let leap = y % 400 == 0 || (y % 4 == 0 && y % 100 != 0);
                let diy = if leap { 366 } else { 365 };
                if d < diy { break; }
                d -= diy; y += 1;
            }
            let leap = y % 400 == 0 || (y % 4 == 0 && y % 100 != 0);
            let mdays = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            let mut m = 0;
            for md in &mdays { if d < *md { break; } d -= *md; m += 1; }
            format!("{:04}-{:02}-{:02}", y, m + 1, d + 1)
        }
        None => "--".to_string(),
    }
}

fn sort_files(files: &mut Vec<LargeFile>, by: &SortField, desc: bool) {
    match by {
        SortField::Size => files.sort_by(|a, b| if desc { b.size.cmp(&a.size) } else { a.size.cmp(&b.size) }),
        SortField::Name => files.sort_by(|a, b| if desc { b.path.cmp(&a.path) } else { a.path.cmp(&b.path) }),
        SortField::Time => files.sort_by(|a, b| {
            let ta = a.mtime.unwrap_or(0);
            let tb = b.mtime.unwrap_or(0);
            if desc { tb.cmp(&ta) } else { ta.cmp(&tb) }
        }),
    }
}

fn render_category_files_popup(f: &mut Frame, area: Rect, drive: char, cat_name: &str, cat_color: Color, files: &Vec<LargeFile>, sort_by: &SortField, sort_desc: bool, selected: usize, detail: Option<usize>, cfg: &Config) {
    let p = &cfg.palette;
    let L = &cfg.labels;
    let popup = centered_rect(area, 75, 75);
    f.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .style(Style::default().bg(p.popup_bg).fg(p.title_text))
        .border_style(Style::default().fg(p.popup_bg));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let table_rows: usize = 25;
    let table_header: usize = 1;
    let table_total = (table_rows + table_header) as u16;
    let chunks = Layout::vertical([
        Constraint::Length(1), Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(table_total),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ]).split(inner);

    let title = Paragraph::new(Line::from(Span::styled(
        format!("{}: {}", drive, cat_name),
        Style::default().fg(cat_color).bold(),
    ))).alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let sort_indicator = if sort_desc { "↓" } else { "↑" };
    let sort_label = match sort_by {
        SortField::Size => format!("{} {}", L.sort_by_size, sort_indicator),
        SortField::Name => format!("{} {}", L.sort_by_name, sort_indicator),
        SortField::Time => format!("{} {}", L.sort_by_time, sort_indicator),
    };
    let total = files.len();
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("[s]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled(&L.sort_by_size, Style::default().fg(p.key_desc)),
        Span::raw("  "),
        Span::styled("[n]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled(&L.sort_by_name, Style::default().fg(p.key_desc)),
        Span::raw("  "),
        Span::styled("[t]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled(&L.sort_by_time, Style::default().fg(p.key_desc)),
        Span::raw("  │  "),
        Span::styled(&sort_label, Style::default().fg(p.text_secondary)),
        Span::raw("  │  "),
        Span::styled(format!("{}/{}", selected + 1, total), Style::default().fg(p.text_secondary)),
    ])), chunks[2]);

    let table_area = chunks[3];
    let path_col_w = table_area.width.saturating_sub(26) as usize;
    let path_max = if path_col_w > 4 { path_col_w } else { 30 };
    let rows: Vec<Row> = files.iter().enumerate().map(|(i, f)| {
        let is_sel = i == selected;
        let row_style = if is_sel { Style::default().fg(p.text_highlight).bg(p.table_selected_bg) } else { Style::default().fg(p.text_primary) };
        let num_color = if is_sel { p.text_highlight } else { p.text_secondary };
        let fg_s = if is_sel { p.text_highlight } else { p.gauge_warn };
        let path_s = if f.path.chars().count() > path_max {
            let c: Vec<char> = f.path.chars().collect();
            let start = c.len().saturating_sub(path_max.saturating_sub(3));
            let tail: String = c[start..].iter().collect();
            format!("...{}", tail)
        } else { f.path.clone() };
        Row::new(vec![
            Cell::from(Line::from(Span::styled(format!("{:>3}", i + 1), Style::default().fg(num_color)))),
            Cell::from(Line::from(Span::styled(format!("{:>10}", format_size(f.size)), Style::default().fg(fg_s)))),
            Cell::from(Line::from(Span::styled(format_mtime(f.mtime), Style::default().fg(if is_sel { p.text_highlight } else { p.text_secondary })))),
            Cell::from(Line::from(Span::styled(path_s, Style::default().fg(if is_sel { p.text_highlight } else { p.text_primary })))),
        ]).style(row_style)
    }).collect();

    let header = Row::new(vec![
        Cell::from(Line::from(Span::styled(format!("{:>3}", L.num), Style::default().fg(p.text_secondary)))),
        Cell::from(Line::from(Span::styled(format!("{:>10}", L.file_size), Style::default().fg(p.text_secondary)))),
        Cell::from(Line::from(Span::styled(&L.modified, Style::default().fg(p.text_secondary)))),
        Cell::from(Line::from(Span::styled(&L.path, Style::default().fg(p.text_secondary)))),
    ]);

    let table = Table::new(rows, [Constraint::Length(3), Constraint::Length(10), Constraint::Length(10), Constraint::Fill(1)])
        .header(header)
        .row_highlight_style(Style::default().fg(p.text_highlight).bg(p.table_selected_bg));
    let mut state = TableState::new().with_selected(Some(selected));
    f.render_stateful_widget(table, table_area, &mut state);

    let footer = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("[↑↓/PgUp/PgDn]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled("滚动", Style::default().fg(p.key_desc)),
        Span::raw("  "),
        Span::styled("[s]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled(&L.sort_by_size, Style::default().fg(p.key_desc)),
        Span::raw("  "),
        Span::styled("[n]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled(&L.sort_by_name, Style::default().fg(p.key_desc)),
        Span::raw("  "),
        Span::styled("[t]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled(&L.sort_by_time, Style::default().fg(p.key_desc)),
        Span::raw("  "),
        Span::styled("[Enter]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled("详情", Style::default().fg(p.key_desc)),
        Span::raw("  "),
        Span::styled("[Esc]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled(&L.back, Style::default().fg(p.key_desc)),
    ]));
    f.render_widget(footer, chunks[5]);

    if let Some(idx) = detail {
        if idx < files.len() {
            render_file_detail_overlay(f, popup, &files[idx], cfg);
        }
    }
}

fn render_file_detail_overlay(f: &mut Frame, parent_area: Rect, file: &LargeFile, cfg: &Config) {
    let p = &cfg.palette;
    let L = &cfg.labels;
    let cat_names = [&L.documents, &L.pictures, &L.audio, &L.video, &L.other, &L.applications, &L.system, &L.cache];
    let cat_colors_arr = [
        p.cat_documents, p.cat_pictures, p.cat_audio, p.cat_video, p.cat_other, p.cat_applications, p.cat_system, p.cat_cache,
    ];
    let cat_idx = file.category.min(7) as usize;
    let cat_display = format!("{} {}", cat_names[cat_idx], L.cat_label);

    let overlay_w = parent_area.width.min(60).max(40);
    let line_w = (overlay_w - 2) as usize;
    let prefix_w = format!("{}: ", L.path_label).chars().count();
    let path_chars: Vec<char> = file.path.chars().collect();
    let path_lines_count = if path_chars.is_empty() { 1 } else {
        let first = line_w.saturating_sub(prefix_w);
        if first >= path_chars.len() { 1 }
        else { 2 + (path_chars.len() - first + line_w - 2 - 1) / (line_w - 2) }
    };
    let overlay_h = (5 + path_lines_count as u16).min(parent_area.height.saturating_sub(2));
    let overlay_x = parent_area.x + (parent_area.width.saturating_sub(overlay_w)) / 2;
    let overlay_y = parent_area.y + (parent_area.height.saturating_sub(overlay_h)) / 2;
    let overlay = Rect { x: overlay_x, y: overlay_y, width: overlay_w, height: overlay_h };

    f.render_widget(Clear, overlay);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .style(Style::default().bg(p.popup_bg).fg(p.title_text))
        .border_style(Style::default().fg(p.popup_bg));
    let inner = block.inner(overlay);
    f.render_widget(block, overlay);

    let label_prefix = format!("{}: ", L.path_label);
    let prefix_w = label_prefix.chars().count();
    let line_w = inner.width as usize;
    let path_chars: Vec<char> = file.path.chars().collect();
    let mut path_lines = vec![];
    let mut pi = 0;
    while pi < path_chars.len() {
        let max = if pi == 0 { line_w.saturating_sub(prefix_w) } else { line_w - 2 };
        let end = (pi + max).min(path_chars.len());
        let chunk: String = path_chars[pi..end].iter().collect();
        if pi == 0 {
            path_lines.push(Line::from(vec![Span::styled(&label_prefix, Style::default().fg(p.text_secondary)), Span::styled(chunk, Style::default().fg(p.text_primary))]));
        } else {
            path_lines.push(Line::from(vec![Span::raw("  "), Span::styled(chunk, Style::default().fg(p.text_secondary))]));
        }
        pi = end;
    }

    let mut lines = vec![
        Line::from(Span::styled(&L.file_detail_title, Style::default().fg(p.title_text).bold())),
        Line::from(Span::raw("")),
    ];
    lines.extend(path_lines);
    lines.extend(vec![
        Line::from(vec![Span::styled(&L.size_label, Style::default().fg(p.text_secondary)), Span::raw(": "), Span::styled(format_size(file.size), Style::default().fg(p.gauge_warn))]),
        Line::from(vec![Span::styled(&L.mtime_label, Style::default().fg(p.text_secondary)), Span::raw(": "), Span::styled(format_mtime(file.mtime), Style::default().fg(p.text_primary))]),
        Line::from(vec![Span::styled(&L.cat_label, Style::default().fg(p.text_secondary)), Span::raw(": "), Span::styled(cat_display, Style::default().fg(cat_colors_arr[cat_idx]))]),
        Line::from(Span::raw("")),
        Line::from(vec![Span::styled("[Esc]", Style::default().fg(p.key_binding)), Span::raw(" "), Span::styled(&L.back, Style::default().fg(p.key_desc))]),
    ]);
    f.render_widget(Paragraph::new(lines), inner);
}

fn render_detail_popup(f: &mut Frame, area: Rect, drive: char, label: &str, fs_type: &str, total: u64, free: u64, cfg: &Config) {
    let p = &cfg.palette;
    let s = &cfg.spacing;
    let L = &cfg.labels;
    let popup = centered_rect(area, 50, 35);
    f.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .style(Style::default().bg(p.popup_bg).fg(p.title_text))
        .border_style(Style::default().fg(p.popup_bg));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let cl = s.popup.content_left;
    let content_l = " ".repeat(cl as usize);
    let dt = &s.detail;

    let used = total.saturating_sub(free);
    let pct_v = if total > 0 { used as f64 / total as f64 * 100.0 } else { 0.0 };
    let gauge_color = usage_color(pct_v, p);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color))
        .percent(pct_v as u16)
        .label(format!("{} / {}  ({:.1}%)", format_size(used), format_size(total), pct_v));

    let vol_display = if label.is_empty() { format!("{} --", drive) } else { label.to_string() };
    let mut lines = vec![
        Line::from(vec![Span::raw(&content_l), Span::styled(&L.volume_label, Style::default().fg(p.text_secondary)), Span::raw(": "), Span::styled(vol_display, Style::default().fg(p.text_primary))]),
        Line::from(vec![Span::raw(&content_l), Span::styled(&L.file_system, Style::default().fg(p.text_secondary)), Span::raw(": "), Span::styled(fs_type, Style::default().fg(p.text_primary))]),
        Line::from(vec![Span::raw(&content_l), Span::styled(&L.capacity, Style::default().fg(p.text_secondary)), Span::raw(": "), Span::styled(format_size(total), Style::default().fg(p.text_primary))]),
        Line::from(vec![Span::raw(&content_l), Span::styled(&L.available, Style::default().fg(p.text_secondary)), Span::raw(": "), Span::styled(format_size(free), Style::default().fg(p.gauge_ok))]),
    ];
    if dt.line_gap > 0 {
        for _ in 0..dt.line_gap { lines.push(Line::from(Span::raw(""))); }
    }

    let detail_height = lines.len() as u16;
    let gauge_margin_top = dt.gauge_margin_top;
    let gauge_margin_bottom = dt.gauge_margin_bottom;
    let mut gauge_layout = vec![Constraint::Length(1)];
    if gauge_margin_top > 0 { gauge_layout.insert(0, Constraint::Length(gauge_margin_top)); }
    if gauge_margin_bottom > 0 { gauge_layout.push(Constraint::Length(gauge_margin_bottom)); }

    let mut vchunks = vec![
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(detail_height),
    ];
    vchunks.extend(gauge_layout);
    vchunks.push(Constraint::Length(1));
    vchunks.push(Constraint::Fill(1));
    vchunks.push(Constraint::Length(1));

    let chunks = Layout::vertical(vchunks).split(inner);

    let title = Paragraph::new(Line::from(Span::styled(
        format!("{}: {}", drive, L.volume_details),
        Style::default().fg(p.title_text).bold(),
    ))).alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let gauge_idx = 3 + if gauge_margin_top > 0 { 1 } else { 0 };

    f.render_widget(Paragraph::new(lines), chunks[2]);
    let garea = chunks[gauge_idx];
    let ginner = Rect { x: garea.x + cl, width: garea.width.saturating_sub(cl * 2), ..garea };
    f.render_widget(gauge, ginner);
    let footer_area = chunks[chunks.len() - 1];
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::raw(&content_l),
        Span::styled("[Esc]", Style::default().fg(p.key_binding)),
        Span::raw(" "),
        Span::styled(&L.back, Style::default().fg(p.key_desc)),
    ])), footer_area);
}

fn render_help_popup(f: &mut Frame, area: Rect, cfg: &Config) {
    let p = &cfg.palette;
    let L = &cfg.labels;
    let popup = centered_rect(area, 65, 75);
    f.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .style(Style::default().bg(p.popup_bg).fg(p.title_text))
        .border_style(Style::default().fg(p.popup_bg));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let lines: Vec<Line> = vec![
        Line::from(Span::styled(format!("  {} ", L.help_title), Style::default().fg(p.title_text).bold())),
        Line::from(Span::raw("")),
        Line::from(vec![Span::styled(format!("  {} ", L.help_general), Style::default().fg(p.text_secondary).bold()),]),
        Line::from(Span::raw("    ?              关闭 / 打开帮助")),
        Line::from(Span::raw("    Esc            关闭帮助 / 返回")),
        Line::from(Span::raw("    q              退出程序")),
        Line::from(Span::raw("")),
        Line::from(vec![Span::styled(format!("  {} ", L.help_main), Style::default().fg(p.text_secondary).bold()),]),
        Line::from(Span::raw("    ↑↓ / k/j      选择磁盘")),
        Line::from(Span::raw("    Enter          存储分析")),
        Line::from(Span::raw("    d              卷详情")),
        Line::from(Span::raw("    m              磁盘管理")),
        Line::from(Span::raw("")),
        Line::from(vec![Span::styled(format!("  {} ", L.help_analysis), Style::default().fg(p.text_secondary).bold()),]),
        Line::from(Span::raw("    ↑↓ / k/j      切换分类")),
        Line::from(Span::raw("    Enter          展开分类文件")),
        Line::from(Span::raw("")),
        Line::from(vec![Span::styled(format!("  {} ", L.help_cat_files), Style::default().fg(p.text_secondary).bold()),]),
        Line::from(Span::raw("    ↑↓ / k/j      选择文件")),
        Line::from(Span::raw("    PgUp / PgDn   翻页")),
        Line::from(Span::raw("    Home / End     首 / 尾")),
        Line::from(Span::raw("    s / n / t      排序（大小/名称/时间）")),
        Line::from(Span::raw("    Enter          文件详情")),
        Line::from(Span::raw("")),
        Line::from(vec![Span::styled(format!("  {} ", L.help_detail), Style::default().fg(p.text_secondary).bold()),]),
        Line::from(Span::raw("    Esc            返回主界面")),
        Line::from(Span::raw("")),
        Line::from(Span::raw("    [Esc] 关闭")),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn run(terminal: &mut DefaultTerminal, interval: u64) -> io::Result<()> {
    let mut app = App::new(interval);
    let sleep_dur = Duration::from_secs(interval);
    let mut last_refresh = Instant::now();

    app.refresh();
    let _ = terminal.draw(|f| render(f, &app));

    loop {
        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(event) = event::read() {
                match event {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if key.code == KeyCode::Char('?') {
                            app.show_help = !app.show_help;
                            continue;
                        }
                        if app.show_help && key.code == KeyCode::Esc {
                            app.show_help = false;
                            continue;
                        }
                        match app.mode {
                            Mode::Normal => {
                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Esc => break,
                                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        if app.selected > 0 { app.selected -= 1; }
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        if app.selected + 1 < app.vols.len() { app.selected += 1; }
                                    }
                                    KeyCode::Enter => {
                                        if !app.vols.is_empty() {
                                            let sel = app.selected;
                                            let letter = app.vols[sel].letter;
                                            if let Some((cats_data, files_data, scanned)) = app.scan_cache.remove(&letter) {
                                                let cats = Arc::new(Mutex::new(cats_data));
                                                let files = Arc::new(Mutex::new(files_data));
                                                let dirs_scanned = Arc::new(AtomicU64::new(scanned));
                                                let cancelled = Arc::new(AtomicBool::new(false));
                                                let scan_done = Arc::new(AtomicBool::new(true));
                                                app.mode = Mode::Analysis { drive: letter, cats, files, dirs_scanned, cancelled, scan_done, selected_category: 0 };
                                            } else {
                                                let cats = Arc::new(Mutex::new(Categories::default()));
                                                let files = Arc::new(Mutex::new(Vec::new()));
                                                let dirs_scanned = Arc::new(AtomicU64::new(0));
                                                let cancelled = Arc::new(AtomicBool::new(false));
                                                let scan_done = Arc::new(AtomicBool::new(false));
                                                let c_thresholds = Arc::new(app.config.thresholds.clone());
                                                let c_cats = cats.clone();
                                                let c_files = files.clone();
                                                let c_scanned = dirs_scanned.clone();
                                                let c_cancelled = cancelled.clone();
                                                let c_done = scan_done.clone();
                                                std::thread::spawn(move || {
                                                    scan_drive(letter, c_thresholds, c_cats, c_files, c_scanned, c_cancelled, c_done);
                                                });
                                                app.mode = Mode::Analysis { drive: letter, cats, files, dirs_scanned, cancelled, scan_done, selected_category: 0 };
                                            }
                                        }
                                    }
                                    KeyCode::Char('d') => {
                                        if !app.vols.is_empty() {
                                            let sel = app.selected;
                                            let drive = app.vols[sel].letter;
                                            let label = app.vols[sel].label.clone();
                                            let fs_type = app.vols[sel].fs_type.clone();
                                            let total = app.vols[sel].total;
                                            let free = app.vols[sel].free;
                                            app.mode = Mode::Detail { drive, label, fs_type, total, free };
                                        }
                                    }
                                    KeyCode::Char('m') => {
                                        let _ = std::process::Command::new("mmc.exe").arg("diskmgmt.msc").spawn();
                                    }
                                    _ => {}
                                }
                            }
                            Mode::Analysis { ref drive, ref cats, ref files, ref dirs_scanned, ref cancelled, ref scan_done, ref selected_category } => {
                                match key.code {
                                    KeyCode::Esc => {
                                        cancelled.store(true, Ordering::Relaxed);
                                        let cats_data = cats.lock().unwrap().clone();
                                        let files_data = files.lock().unwrap().clone();
                                        app.scan_cache.insert(*drive, (cats_data, files_data, dirs_scanned.load(Ordering::Relaxed)));
                                        app.mode = Mode::Normal;
                                    }
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        let new = if *selected_category == 7 { 5 }
                                                  else if *selected_category > 0 { selected_category - 1 }
                                                  else { 0 };
                                        if new != *selected_category {
                                            app.mode = Mode::Analysis { drive: *drive, cats: cats.clone(), files: files.clone(), dirs_scanned: dirs_scanned.clone(), cancelled: cancelled.clone(), scan_done: scan_done.clone(), selected_category: new };
                                        }
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        let new = if *selected_category == 5 { 7 }
                                                  else if *selected_category < 7 { selected_category + 1 }
                                                  else { 7 };
                                        if new != *selected_category {
                                            app.mode = Mode::Analysis { drive: *drive, cats: cats.clone(), files: files.clone(), dirs_scanned: dirs_scanned.clone(), cancelled: cancelled.clone(), scan_done: scan_done.clone(), selected_category: new };
                                        }
                                    }
                                    KeyCode::Enter => {
                                        let cat_idx = *selected_category as u8;
                                        let cat_names = [&app.config.labels.documents, &app.config.labels.pictures, &app.config.labels.audio, &app.config.labels.video, &app.config.labels.other, &app.config.labels.applications, &app.config.labels.system, &app.config.labels.cache];
                                        let cat_colors_arr = [app.config.palette.cat_documents, app.config.palette.cat_pictures, app.config.palette.cat_audio, app.config.palette.cat_video, app.config.palette.cat_other, app.config.palette.cat_applications, app.config.palette.cat_system, app.config.palette.cat_cache];
                                        let files_guard = files.lock().unwrap();
                                        let mut filtered: Vec<LargeFile> = files_guard.iter().filter(|f| f.category == cat_idx).cloned().collect();
                                        drop(files_guard);
                                        filtered.sort_by(|a, b| b.size.cmp(&a.size));
                                        app.mode = Mode::CategoryFiles {
                                            drive: *drive,
                                            cat_index: cat_idx,
                                            cat_name: cat_names[*selected_category].clone(),
                                            cat_color: cat_colors_arr[*selected_category],
                                            parent_cats: cats.clone(),
                                            parent_files: files.clone(),
                                            parent_dirs_scanned: dirs_scanned.clone(),
                                            parent_cancelled: cancelled.clone(),
                                            parent_scan_done: scan_done.clone(),
                                            files: filtered,
                                            sort_by: SortField::Size,
                                            sort_desc: true,
                                            selected: 0,
                                            detail: None,
                                        };
                                    }
                                    _ => {}
                                }
                            }
                            Mode::CategoryFiles { ref drive, ref cat_index, ref cat_name, ref cat_color, ref parent_cats, ref parent_files, ref parent_dirs_scanned, ref parent_cancelled, ref parent_scan_done, ref mut files, ref mut sort_by, ref mut sort_desc, ref mut selected, ref mut detail } => {
                                match key.code {
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        if *selected > 0 { *selected -= 1; }
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        if *selected + 1 < files.len() { *selected += 1; }
                                    }
                                    KeyCode::PageDown => {
                                        let page = 20usize;
                                        *selected = (*selected + page).min(files.len().saturating_sub(1));
                                    }
                                    KeyCode::PageUp => {
                                        let page = 20usize;
                                        *selected = selected.saturating_sub(page);
                                    }
                                    KeyCode::Home => { *selected = 0; }
                                    KeyCode::End => { *selected = files.len().saturating_sub(1); }
                                    KeyCode::Char('s') => {
                                        if matches!(sort_by, SortField::Size) { *sort_desc = !*sort_desc; }
                                        else { *sort_by = SortField::Size; *sort_desc = true; }
                                        sort_files(files, sort_by, *sort_desc);
                                        *selected = 0;
                                    }
                                    KeyCode::Char('n') => {
                                        if matches!(sort_by, SortField::Name) { *sort_desc = !*sort_desc; }
                                        else { *sort_by = SortField::Name; *sort_desc = false; }
                                        sort_files(files, sort_by, *sort_desc);
                                        *selected = 0;
                                    }
                                    KeyCode::Char('t') => {
                                        if matches!(sort_by, SortField::Time) { *sort_desc = !*sort_desc; }
                                        else { *sort_by = SortField::Time; *sort_desc = true; }
                                        sort_files(files, sort_by, *sort_desc);
                                        *selected = 0;
                                    }
                                    KeyCode::Enter => {
                                        if detail.is_none() && !files.is_empty() {
                                            *detail = Some(*selected);
                                        }
                                    }
                                    KeyCode::Esc => {
                                        if detail.is_some() {
                                            *detail = None;
                                        } else {
                                            app.mode = Mode::Analysis { drive: *drive, cats: parent_cats.clone(), files: parent_files.clone(), dirs_scanned: parent_dirs_scanned.clone(), cancelled: parent_cancelled.clone(), scan_done: parent_scan_done.clone(), selected_category: *cat_index as usize };
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Mode::Detail { .. } => {
                                if let KeyCode::Esc = key.code {
                                    app.mode = Mode::Normal;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if last_refresh.elapsed() >= sleep_dur {
            app.refresh();
            last_refresh = Instant::now();
        }

        let _ = terminal.draw(|f| render(f, &app));
    }

    Ok(())
}

fn print_usage() {
    eprintln!("Usage: dfree [options]");
    eprintln!("Disk usage analyzer and volume manager.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -n <seconds>    Refresh interval (default: 3)");
    eprintln!("  --help, -h      Show this help");
}

pub fn uumain(mut raw_args: impl Iterator<Item = OsString>) -> i32 {
    install_ctrl_handler();

    let _prog = raw_args.next();

    let args: Vec<OsString> = raw_args.collect();
    let mut interval = 3u64;

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

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, interval);
    ratatui::restore();
    if let Err(e) = result {
        eprintln!("{}", e);
        1
    } else {
        0
    }
}

pub fn uu_app() -> Command {
    uumain([OsString::from("dfree"), OsString::from("--help")].into_iter());
    unreachable!()
}
