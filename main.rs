#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod functions;
use crate::functions::*;
use web_view::*;
use serde_json::json;

static HTML: &str = include_str!("assets/main.html");
static CSS: &str = include_str!("assets/main.css");
static JS: &str = include_str!("assets/main.js");

fn main() {
    let mut webview = web_view::builder()
        .title("N.E.O.N.")
        .content(Content::Html(HTML))
        .size(420, 580)
        .resizable(false)
        .debug(false)
        .user_data(())
        .invoke_handler(|webview, arg| {
            match arg {
                "refresh" => {
                    let data = json!({
                        "now": get_now(),
                        "date": get_date(),
                        "manufacturer": get_manufacturer(),
                        "os": format!("{} [{}]", get_os(), std::env::consts::ARCH),
                        "user": get_current_user(),
                        "hostname": get_hostname(),
                        "cpu": get_cpu_info(),
                        "cpu_usage": get_cpu_usage(),
                        "memory": get_memory_info(),
                        "firewall": if is_firewall_enabled() { "Ativado" } else { "Desativado" },
                        "proxy": if is_proxy_enabled() { "<green>PROXY ATIVO</green>" } else { "<red>PROXY INATIVO</red>" },
                        "join": get_join_info(),
                        "intranet": if have_intranet_access() { "<green>ACESSO À REDE INTERNA</green>" } else { "<red>SEM ACESSO À REDE INTERNA</red>" },
                        "uptime": get_uptime(),
                        "forticlient": get_file_version("C:\\Program Files\\Fortinet\\FortiClient\\FortiClient.exe").unwrap_or_else(|_| "Não Instalado".to_string()),
                        "crowd": get_file_version("C:\\Program Files\\CrowdStrike\\CSFalconService.exe").unwrap_or_else(|_| "Não Instalado".to_string()),
                        "kace": get_file_version("C:\\Program Files (x86)\\Quest\\KACE\\AMPTools.exe").unwrap_or_else(|_| "Não Instalado".to_string()),
                        "netskope": get_file_version("C:\\Program Files\\Netskope\\EPDLP\\EPDLP.exe").unwrap_or_else(|_| "Não Instalado".to_string())
                    });
                    let js_code = format!("data = {};", data.to_string());
                    webview.eval(&js_code).unwrap();
                    Ok(())
                }
                "screenshot" => {
                    if let Err(e) = screenshot() {
                        eprintln!("Erro ao capturar a tela: {}", e);
                    }
                    Ok(())
                }
                 _ => Ok(()),
            }
        })
        .build()
        .unwrap();

    webview.inject_css(CSS).unwrap();
    webview.eval(JS).unwrap();

    webview.run().unwrap();
}
