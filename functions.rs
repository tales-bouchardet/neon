
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows::Win32::NetworkManagement::WindowsFirewall::{INetFwPolicy2, NetFwPolicy2, NET_FW_PROFILE2_ALL};
use windows::Win32::System::SystemInformation::{GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::Threading::GetSystemTimes;
use windows::Win32::Networking::WinHttp::{WinHttpGetIEProxyConfigForCurrentUser, WINHTTP_CURRENT_USER_IE_PROXY_CONFIG};
use windows::Win32::System::Com::{CoInitializeEx, CoCreateInstance, COINIT_APARTMENTTHREADED, CLSCTX_ALL};
use windows::core::PCWSTR;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;
use std::os::windows::process::CommandExt;
use win_screenshot::prelude::*;
use image::{RgbaImage};
use regex::Regex;
use std::error::Error;
use arboard::{Clipboard, ImageData};

/*========================= THREADING ============================*/

const TTL_SLOW: Duration = Duration::from_secs(15);
const TTL_FAST: Duration = Duration::from_secs(3);

static HOSTNAME: OnceLock<String> = OnceLock::new();
static CURRENT_USER: OnceLock<String> = OnceLock::new();
static PROXY_CACHE: OnceLock<Mutex<(Instant, bool)>> = OnceLock::new();
static FIREWALL_CACHE: OnceLock<Mutex<(Instant, bool)>> = OnceLock::new();

static CPU_USAGE_CACHE: OnceLock<Mutex<(Instant, String)>> = OnceLock::new();
static MEM_CACHE: OnceLock<Mutex<(Instant, String)>> = OnceLock::new();
static CPU_TIMES_LAST: OnceLock<Mutex<(u64, u64, u64)>> = OnceLock::new();
static OS_CACHE: OnceLock<Mutex<(Instant, String)>> = OnceLock::new();
static CPU_INFO_CACHE: OnceLock<Mutex<(Instant, String)>> = OnceLock::new();
static MANUF_CACHE: OnceLock<Mutex<(Instant, String)>> = OnceLock::new();
static JOIN_CACHE: OnceLock<Mutex<(Instant, String)>> = OnceLock::new();
static INTRANET_CACHE: OnceLock<Mutex<(Instant, bool)>> = OnceLock::new();
static FILE_VERSION_CACHE: OnceLock<Mutex<std::collections::HashMap<String, (Instant, windows::core::Result<String>)>>> = OnceLock::new();

#[link(name = "Kernel32")]
extern "system" {
    fn GetLocalTime(lpSystemTime: *mut SYSTEMTIME);
}

/*=============================== QUERYS =================================*/
/*============================= REG QUERYS ===============================*/

pub fn get_file_version(path: &str) -> windows::core::Result<String> {
    let now = Instant::now();
    let cache = FILE_VERSION_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    let mut map = cache.lock().unwrap();

    const TTL: Duration = TTL_SLOW;

    if let Some((ts, res)) = map.get(path) {
        if now.duration_since(*ts) < TTL {
            return res.clone();
        }
    }

    use windows::Win32::Storage::FileSystem::*;
    use windows::core::PCWSTR;

    let result = unsafe {
        let filename: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let mut handle = 0u32;
        let size = GetFileVersionInfoSizeW(PCWSTR(filename.as_ptr()), Some(&mut handle));
        if size == 0 { Err(windows::core::Error::from_win32()) }
        else {
            let mut buffer = vec![0u8; size as usize];
            if GetFileVersionInfoW(PCWSTR(filename.as_ptr()), Some(handle), size, buffer.as_mut_ptr() as *mut _).is_err() {
                Err(windows::core::Error::from_win32())
            } else {
                let mut lp_buffer: *mut std::ffi::c_void = std::ptr::null_mut();
                let mut len = 0u32;
                let sub_block: Vec<u16> = "\\\0".encode_utf16().collect();
                if !VerQueryValueW(buffer.as_ptr() as *const _, PCWSTR(sub_block.as_ptr()), &mut lp_buffer, &mut len).as_bool() || lp_buffer.is_null() {
                    Err(windows::core::Error::from_win32())
                } else {
                    let info = &*(lp_buffer as *const VS_FIXEDFILEINFO);
                    Ok(format!("{}.{}.{}.{}", info.dwFileVersionMS >> 16, info.dwFileVersionMS & 0xffff, info.dwFileVersionLS >> 16, info.dwFileVersionLS & 0xffff))
                }
            }
        }
    };

    map.insert(path.to_string(), (now, result.clone()));
    result
}

pub fn get_os() -> String {
    let now = Instant::now();
    let cache = OS_CACHE.get_or_init(|| Mutex::new((Instant::now() - TTL_SLOW, String::new())));
    let mut c = cache.lock().unwrap();
    if now.duration_since(c.0) < TTL_SLOW { return c.1.clone(); }
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let v = if let Ok(key) = hklm.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion", KEY_READ) {
        let name: String = key.get_value("ProductName").unwrap_or_default();
        let version: String = key.get_value("DisplayVersion").or_else(|_| key.get_value("ReleaseId")).unwrap_or_default();
        format!("{} ({})", name.replace("Single Language", "").replace("Multi Language", ""), version).trim().to_string()
    } else { String::new() };
    *c = (now, v.clone());
    v
}

pub fn get_cpu_info() -> String {
    let now = Instant::now();
    let cache = CPU_INFO_CACHE.get_or_init(|| Mutex::new((Instant::now() - TTL_SLOW, String::new())));
    let mut c = cache.lock().unwrap();
    if now.duration_since(c.0) < TTL_SLOW { return c.1.clone(); }
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let v = if let Ok(key) = hklm.open_subkey_with_flags(r"HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0", KEY_READ) {
        key.get_value("ProcessorNameString").unwrap_or_default()
    } else { String::new() };
    *c = (now, v.clone());
    v
}

pub fn get_manufacturer() -> String {
    let now = Instant::now();
    let cache = MANUF_CACHE.get_or_init(|| Mutex::new((Instant::now() - TTL_SLOW, String::new())));
    let mut c = cache.lock().unwrap();
    if now.duration_since(c.0) < TTL_SLOW { return c.1.clone(); }
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let v = if let Ok(key) = hklm.open_subkey_with_flags(r"HARDWARE\\DESCRIPTION\\System\\BIOS", KEY_READ) {
        let m: String = key.get_value("SystemManufacturer").unwrap_or_default();
        let p: String = key.get_value("SystemProductName").unwrap_or_default();
        format!("{} {}", m, p).trim().to_string()
    } else { String::new() };
    *c = (now, v.clone());
    v
}

/*======================== ENVIROMENT VARIABLES =========================*/

pub fn get_hostname() -> String {
    HOSTNAME
        .get_or_init(|| std::env::var("COMPUTERNAME").unwrap_or_default())
        .clone()
}

pub fn get_current_user() -> String {
    CURRENT_USER
        .get_or_init(|| std::env::var("USERNAME").unwrap_or_default())
        .clone()
}

/*===================== WIN32 READ SYSTEM BUFFERS ======================*/

pub fn get_cpu_usage() -> String {
    let now = Instant::now();
    let mut cache = CPU_USAGE_CACHE
        .get_or_init(|| Mutex::new((Instant::now() - TTL_FAST, String::new())))
        .lock()
        .unwrap();
    if now.duration_since(cache.0) < TTL_FAST {
        return cache.1.clone();
    }
    let percent = unsafe {
        let mut idle = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        if GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)).is_err() {
            0.0
        } else {
            let i = ((idle.dwHighDateTime as u64) << 32) | (idle.dwLowDateTime as u64);
            let k = ((kernel.dwHighDateTime as u64) << 32) | (kernel.dwLowDateTime as u64);
            let u = ((user.dwHighDateTime as u64) << 32) | (user.dwLowDateTime as u64);
            let mut last = CPU_TIMES_LAST.get_or_init(|| Mutex::new((i, k, u))).lock().unwrap();
            let (pi, pk, pu) = *last;
            let di = i.saturating_sub(pi);
            let dk = k.saturating_sub(pk);
            let du = u.saturating_sub(pu);
            *last = (i, k, u);
            let total = (dk + du) as f64;
            if total <= 0.0 { 0.0 } else { ((total - di as f64) / total) * 100.0 }
        }
    };
    let val = format!("{:.2}%", percent);
    *cache = (now, val.clone());
    val
}

pub fn get_uptime() -> String {
    let uptime = unsafe { GetTickCount64() / 1000 };
    let d = uptime / 86_400;
    let h = (uptime % 86_400) / 3_600;
    let m = (uptime % 3_600) / 60;
    let s = uptime % 60;
    format!("{:02}d {:02}h {:02}m {:02}s", d, h, m, s)
}

pub fn get_memory_info() -> String {
    let now = Instant::now();
    let mut cache = MEM_CACHE
        .get_or_init(|| Mutex::new((Instant::now() - TTL_FAST, String::new())))
        .lock()
        .unwrap();
    if now.duration_since(cache.0) >= TTL_FAST {
        unsafe {
            let mut msx = MEMORYSTATUSEX {
                dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                ..Default::default()
            };
            if GlobalMemoryStatusEx(&mut msx).is_ok() {
                let total_mb = format!("{:.2} MB", msx.ullTotalPhys as f64 / 1024.0 / 1024.0);
                let avail_mb = format!("{:.2} MB", msx.ullAvailPhys as f64 / 1024.0 / 1024.0);
                *cache = (now, format!("[\"{}\", \"{}\"]", total_mb, avail_mb));
            }
        }
    }
    cache.1.clone()
}

pub fn is_firewall_enabled() -> bool {
    let now = Instant::now();
    let cache = FIREWALL_CACHE
        .get_or_init(|| Mutex::new((Instant::now() - TTL_SLOW, false)));
    let mut guard = cache.lock().unwrap();

    if now.duration_since(guard.0) >= TTL_SLOW {
        let val = unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok()
                && CoCreateInstance::<Option<&windows::core::IUnknown>, INetFwPolicy2>(
                    &NetFwPolicy2,
                    None,
                    CLSCTX_ALL,
                )
                .and_then(|p| p.get_FirewallEnabled(NET_FW_PROFILE2_ALL))
                .map(|v| v.as_bool())
                .unwrap_or(false)
        };
        guard.0 = now;
        guard.1 = val;
    }

    guard.1
}

pub fn is_proxy_enabled() -> bool {
    let now = Instant::now();
    let mut guard = PROXY_CACHE
        .get_or_init(|| Mutex::new((Instant::now() - TTL_SLOW, false)))
        .lock()
        .unwrap();

    if now.duration_since(guard.0) >= TTL_SLOW {
        let val = {
            let mut c = WINHTTP_CURRENT_USER_IE_PROXY_CONFIG::default();
            unsafe { WinHttpGetIEProxyConfigForCurrentUser(&mut c) }.is_ok() && !c.lpszProxy.is_null()
        };

        guard.0 = now;
        guard.1 = val;
    }

    guard.1
}

pub fn get_now() -> String {
    unsafe {
        let mut st = SYSTEMTIME::default();
        GetLocalTime(&mut st);
        format!("{:02}:{:02}:{:02}", st.wHour, st.wMinute, st.wSecond)
    }
}

pub fn get_date() -> String {
    unsafe {
        let mut st = SYSTEMTIME::default();
        GetLocalTime(&mut st);
        format!("{:02}/{:02}/{:04}", st.wDay, st.wMonth, st.wYear)
    }
}

/*========================== WIN32 DLL LOAD =========================*/

pub fn get_join_info() -> String {
    let now = Instant::now();
    let cache = JOIN_CACHE.get_or_init(|| Mutex::new((Instant::now() - TTL_SLOW, String::new())));
    let mut c = cache.lock().unwrap();
    if now.duration_since(c.0) < TTL_SLOW {
        return c.1.clone();
    }
    let v: String = unsafe {
        use windows::Win32::NetworkManagement::NetManagement::*;
        let mut status = NETSETUP_JOIN_STATUS(0);
        let mut name: windows::core::PWSTR = windows::core::PWSTR::null();
        let result = NetGetJoinInformation(PCWSTR::null(), &mut name, &mut status);

        #[allow(non_upper_case_globals)]
        const NetSetupAzureADJoined: NETSETUP_JOIN_STATUS = NETSETUP_JOIN_STATUS(3);

        if result == 0 {
            if !name.is_null() {
                let _ = NetApiBufferFree(Some(name.0 as _));
            }
            if status == NetSetupDomainName {
                return "<green>ACTIVE DIRECTORY</green>".into();
            } else if status == NetSetupAzureADJoined {
                return "<green>AZURE AD</green>".into();
            } else {
                use windows::Win32::System::Services::{
                    CloseServiceHandle, OpenSCManagerW, OpenServiceW, SC_MANAGER_CONNECT, SERVICE_QUERY_STATUS,
                };

                let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT);
                if let Ok(scm_handle) = scm {
                    if let Ok(svc_handle) = OpenServiceW(scm_handle, windows::core::w!("IntuneManagementExtension"), SERVICE_QUERY_STATUS) {
                        let _ = CloseServiceHandle(svc_handle);
                        let _ = CloseServiceHandle(scm_handle);
                        return "<green>INTUNE</green>".into();
                    }
                    let _ = CloseServiceHandle(scm_handle);
                }
            }
        }
        "<red>NÃO GERENCIADA</red>".into()
    };
    *c = (now, v.clone());
    v
}

pub fn have_intranet_access() -> bool {
    let now = Instant::now();
    let mut guard = INTRANET_CACHE
        .get_or_init(|| Mutex::new((Instant::now() - TTL_SLOW, false)))
        .lock()
        .unwrap();

    if now.duration_since(guard.0) >= TTL_SLOW {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let val = std::process::Command::new("ping.exe")
            .args(&["-n", "1", "grupoaec.com.br"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        guard.0 = now;
        guard.1 = val;
    }

    guard.1
}

pub fn screenshot() -> Result<(), Box<dyn Error>> {
    let re = Regex::new(r"N.E.O.N")?; 

    let hwnd = window_list() 
    .map_err(|e| Box::<dyn Error>::from(format!("{:?}", e)))?
    .into_iter() .find(|i| re.is_match(&i.window_name))
    .ok_or("No matching window found")? 
    .hwnd; 

    let area = Area::ClientOnly;
    let using = Using::PrintWindow;

    let crop_xy = None; 
    let crop_wh = None;

    let buf = capture_window_ex(hwnd, using, area, crop_xy, crop_wh)
    .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    let rgba = RgbaImage::from_raw(buf.width, buf.height, buf.pixels)
    .ok_or("Failed to create image from buffer")?; 

    let (width, height) = rgba.dimensions();

    let bytes = rgba.into_raw();

    let mut clipboard = Clipboard::new()?;
    let image = ImageData {
        width: width as usize,
        height: height as usize,
        bytes: std::borrow::Cow::Owned(bytes),
    };
    clipboard.set_image(image)?;

    Ok(())
}

