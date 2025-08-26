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
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use std::os::windows::process::CommandExt;

/*========================= THREADING ============================*/

const TTL_SLOW: Duration = Duration::from_secs(15);
const TTL_FAST: Duration = Duration::from_secs(3);

static HOSTNAME: OnceLock<String> = OnceLock::new();
static CURRENT_USER: OnceLock<String> = OnceLock::new();
static PROXY_CACHE: OnceLock<Mutex<(Instant, bool)>> = OnceLock::new();
static FIREWALL_CACHE: OnceLock<Mutex<(Instant, bool)>> = OnceLock::new();

static CPU_USAGE_CACHE: OnceLock<Mutex<(Instant, String)>> = OnceLock::new();
static MEM_CACHE: OnceLock<Mutex<(Instant, String, String)>> = OnceLock::new();
static CPU_TIMES_LAST: OnceLock<Mutex<(u64, u64, u64)>> = OnceLock::new();
static OS_CACHE: OnceLock<Mutex<(Instant, Option<(String, String)>)>> = OnceLock::new();
static CPU_INFO_CACHE: OnceLock<Mutex<(Instant, String)>> = OnceLock::new();
static MANUF_CACHE: OnceLock<Mutex<(Instant, String)>> = OnceLock::new();
static JOIN_CACHE: OnceLock<Mutex<(Instant, String)>> = OnceLock::new();
static INTRANET_CACHE: OnceLock<Mutex<(Instant, bool)>> = OnceLock::new();
static INSTALLED_CACHE: OnceLock<Mutex<std::collections::HashMap<String, (Instant, (bool, String))>>> = OnceLock::new();

#[link(name = "Kernel32")]
extern "system" {
    fn GetLocalTime(lpSystemTime: *mut SYSTEMTIME);
}

/*=============================== QUERYS =================================*/
/*============================= REG QUERYS ===============================*/

pub fn is_installed(prog: &str) -> (bool, String) {
    let needle = prog.to_ascii_lowercase();
    let now = Instant::now();
    let map = INSTALLED_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    if let Ok(mut m) = map.lock() {
        if let Some((ts, v)) = m.get(&needle).cloned() {
            if now.duration_since(ts) < TTL_SLOW { return v; }
        }
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let mut out = (false, String::from("Não Instalado"));
        if let Ok(uninstall) = hklm.open_subkey_with_flags(r"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall", KEY_READ) {
            for sub in uninstall.enum_keys().flatten() {
                if let Ok(appkey) = uninstall.open_subkey_with_flags(&sub, KEY_READ) {
                    let name: Option<String> = appkey.get_value("DisplayName").ok()
                        .or_else(|| appkey.get_value("QuietDisplayName").ok());
                    if let Some(name) = name {
                        if name.to_ascii_lowercase().contains(&needle) {
                            out = (true, appkey.get_value("DisplayVersion").unwrap_or_else(|_| "Não Instalado".into()));
                            break;
                        }
                    }
                }
            }
        }
        m.insert(needle, (now, out.clone()));
        return out;
    }
    (false, "Não Instalado".into())
}

pub fn get_os() -> Option<(String, String)> {
    let now = Instant::now();
    let cache = OS_CACHE.get_or_init(|| Mutex::new((Instant::now() - TTL_SLOW, None)));
    let mut c = cache.lock().unwrap();
    if now.duration_since(c.0) < TTL_SLOW { return c.1.clone(); }
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let v = if let Ok(key) = hklm.open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", KEY_READ) {
        let name: String = key.get_value("ProductName").unwrap_or_default();
        let version: String = key.get_value("DisplayVersion").or_else(|_| key.get_value("ReleaseId")).unwrap_or_default();
        Some((format!("{} {}", name, version).trim().to_string(), std::env::consts::ARCH.to_string()))
    } else { None };
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

pub fn get_memory_info() -> (String, String) {
    let now = Instant::now();
    let mut cache = MEM_CACHE
        .get_or_init(|| Mutex::new((Instant::now() - TTL_FAST, String::new(), String::new())))
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
                *cache = (now, total_mb, avail_mb);
            }
        }
    }
    (cache.1.clone(), cache.2.clone())
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
    if now.duration_since(c.0) < TTL_SLOW { return c.1.clone(); }
    let v: String = unsafe {
        #[repr(C)]
        struct DSREG_JOIN_INFO {
            join_type: u32,
        }

        if let Ok(h) = LoadLibraryW(windows::core::w!("dsreg.dll")) {
            if let (Some(g), Some(f)) = (
                GetProcAddress(h, windows::core::s!("DsRegGetJoinInfo")),
                GetProcAddress(h, windows::core::s!("DsRegFreeJoinInfo")),
            ) {
                let get: extern "system" fn(PCWSTR, *mut *mut DSREG_JOIN_INFO) -> i32 =
                    std::mem::transmute(g);
                let free: extern "system" fn(*mut DSREG_JOIN_INFO) =
                    std::mem::transmute(f);

                let mut info: *mut DSREG_JOIN_INFO = std::ptr::null_mut();
                if windows::core::HRESULT(get(PCWSTR::null(), &mut info)).is_ok() && !info.is_null() {
                    let joined = (*info).join_type == 1 || (*info).join_type == 2;
                    free(info);
                    if joined {
                        return "INTUNE".into();
                    }
                }
            }
        }

        use windows::Win32::NetworkManagement::NetManagement::*;
        let mut status = NETSETUP_JOIN_STATUS(0);
        // Pass a valid LPWSTR* to receive the name and free it afterwards
        let mut name: windows::core::PWSTR = windows::core::PWSTR::null();
        if NetGetJoinInformation(PCWSTR::null(), &mut name, &mut status) == 0 {
            if !name.is_null() {
                let _ = NetApiBufferFree(Some(name.0 as _));
            }
            if status == NetSetupDomainName {
                return "ACTIVE DIRECTORY".into();
            }
        }
        "NÃO GERENCIADA".into()
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
            .args(&["-n", "1", "intranet.com.br"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        guard.0 = now;
        guard.1 = val;
    }

    guard.1
}

