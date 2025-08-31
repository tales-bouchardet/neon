#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod functions;
use eframe::egui::{self, CentralPanel, Color32, RichText, Layout, Align, FontDefinitions, FontData, FontFamily, FontId};
use crate::functions::*;

#[derive(Default)]
struct Neon;

impl eframe::App for Neon {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        let text_color = Color32::BLACK;
        let label_color = Color32::from_black_alpha(210);
        let green_color = Color32::from_rgb(0, 184, 107);
        let red_color = Color32::from_rgb(228, 30, 33);
        let shadow_color = Color32::LIGHT_GRAY;
        let white_color = Color32::WHITE;
        let light_gray_color = Color32::LIGHT_GRAY;

    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "segoe_ui_bold".to_owned(),
        FontData::from_owned(include_bytes!("assets/SEGOEUIB.TTF").to_vec()).into(),
    );
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "segoe_ui_bold".to_owned());
    ctx.set_fonts(fonts);

    fn group<R>(
        ui: &mut egui::Ui,
        shadow_color: Color32,
        white_color: Color32,
        light_gray_color: Color32,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        egui::Frame::group(ui.style())
            .corner_radius(10)
            .fill(white_color)
            .stroke(egui::Stroke::new(1.0, light_gray_color))
            .shadow(egui::Shadow {
                offset: [0, 2],
                blur: 5,
                spread: 0,
                color: shadow_color,
            })
            .show(ui, |ui| {
                ui.set_width(420.0);
                add_contents(ui)
            })
            .inner
    }

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                group(ui, shadow_color, white_color, light_gray_color,|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(format!("{}", get_now())).color(label_color).font(FontId::new(20.0, FontFamily::Monospace)));
                    });
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(format!("{}", get_date())).color(label_color).font(FontId::new(20.0, FontFamily::Monospace)));
                    });
                });
                ui.add_space(5.0);
                group(ui, shadow_color, white_color, light_gray_color,|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("FABRICANTE:").color(label_color).font(FontId::new(15.0, FontFamily::Monospace)));
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            ui.label(RichText::new(format!("{}", get_manufacturer())).color(text_color).size(15.0));
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("SO:").color(label_color).font(FontId::new(15.0, FontFamily::Monospace)));
                        if let Some((os, arch)) = get_os() {
                            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                                ui.label(RichText::new(format!("{} ({})", os, arch)).color(text_color).size(15.0));
                            });
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("USUÁRIO:").color(label_color).font(FontId::new(15.0, FontFamily::Monospace)));
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            ui.label(RichText::new(get_current_user()).color(text_color).size(15.0));
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("HOSTNAME:").color(label_color).font(FontId::new(15.0, FontFamily::Monospace)));
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            ui.label(RichText::new(format!("{}", get_hostname())).color(text_color).size(15.0));
                        });
                    });
                });
                ui.add_space(5.0);
                group(ui, shadow_color, white_color, light_gray_color,|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("CPU:").color(label_color).font(FontId::new(15.0, FontFamily::Monospace)));
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            ui.label(RichText::new(format!("{}", get_cpu_info())).color(text_color).size(15.0));
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("USO DE CPU:").color(label_color).font(FontId::new(15.0, FontFamily::Monospace)));
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            ui.label(RichText::new(format!("{}", get_cpu_usage())).color(text_color).size(15.0));
                        });
                    });
                });
                ui.add_space(5.0);
                group(ui, shadow_color, white_color, light_gray_color, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("RAM:").color(label_color).font(FontId::new(15.0, FontFamily::Monospace)));
                            let (total, _avail) = get_memory_info();
                            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                                ui.label(RichText::new(total).color(text_color).size(15.0));
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("RAM DISP.:").color(label_color).font(FontId::new(15.0, FontFamily::Monospace)));
                            let (_, avail) = get_memory_info();
                            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                                ui.label(RichText::new(avail).color(text_color).size(15.0));
                            });
                    });
                });
                ui.add_space(5.0);
                group(ui, shadow_color, white_color, light_gray_color, |ui| {
                    ui.horizontal(|ui| {
                        if is_firewall_enabled() {
                            ui.label(RichText::new("Firewall:").color(red_color).font(FontId::new(15.0, FontFamily::Monospace)));
                            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                                ui.label(RichText::new("Ativado").color(text_color).size(15.0));
                            });
                        } else {
                            ui.label(RichText::new("Firewall:").color(green_color).font(FontId::new(15.0, FontFamily::Monospace)));
                            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                                ui.label(RichText::new("Desativado").color(text_color).size(15.0));
                            });
                        }
                    });
                    ui.horizontal(|ui| {
                        let forti = get_file_version("C:\\Program Files\\Fortinet\\FortiClient\\FortiClient.exe").unwrap_or_else(|_| "Não Instalado".to_string());
                        let label_col = if forti.contains("Não Instalado") { red_color } else { green_color };
                        ui.label(RichText::new("FortiClient:").color(label_col).font(FontId::new(15.0, FontFamily::Monospace)));
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            ui.label(RichText::new(format!("{}", forti)).color(text_color).size(15.0));
                        });
                    });

                    ui.horizontal(|ui| {
                        let cs = get_file_version("C:\\Program Files\\CrowdStrike\\CrowdStrike.exe").unwrap_or_else(|_| "Não Instalado".to_string());
                        let label_col = if cs.contains("Não Instalado") { red_color } else { green_color };
                        ui.label(RichText::new("CrowdStrike:").color(label_col).font(FontId::new(15.0, FontFamily::Monospace)));
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            ui.label(RichText::new(format!("{}", cs)).color(text_color).size(15.0));
                        });
                    });

                    ui.horizontal(|ui| {
                        let kace = get_file_version("C:\\Program Files (x86)\\Quest\\KACE\\AMPTools.exe").unwrap_or_else(|_| "Não Instalado".to_string());
                        let label_col = if kace.contains("Não Instalado") { red_color } else { green_color };
                        ui.label(RichText::new("KACE:").color(label_col).font(FontId::new(15.0, FontFamily::Monospace)));
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            ui.label(RichText::new(format!("{}", kace)).color(text_color).size(15.0));
                        });
                    });
                    
                    ui.horizontal(|ui| {
                        let netskope = get_file_version("C:\\Program Files\\Netskope\\EPDLP\\EPDLP.exe").unwrap_or_else(|_| "Não Instalado".to_string());
                        let label_col = if netskope.contains("Não Instalado") { red_color } else { green_color };
                        ui.label(RichText::new("Netskope:").color(label_col).font(FontId::new(15.0, FontFamily::Monospace)));
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            ui.label(RichText::new(format!("{}", netskope)).color(text_color).size(15.0));
                        });
                    });
                });
                ui.add_space(5.0);
                group(ui, shadow_color, white_color, light_gray_color, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("EM ATIVIDADE:").color(label_color).font(FontId::new(15.0, FontFamily::Monospace)));
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            ui.label(RichText::new(format!("{}", get_uptime())).color(text_color).size(15.0));
                        });
                    });
                });
                ui.add_space(5.0);
                group(ui, shadow_color, white_color, light_gray_color, |ui| {
                    let join_info = get_join_info();
                    if join_info == "NÃO GERENCIADA" {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(format!("{}", join_info)).color(red_color).font(FontId::new(15.0, FontFamily::Monospace)));
                        });
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new(format!("{}", join_info)).color(green_color).font(FontId::new(15.0, FontFamily::Monospace)));
                            });
                        }
                        if is_proxy_enabled() {
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new(format!("PROXY ATIVO")).color(green_color).font(FontId::new(15.0, FontFamily::Monospace)));
                            });
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new(format!("PROXY INATIVO")).color(red_color).font(FontId::new(15.0, FontFamily::Monospace)));
                            });
                        }
                        if have_intranet_access() {
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new(format!("ACESSO A REDE INTERNA")).color(green_color).font(FontId::new(15.0, FontFamily::Monospace)));
                            });
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new(format!("SEM ACESSO A REDE INTERNA")).color(red_color).font(FontId::new(15.0, FontFamily::Monospace)));
                            });
                        }
                    });
                ui.add_space(5.0);
                if ui.add_sized(
                    [433.5, 30.0],
                    egui::Button::new(
                        RichText::new("COPIAR PARA CLIPBOARD")
                            .color(white_color)
                            .font(FontId::new(12.0, FontFamily::Monospace)),
                    )
                    .fill(text_color)
                    .corner_radius(8)
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked() {
                    screenshot().unwrap();
                }
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
    let size = [450.0, 630.0];
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
