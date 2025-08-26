#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod functions;
use eframe::egui::{self, CentralPanel, Color32, RichText};
use crate::functions::*;

#[derive(Default)]
    struct Neon;

impl eframe::App for Neon {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {     
        ctx.set_visuals(egui::Visuals::dark());
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        let text_color = Color32::from_rgb(200, 200, 200);
        let label_color = Color32::from_rgb(30, 96, 228);
        let found_color = Color32::from_rgb(30, 228, 82);
        let not_found_color = Color32::from_rgb(228, 30, 33);

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.set_width(450.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(format!("{}", get_now())).color(text_color).size(20.0));
                    });
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(format!("{}", get_date())).color(text_color).size(20.0));
                    });
                });
                ui.group(|ui| {
                    ui.set_width(450.0);
                    ui.horizontal(|ui| { 
                        ui.label(RichText::new("FABRICANTE:").color(label_color).size(15.0));
                        ui.label(RichText::new(format!("{}", get_manufacturer())).color(text_color).size(15.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("SO:").color(label_color).size(15.0));
                        if let Some((os, arch)) = get_os() {
                            ui.label(RichText::new(format!("{} ({})", os, arch)).color(text_color).size(15.0));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("USUÁRIO:").color(label_color).size(15.0));
                        ui.label(RichText::new(get_current_user()).color(text_color).size(15.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("HOSTNAME:").color(label_color).size(15.0));
                        ui.label(RichText::new(format!("{}", get_hostname())).color(text_color).size(15.0));
                    });
                });

                ui.group(|ui| {
                    ui.set_width(450.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("CPU:").color(label_color).size(15.0));
                        ui.label(RichText::new(format!("{}", get_cpu_info())).color(text_color).size(15.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("USO DE CPU:").color(label_color).size(15.0));
                        ui.label(RichText::new(format!("{}", get_cpu_usage())).color(text_color).size(15.0));
                    });
                });

                ui.group(|ui| {
                    ui.set_width(450.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("RAM:").color(label_color).size(15.0));
                            let (total, _avail) = get_memory_info();
                            ui.label(RichText::new(total).color(text_color).size(15.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("RAM DISP.:").color(label_color).size(15.0));
                            let (_, avail) = get_memory_info();
                            ui.label(RichText::new(avail).color(text_color).size(15.0));
                    });
                });

                ui.group(|ui| {
                    ui.set_width(450.0);
                    ui.horizontal(|ui| {
                        if is_firewall_enabled() {
                            ui.label(RichText::new("Firewall:").color(not_found_color).size(15.0));
                            ui.label(RichText::new("Ativado").color(text_color).size(15.0));
                        } else {
                            ui.label(RichText::new("Firewall:").color(found_color).size(15.0));
                            ui.label(RichText::new("Desativado").color(text_color).size(15.0));
                        }
                    });

                    let (forti_installed, _forti_info) = is_installed("Forti");
                    ui.horizontal(|ui| {
                        let label_col = if forti_installed { found_color } else { not_found_color };
                        ui.label(RichText::new("FortiClient:").color(label_col).size(15.0));
                        ui.label(RichText::new(format!("{}", _forti_info)).color(text_color).size(15.0));
                    });

                    let (cs_installed, _cs_info) = is_installed("CrowdStrike");
                    ui.horizontal(|ui| {
                        let label_col = if cs_installed { found_color } else { not_found_color };
                        ui.label(RichText::new("CrowdStrike:").color(label_col).size(15.0));
                        ui.label(RichText::new(format!("{}", _cs_info)).color(text_color).size(15.0));
                    });

                    let (kace_installed, _kace_info) = is_installed("KACE");
                    ui.horizontal(|ui| {
                        let label_col = if kace_installed { found_color } else { not_found_color };
                        ui.label(RichText::new("KACE:").color(label_col).size(15.0));
                        ui.label(RichText::new(format!("{}", _kace_info)).color(text_color).size(15.0));
                    });
                });

                ui.group(|ui| {
                    ui.set_width(450.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("EM ATIVIDADE:").color(label_color).size(15.0));
                        ui.label(RichText::new(format!("{}", get_uptime())).color(text_color).size(15.0));
                    });
                });

                // Centralizado apenas neste grupo
                ui.group(|ui| {
                    ui.set_width(450.0);
                    let join_info = get_join_info();
                    if join_info == "NÃO GERENCIADA" {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(format!("{}", join_info)).color(not_found_color).size(15.0));
                        });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(format!("{}", join_info)).color(found_color).size(15.0));
                        });
                    }
                    if is_proxy_enabled() {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(format!("PROXY ATIVO")).color(found_color).size(15.0));
                        });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(format!("PROXY INATIVO")).color(not_found_color).size(15.0));
                        });
                    }
                    if have_intranet_access() {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(format!("ACESSO A REDE INTERNA")).color(found_color).size(15.0));
                        });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(format!("SEM ACESSO A REDE INTERNA")).color(not_found_color).size(15.0));
                        });
                    }
                });
            });
        });
    }
}

fn load_app_icon() -> egui::viewport::IconData {
    let bytes = include_bytes!("assets/icon.ico");
    let cursor = std::io::Cursor::new(&bytes[..]);
    let dir = ico::IconDir::read(cursor).expect("Falha ao ler ícone");
    let entry = dir.entries().first().expect("Ícone vazio");
    let img = entry.decode().expect("Falha ao decodificar ícone");
    let rgba = img.rgba_data().to_vec();
    let width = img.width();
    let height = img.height();
    egui::viewport::IconData { rgba, width, height }
}

fn main() -> eframe::Result<()> {
    let size = [480.0, 495.0];
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_close_button(true)
            .with_maximize_button(false)
            .with_resizable(false)
            .with_icon(load_app_icon()),
        ..Default::default()
    };
    eframe::run_native("N.E.O.N.", options, Box::new(|_cc| Ok(Box::<Neon>::default())))
}
