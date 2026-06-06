use std::ffi::OsString;
use std::io::{self, Read, Write};

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

type BOOL = i32;
type DWORD = u32;
type LSTATUS = i32;
type HKEY = usize;
type HANDLE = isize;
type LPCWSTR = *const u16;

const INVALID_HANDLE_VALUE: HANDLE = -1;
const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize;
const RRF_RT_REG_SZ: DWORD = 0x0000_0002;
const RRF_RT_REG_DWORD: DWORD = 0x0000_0010;
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

#[derive(Clone)]
#[allow(non_snake_case)]
#[repr(C)]
struct SYSTEM_LOGICAL_PROCESSOR_INFORMATION {
    processor_mask: usize,
    relationship: u32,
    _align: u32,
    data: [u8; 24],
}

const RELATION_PROCESSOR_CORE: u32 = 0;

const NO_ERROR: DWORD = 0;
const ERROR_BUFFER_OVERFLOW: DWORD = 111;
const AF_INET: i16 = 2;
const AF_INET6: i16 = 23;

#[allow(non_snake_case)]
#[repr(C)]
struct SOCKADDR_IN {
    sin_family: i16,
    sin_port: u16,
    sin_addr: [u8; 4],
    sin_zero: [u8; 8],
}

#[allow(non_snake_case)]
#[repr(C)]
struct SOCKADDR_IN6 {
    sin6_family: i16,
    sin6_port: u16,
    sin6_flowinfo: u32,
    sin6_addr: [u8; 16],
    sin6_scope_id: u32,
}

#[allow(non_snake_case)]
#[repr(C)]
struct SOCKET_ADDRESS {
    lp_sockaddr: *mut u8,
    i_sockaddr_length: i32,
}

#[allow(non_snake_case)]
#[repr(C)]
struct IP_ADAPTER_UNICAST_ADDRESS_LH {
    length: u32,
    flags: u32,
    next: *mut IP_ADAPTER_UNICAST_ADDRESS_LH,
    address: SOCKET_ADDRESS,
}

#[allow(non_snake_case)]
#[repr(C)]
struct IP_ADAPTER_ADDRESSES_LH {
    length: u32,
    if_index: u32,
    next: *mut IP_ADAPTER_ADDRESSES_LH,
    adapter_name: *mut u8,
    first_unicast: *mut IP_ADAPTER_UNICAST_ADDRESS_LH,
    first_anycast: *mut u8,
    first_multicast: *mut u8,
    first_dns: *mut u8,
    dns_suffix: *mut u16,
    description: *mut u16,
    friendly_name: *mut u16,
    physical_address: [u8; 8],
    physical_address_len: u32,
    _pad1: u32,
    flags: u32,
    mtu: u32,
    if_type: u32,
    oper_status: u32,
    ipv6_if_index: u32,
    zone_indices: [u32; 16],
    _pad2: u32,
}

#[allow(non_snake_case)]
#[repr(C)]
struct DISK_EXTENT {
    disk_number: DWORD,
    starting_offset: u64,
    extent_length: u64,
}

#[allow(non_snake_case)]
#[repr(C)]
struct VOLUME_DISK_EXTENTS {
    number_of_disk_extents: DWORD,
    extents: [DISK_EXTENT; 1],
}

#[allow(non_snake_case)]
#[repr(C)]
struct STORAGE_PROPERTY_QUERY {
    property_id: DWORD,
    query_type: DWORD,
    additional_parameters: [u8; 1],
}

#[allow(non_snake_case)]
#[repr(C)]
struct STORAGE_DEVICE_DESCRIPTOR {
    version: DWORD,
    size: DWORD,
    device_type: u8,
    device_type_modifier: u8,
    removable_media: u8,
    command_queueing: u8,
    vendor_id_offset: DWORD,
    product_id_offset: DWORD,
    product_revision_offset: DWORD,
    serial_number_offset: DWORD,
    bus_type: DWORD,
    raw_properties_length: DWORD,
    raw_device_properties: [u8; 1],
}

#[allow(non_snake_case)]
#[repr(C)]
struct STORAGE_SEEK_PENALTY_DESCRIPTOR {
    version: DWORD,
    size: DWORD,
    incurs_seek_penalty: u8,
}

const GENERIC_READ: DWORD = 0x8000_0000;
const FILE_SHARE_READ: DWORD = 1;
const FILE_SHARE_WRITE: DWORD = 2;
const OPEN_EXISTING: DWORD = 3;
const IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS: DWORD = 0x00560000;
const IOCTL_STORAGE_QUERY_PROPERTY: DWORD = 0x002D1400;
const STORAGE_DEVICE_PROPERTY: DWORD = 0;
const STORAGE_DEVICE_SEEK_PENALTY_PROPERTY: DWORD = 7;
const BUS_TYPE_NVME: DWORD = 0x11;

#[link(name = "ntdll")]
unsafe extern "system" {
    fn RtlGetVersion(lpVersionInformation: *mut RTL_OSVERSIONINFOW) -> LSTATUS;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetComputerNameExW(NameType: DWORD, lpBuffer: *mut u16, lpnSize: *mut DWORD) -> BOOL;
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
    fn GetSystemInfo(lpSystemInfo: *mut SYSTEM_INFO);
    fn GetLogicalProcessorInformation(
        Buffer: *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION,
        ReturnedLength: *mut DWORD,
    ) -> BOOL;
    fn CreateFileW(
        lpFileName: LPCWSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: *mut std::ffi::c_void,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    fn DeviceIoControl(
        hDevice: HANDLE,
        dwIoControlCode: DWORD,
        lpInBuffer: *const std::ffi::c_void,
        nInBufferSize: DWORD,
        lpOutBuffer: *mut std::ffi::c_void,
        nOutBufferSize: DWORD,
        lpBytesReturned: *mut DWORD,
        lpOverlapped: *mut std::ffi::c_void,
    ) -> BOOL;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
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

#[link(name = "iphlpapi")]
unsafe extern "system" {
    fn GetAdaptersAddresses(
        Family: u32,
        Flags: u32,
        Reserved: *mut u8,
        AdapterAddresses: *mut IP_ADAPTER_ADDRESSES_LH,
        SizePointer: *mut u32,
    ) -> u32;
}

#[link(name = "ws2_32")]
unsafe extern "system" {
    fn inet_ntop(Family: i32, pAddr: *const u8, pStringBuf: *mut u8, StringBufSize: u32) -> *mut u8;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetSystemFirmwareTable(
        FirmwareTableProviderSignature: DWORD,
        FirmwareTableID: DWORD,
        pFirmwareTableBuffer: *mut u8,
        BufferSize: DWORD,
    ) -> DWORD;
}

const SMBIOS_SIGNATURE: DWORD = 0x52534D42; // 'RSMB'

const ART: &[&str] = &[
    r"┌──────┐ ┌──────┐",
    r"│ ┌──┐ │ │ ┌──┐ │",
    r"│ │  │ │ │ │  │ │",
    r"│ └──┘ │ │ └──┘ │",
    r"└──────┘ └──────┘",
    r"┌──────┐ ┌──────┐",
    r"│ ┌──┐ │ │ ┌──┐ │",
    r"│ │  │ │ │ │  │ │",
    r"│ └──┘ │ │ └──┘ │",
    r"└──────┘ └──────┘",
];

fn display_banner() {
    for line in ART {
        println!("{line}");
    }
}

const ART_COLOR: Color = Color::Rgb(140, 140, 135);
const DISK: Color = Color::Rgb(156, 175, 136);

const LBL_OS:    Color = Color::Rgb(184, 176, 160);
const LBL_BIOS:  Color = Color::Rgb(160, 178, 190);
const LBL_OWNER: Color = Color::Rgb(168, 185, 158);
const LBL_PROD:  Color = Color::Rgb(185, 165, 165);
const LBL_HOST:  Color = Color::Rgb(175, 165, 185);
const LBL_CPU:   Color = Color::Rgb(190, 175, 155);
const LBL_MEM:   Color = Color::Rgb(155, 182, 178);
const LBL_SWAP:  Color = Color::Rgb(182, 178, 155);
const LBL_NIC:   Color = Color::Rgb(165, 172, 188);
const LBL_IP:    Color = Color::Rgb(188, 165, 175);

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

fn read_reg_dword(subkey: &str, value: &str) -> Option<u32> {
    let sk = wstr(subkey);
    let val = wstr(value);
    let mut data: u32 = 0;
    let mut size = 4u32;
    unsafe {
        let ret = RegGetValueW(
            HKEY_LOCAL_MACHINE,
            sk.as_ptr(),
            val.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            &mut data as *mut u32 as *mut u8,
            &mut size,
        );
        if ret == ERROR_SUCCESS { Some(data) } else { None }
    }
}

fn get_cpu_thread_count() -> u32 {
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

fn get_cpu_core_count() -> u32 {
    let mut len = 0u32;
    unsafe { GetLogicalProcessorInformation(std::ptr::null_mut(), &mut len); }
    if len == 0 { return 0; }
    let count = len as usize / size_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION>();
    let mut buffer: Vec<SYSTEM_LOGICAL_PROCESSOR_INFORMATION> = vec![
        SYSTEM_LOGICAL_PROCESSOR_INFORMATION {
            processor_mask: 0,
            relationship: 0,
            _align: 0,
            data: [0u8; 24],
        };
        count
    ];
    unsafe {
        GetLogicalProcessorInformation(buffer.as_mut_ptr(), &mut len);
    }
    buffer.iter().filter(|e| e.relationship == RELATION_PROCESSOR_CORE).count() as u32
}

fn get_cpu_freq_mhz() -> Option<u32> {
    read_reg_dword(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0", "~MHz")
}

fn get_system_hw() -> String {
    let mfr = read_reg_string(r"HARDWARE\DESCRIPTION\System\BIOS", "SystemManufacturer").unwrap_or_default();
    let model = read_reg_string(r"HARDWARE\DESCRIPTION\System\BIOS", "SystemProductName").unwrap_or_default();
    format!("{} {}", mfr.trim(), model.trim()).trim().to_string()
}

fn get_bios_version() -> String {
    let vendor = read_reg_string(r"HARDWARE\DESCRIPTION\System\BIOS", "BIOSVendor").unwrap_or_default().trim().to_string();
    let ver = read_reg_string(r"HARDWARE\DESCRIPTION\System\BIOS", "BIOSVersion").unwrap_or_default().trim().to_string();
    let date = read_reg_string(r"HARDWARE\DESCRIPTION\System\BIOS", "BIOSReleaseDate").unwrap_or_default().trim().to_string();
    if vendor.is_empty() && date.is_empty() {
        return ver;
    }
    let ver_part = [&vendor, &ver].iter()
        .filter(|s| !s.is_empty()).map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
    let date_part = date;
    if !date_part.is_empty() {
        format!("{ver_part}, {date_part}")
    } else {
        ver_part
    }
}

fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn timestamp_to_date(secs: u64) -> String {
    const SECS_PER_DAY: u64 = 86400;
    let days = secs / SECS_PER_DAY;
    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let diy = if is_leap_year(y) { 366 } else { 365 };
        if d < diy { break; }
        d -= diy;
        y += 1;
    }
    let month_days: &[i64] = &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 1i64;
    for &md in month_days {
        let dim = if m == 2 && is_leap_year(y) { 29 } else { md };
        if d < dim { break; }
        d -= dim;
        m += 1;
    }
    format!("{y:04}-{m:02}-{:02}", d + 1)
}

fn get_install_date() -> String {
    let ts = read_reg_dword(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", "InstallDate");
    ts.map(|s| timestamp_to_date(s as u64)).unwrap_or_default()
}

fn get_swap() -> (u64, u64) {
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
    (state.ullTotalPageFile, state.ullAvailPageFile)
}

fn get_product_id() -> String {
    read_reg_string(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", "ProductId").unwrap_or_default()
}

fn get_registered_owner() -> String {
    read_reg_string(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", "RegisteredOwner").unwrap_or_default()
}

fn get_memory_freq_mhz() -> Option<u32> {
    // First try registry
    if let Some(val) = read_reg_dword(r"HARDWARE\DESCRIPTION\System\BIOS", "SystemMemorySpeed") {
        return Some(val);
    }
    // Fallback: parse SMBIOS Type 17 (Memory Device) via GetSystemFirmwareTable
    unsafe {
        // Get required buffer size
        let size = GetSystemFirmwareTable(SMBIOS_SIGNATURE, 0, std::ptr::null_mut(), 0);
        if size == 0 { return None; }
        let mut buf = vec![0u8; size as usize];
        let written = GetSystemFirmwareTable(SMBIOS_SIGNATURE, 0, buf.as_mut_ptr(), size);
        if written == 0 || written < 32 { return None; }
        let buf = &buf[..written as usize];

        // Determine where the structure table starts
        let ep_len;
        if &buf[..4] == b"_SM_" {
            ep_len = buf[5] as usize; // entry point length for SMBIOS 2.x
        } else if &buf[..5] == b"_SM3_" {
            ep_len = buf[6] as usize; // entry point length for SMBIOS 3.x
        } else {
            return None;
        }
        if ep_len >= written as usize { return None; }

        let mut pos = ep_len;
        let mut max_speed = 0u32;
        while pos + 4 <= written as usize {
            let stype = buf[pos];
            let slen = buf[pos + 1] as usize;
            if slen < 2 { break; }
            if stype == 17 { // Memory Device
                // speed (MHz) at offset 0x15
                if slen > 0x15 + 1 {
                    let speed = u16::from_ne_bytes([buf[pos + 0x15], buf[pos + 0x16]]);
                    // configured_memory_clock_speed (MHz) at offset 0x20 (SMBIOS 2.7+)
                    if slen > 0x20 + 1 {
                        let cfg_speed = u16::from_ne_bytes([buf[pos + 0x20], buf[pos + 0x21]]);
                        if cfg_speed != 0 && (cfg_speed > speed || speed == 0) {
                            if (cfg_speed as u32) > max_speed { max_speed = cfg_speed as u32; }
                        } else if speed != 0 && (speed as u32) > max_speed {
                            max_speed = speed as u32;
                        }
                    } else if speed != 0 && (speed as u32) > max_speed {
                        max_speed = speed as u32;
                    }
                }
                // Skip strings after the formatted section
                pos += slen;
                while pos < written as usize && buf[pos] != 0 { pos += 1; }
                pos += 1; // skip null terminator
            } else if stype == 127 { // End-of-Table
                break;
            } else {
                // Skip to next structure
                pos += slen;
                while pos < written as usize && buf[pos] != 0 { pos += 1; }
                pos += 1; // skip null terminator
            }
        }
        if max_speed > 0 { Some(max_speed) } else { None }
    }
}

fn get_disk_type(letter: char) -> String {
    unsafe {
        let vol_path = wstr(&format!("\\\\.\\{}:", letter));
        let hvol = CreateFileW(
            vol_path.as_ptr(), GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(), OPEN_EXISTING, 0, 0,
        );
        if hvol == INVALID_HANDLE_VALUE { return String::new(); }

        let mut ext_buf = vec![0u8; 256];
        let mut ret = 0u32;
        let ok = DeviceIoControl(
            hvol, IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
            std::ptr::null::<std::ffi::c_void>(), 0,
            ext_buf.as_mut_ptr() as *mut std::ffi::c_void, ext_buf.len() as u32,
            &mut ret, std::ptr::null_mut(),
        );
        CloseHandle(hvol);
        if ok == 0 { return String::new(); }
        let ext = &mut *(ext_buf.as_mut_ptr() as *mut VOLUME_DISK_EXTENTS);
        if ext.number_of_disk_extents == 0 { return String::new(); }

        let phys_path = wstr(&format!("\\\\.\\PhysicalDrive{}", ext.extents[0].disk_number));
        let hphys = CreateFileW(
            phys_path.as_ptr(), 0, FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(), OPEN_EXISTING, 0, 0,
        );
        if hphys == INVALID_HANDLE_VALUE { return String::new(); }

        // Two-step query: first get descriptor size, then full descriptor
        let query = STORAGE_PROPERTY_QUERY { property_id: STORAGE_DEVICE_PROPERTY, query_type: 0, additional_parameters: [0] };
        let qsize = size_of::<STORAGE_PROPERTY_QUERY>() as u32;

        // Step 1: get size
        let mut header = [0u8; 8];
        let mut ret = 0u32;
        let ok1 = DeviceIoControl(
            hphys, IOCTL_STORAGE_QUERY_PROPERTY,
            &query as *const _ as *const std::ffi::c_void, qsize,
            header.as_mut_ptr() as *mut std::ffi::c_void, header.len() as u32,
            &mut ret, std::ptr::null_mut(),
        );
        if ok1 == 0 { CloseHandle(hphys); return String::new(); }
        let desc_size = u32::from_ne_bytes([header[4], header[5], header[6], header[7]]);

        // Step 2: allocate full buffer and query
        let mut buf = vec![0u8; desc_size as usize];
        let mut ret = 0u32;
        let ok2 = DeviceIoControl(
            hphys, IOCTL_STORAGE_QUERY_PROPERTY,
            &query as *const _ as *const std::ffi::c_void, qsize,
            buf.as_mut_ptr() as *mut std::ffi::c_void, buf.len() as u32,
            &mut ret, std::ptr::null_mut(),
        );

        let mut typ = String::new();
        if ok2 != 0 && buf.len() >= 32 {
            // bus_type is at offset 28 (DWORD)
            let bus_type = u32::from_ne_bytes([buf[28], buf[29], buf[30], buf[31]]);
            if bus_type == BUS_TYPE_NVME {
                typ = "SSD NVME".to_string();
            }
        }
        if typ.is_empty() {
            // Check seek penalty to distinguish HDD vs SSD
            let query2 = STORAGE_PROPERTY_QUERY {
                property_id: STORAGE_DEVICE_SEEK_PENALTY_PROPERTY,
                query_type: 0,
                additional_parameters: [0],
            };
            let mut seek = std::mem::zeroed::<STORAGE_SEEK_PENALTY_DESCRIPTOR>();
            let mut ret2 = 0u32;
            let ok2 = DeviceIoControl(
                hphys, IOCTL_STORAGE_QUERY_PROPERTY,
                &query2 as *const _ as *const std::ffi::c_void, qsize,
                &mut seek as *mut _ as *mut std::ffi::c_void, size_of::<STORAGE_SEEK_PENALTY_DESCRIPTOR>() as u32,
                &mut ret2, std::ptr::null_mut(),
            );
            if ok2 != 0 && seek.incurs_seek_penalty != 0 {
                typ = "HDD".to_string();
            } else {
                typ = "SSD".to_string();
            }
        }
        CloseHandle(hphys);
        typ
    }
}

fn mask_product_id(id: &str) -> String {
    let parts: Vec<&str> = id.split('-').collect();
    if parts.len() == 4 {
        format!("{}-*****-*****-{}", parts[0], parts[3])
    } else {
        id.to_string()
    }
}

fn mask_owner(owner: &str) -> String {
    if owner.is_empty() { return owner.to_string(); }
    if let Some(at) = owner.find('@') {
        let name = &owner[..at];
        let domain = &owner[at..];
        if name.len() > 4 {
            format!("{}*****{domain}", &name[..4])
        } else {
            format!("{name}*****{domain}")
        }
    } else if owner.len() > 4 {
        format!("{}*****", &owner[..4])
    } else {
        format!("{owner}*****")
    }
}

fn get_network_info() -> (String, String, String, String, bool) {
    let mut len = 0u32;
    let ret = unsafe {
        GetAdaptersAddresses(0, 0, std::ptr::null_mut(), std::ptr::null_mut(), &mut len)
    };
    if ret != ERROR_BUFFER_OVERFLOW || len < size_of::<IP_ADAPTER_ADDRESSES_LH>() as u32 {
        return (String::new(), String::new(), String::new(), String::new(), false);
    }
    let mut buf = vec![0u8; len as usize];
    let ret = unsafe {
        GetAdaptersAddresses(
            0, 0, std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH,
            &mut len,
        )
    };
    if ret != NO_ERROR { return (String::new(), String::new(), String::new(), String::new(), false); }

    let mut cur: *const IP_ADAPTER_ADDRESSES_LH = buf.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
    loop {
        let info = unsafe { &*cur };

        if !info.friendly_name.is_null() {
            let mut name_buf = [0u16; 256];
            let mut i = 0usize;
            unsafe {
                while i < 255 {
                    let c = *info.friendly_name.add(i);
                    name_buf[i] = c;
                    if c == 0 { break; }
                    i += 1;
                }
            }
            let name = String::from_utf16_lossy(&name_buf[..i]);
            if name.is_empty() { cur = info.next; if cur.is_null() { break; } continue; }

            let mut ipv4 = String::new();
            let mut ipv6 = String::new();

            let mut uc: *const IP_ADAPTER_UNICAST_ADDRESS_LH = info.first_unicast;
            while !uc.is_null() {
                let addr = unsafe { &*uc };
                let sa = addr.address.lp_sockaddr;
                if !sa.is_null() {
                    let family = unsafe { *(sa as *const i16) };
                    let mut ip_str = [0u8; 64];
                    let ptr = unsafe {
                        let data = match family {
                            AF_INET => {
                                let sin = &*(sa as *const SOCKADDR_IN);
                                &sin.sin_addr as *const u8
                            }
                            AF_INET6 => {
                                let sin6 = &*(sa as *const SOCKADDR_IN6);
                                &sin6.sin6_addr as *const u8
                            }
                            _ => { uc = addr.next; continue; }
                        };
                        inet_ntop(family as i32, data, ip_str.as_mut_ptr(), 64)
                    };
                    if !ptr.is_null() {
                        let end = ip_str.iter().position(|&c| c == 0).unwrap_or(64);
                        let s = String::from_utf8_lossy(&ip_str[..end]).to_string();
                        if !s.is_empty() && s != "0.0.0.0" && s != "::" && !s.starts_with("fe80") && !s.starts_with("127.") && !s.starts_with("169.254") {
                            match family {
                                AF_INET => ipv4 = s,
                                AF_INET6 => ipv6 = s,
                                _ => {}
                            }
                        }
                    }
                }
                uc = addr.next;
            }
            if !ipv4.is_empty() || !ipv6.is_empty() {
                // 读取描述
                let desc = if !info.description.is_null() {
                    let mut desc_buf = [0u16; 256];
                    let mut i = 0usize;
                    unsafe {
                        while i < 255 {
                            let c = *info.description.add(i);
                            desc_buf[i] = c;
                            if c == 0 { break; }
                            i += 1;
                        }
                    }
                    String::from_utf16_lossy(&desc_buf[..i])
                } else {
                    String::new()
                };

                // 通过 adapter_name (GUID) 查注册表 DHCP 状态
                let dhcp = if !info.adapter_name.is_null() {
                    let mut guid_buf = [0u8; 200];
                    let mut i = 0usize;
                    unsafe {
                        while i < 199 {
                            let c = *info.adapter_name.add(i);
                            guid_buf[i] = c;
                            if c == 0 { break; }
                            i += 1;
                        }
                    }
                    let name_ansi = String::from_utf8_lossy(&guid_buf[..i]).to_string();
                    if let (Some(start), Some(end)) = (name_ansi.find('{'), name_ansi.find('}')) {
                        let guid = &name_ansi[start..=end];
                        let key = format!(r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces\{guid}");
                        read_reg_dword(&key, "EnableDHCP").unwrap_or(0) == 1
                    } else {
                        false
                    }
                } else {
                    false
                };

                return (name, desc, ipv4, ipv6, dhcp);
            }
        }
        if info.next.is_null() { break; }
        cur = info.next;
    }
    (String::new(), String::new(), String::new(), String::new(), false)
}

fn get_os_info() -> (String, String, String) {
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

    // Windows 11 的注册表 ProductName 仍写 "Windows 10"，用版本号判断
    let product = if ver.dwBuildNumber >= 22000 {
        product.replace("Windows 10", "Windows 11")
    } else {
        product
    };

    let display = read_reg_string(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
        "DisplayVersion",
    ).unwrap_or_default();

    let ubr = read_reg_dword(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
        "UBR",
    ).unwrap_or(0);
    let full_build = format!("{}.{ubr}", ver.dwBuildNumber);

    (product, display, full_build)
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
    disk_type: String,
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
                let disk_type = get_disk_type(letter);
                disks.push(DiskInfo { letter, total, free, disk_type });
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

fn display_width(s: &str) -> usize {
    s.chars().count()
}

fn render(frame: &mut Frame, art_pad: &[String], info_lines: &[Line]) {
    let area = frame.area();

    let art_w = art_pad.iter().map(|s| display_width(s)).max().unwrap_or(0) as u16;
    let max_lines = art_pad.len().max(info_lines.len());

    let vert = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(max_lines as u16),
        Constraint::Length(1),
        Constraint::Fill(1),
    ]).split(area);

    let horiz = Layout::horizontal([
        Constraint::Length(2),
        Constraint::Length(art_w),
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(2),
    ]).split(vert[2]);

    let mut art_lines: Vec<Line> = Vec::with_capacity(max_lines);
    for i in 0..max_lines {
        art_lines.push(Line::from(
            Span::styled(
                art_pad.get(i).map_or("", |s| s.as_str()),
                Style::default().fg(ART_COLOR),
            ),
        ));
    }

    let mut info_full: Vec<Line> = Vec::with_capacity(max_lines);
    for i in 0..max_lines {
        info_full.push(info_lines.get(i).cloned().unwrap_or(Line::from("")));
    }

    let art_para = Paragraph::new(art_lines);
    let info_para = Paragraph::new(info_full);

    frame.render_widget(art_para, horiz[1]);
    frame.render_widget(info_para, horiz[3]);
}

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

    let user = get_username();
    let host = get_hostname();
    let (os_name, os_ver, os_build) = get_os_info();
    let (days, hours, mins) = get_uptime();
    let shell = get_shell();
    let cpu = get_cpu_name();
    let cores = get_cpu_core_count();
    let threads = get_cpu_thread_count();
    let freq_mhz = get_cpu_freq_mhz();
    let (total_phys, avail_phys) = get_memory();
    let used_phys = total_phys.saturating_sub(avail_phys);
    let hw = get_system_hw();
    let bios = get_bios_version();
    let installed = get_install_date();
    let (total_swap, avail_swap) = get_swap();
    let used_swap = total_swap.saturating_sub(avail_swap);
    let product_id = get_product_id();
    let owner = get_registered_owner();
    let (nic_name, nic_desc, nic_ipv4, nic_ipv6, nic_dhcp) = get_network_info();
    let disks = get_disks();

    let mut info_lines: Vec<Line> = Vec::new();

    // label width = 21 (longest is "OS · Version · Kernel")
    let pad = |l: &str, v: &str, c: Color| -> Line {
        Line::from(vec![
            Span::styled(format!("{:<21}", l), Style::default().fg(c)),
            Span::raw("  "),
            Span::styled(v.to_string(), Style::default().fg(c)),
        ])
    };

    info_lines.push(pad("OS · Version · Kernel", &format!("{} · {} · {}", os_name, os_ver, os_build), LBL_OS));
    info_lines.push(pad("BIOS · Uptime", &format!("{} · {}d {}h {}m", bios, days, hours, mins), LBL_BIOS));
    info_lines.push(pad("Owner · HW",    &format!("{} · {}", mask_owner(&owner), hw), LBL_OWNER));
    info_lines.push(pad("Product · Installed",
        &format!("{} · {}", mask_product_id(&product_id), installed), LBL_PROD));
    info_lines.push(pad("host · Shell",  &format!("{}@{} · {}", user, host, shell), LBL_HOST));

    let cpu_extra = {
        let freq = freq_mhz.map(|f| format!(" @ {} MHz", f)).unwrap_or_default();
        if cores > 0 && threads > 0 {
            format!(" ({cores}C·{threads}T{freq})")
        } else if threads > 0 {
            format!(" ({threads}T{freq})")
        } else {
            freq
        }
    };
    info_lines.push(pad("CPU", &format!("{cpu}{cpu_extra}"), LBL_CPU));
    let mem_freq = get_memory_freq_mhz();
    let mem_val = if let Some(f) = mem_freq {
        format!("{} / {} · {} MHz", format_size(used_phys), format_size(total_phys), f)
    } else {
        format!("{} / {}", format_size(used_phys), format_size(total_phys))
    };
    info_lines.push(pad("Memory", &mem_val, LBL_MEM));

    if total_swap > 0 {
        info_lines.push(pad("Swap", &format!("{} · {}", format_size(used_swap), format_size(total_swap)), LBL_SWAP));
    }

    if !nic_name.is_empty() {
        let nic_line = if !nic_desc.is_empty() && nic_dhcp {
            format!("{} · {} · DHCP", nic_name, nic_desc)
        } else if !nic_desc.is_empty() {
            format!("{} · {}", nic_name, nic_desc)
        } else if nic_dhcp {
            format!("{} · DHCP", nic_name)
        } else {
            nic_name.clone()
        };
        info_lines.push(pad("NIC", &nic_line, LBL_NIC));
        let mut first_ip = true;
        for (tag, addr) in [("ipv4", &nic_ipv4[..]), ("ipv6", &nic_ipv6[..])] {
            if !addr.is_empty() {
                let lbl = if first_ip { "IP" } else { "" };
                info_lines.push(pad(lbl, &format!("[{tag}] {addr}"), LBL_IP));
                first_ip = false;
            }
        }
    }

    if !disks.is_empty() {
        for (i, d) in disks.iter().enumerate() {
            let used = d.total.saturating_sub(d.free);
            let pct = if d.total > 0 { used as f64 / d.total as f64 * 100.0 } else { 0.0 };
            let lbl = if i == 0 { "Disks".to_string() } else { String::new() };
            let dt = if d.disk_type.is_empty() { String::new() } else { format!(" · {}", d.disk_type) };
            info_lines.push(Line::from(vec![
                Span::styled(format!("{:<21}", lbl), Style::default().fg(DISK).bold()),
                Span::raw("  "),
                Span::styled(format!("[{}]{}", d.letter, dt), Style::default().fg(DISK)),
                Span::styled(" · ", Style::default().fg(DISK)),
                Span::styled(
                    format!("{} / {}  ({:.0}%)", format_size(used), format_size(d.total), pct),
                    Style::default().fg(DISK),
                ),
            ]));
        }
    }

    let art_w = ART.iter().map(|s| display_width(s)).max().unwrap_or(0);
    let mut art_pad: Vec<String> = ART.iter().map(|&s| {
        let mut line = s.to_string();
        let w = display_width(&line);
        for _ in w..art_w {
            line.push(' ');
        }
        line
    }).collect();
    while art_pad.len() < info_lines.len() {
        art_pad.push(" ".repeat(art_w));
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut terminal = ratatui::init();
        let _ = terminal.draw(|f| {
            render(f, &art_pad, &info_lines);
        });
        let _ = io::stdin().read(&mut [0u8; 64]);
        ratatui::restore();
    }));

    match result {
        Ok(()) => 0,
        Err(_) => {
            // try to restore terminal even if panic occurred mid-way
            let _ = ratatui::try_restore();
            // write a visible message so the user knows something went wrong
            let _ = writeln!(io::stderr(), "winfo: terminal error — please try again in a normal terminal window");
            1
        }
    }
}
