// Nyash Explorer - Windows API Drive Information Viewer
// エクスプローラー風ドライブ情報ビューアー

use eframe::egui::{self, FontFamily};
use std::path::PathBuf;

#[cfg(windows)]
use windows::{
    core::*,
    Win32::{
        Foundation::*, Storage::FileSystem::*, System::Com::*, UI::Shell::*,
        UI::WindowsAndMessaging::*,
    },
};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("Nyash Explorer - ドライブ情報ビューアー"),
        ..Default::default()
    };

    eframe::run_native(
        "Nyash Explorer",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(NyashExplorer::new()))
        }),
    )
}

// フォント設定
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "noto_sans_jp".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/NotoSansJP-VariableFont_wght.ttf"))
            .into(),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans_jp".to_owned());

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("noto_sans_jp".to_owned());

    ctx.set_fonts(fonts);
}

#[derive(Debug)]
struct DriveInfo {
    letter: String,
    name: String,
    drive_type: String,
    total_bytes: u64,
    free_bytes: u64,
    icon_data: Option<Vec<u8>>,
}

struct NyashExplorer {
    drives: Vec<DriveInfo>,
    selected_drive: Option<usize>,
    status: String,
}

impl NyashExplorer {
    fn new() -> Self {
        let mut explorer = Self {
            drives: Vec::new(),
            selected_drive: None,
            status: "初期化中...".to_string(),
        };
        explorer.refresh_drives();
        explorer
    }

    fn refresh_drives(&mut self) {
        self.drives.clear();
        self.status = "ドライブ情報を取得中...".to_string();

        #[cfg(windows)]
        {
            unsafe {
                // 論理ドライブのビットマスクを取得
                let drives_mask = GetLogicalDrives();

                for i in 0..26 {
                    if drives_mask & (1 << i) != 0 {
                        let drive_letter = format!("{}:", (b'A' + i) as char);
                        let drive_path = format!("{}\\", drive_letter);

                        // ドライブ情報を取得
                        let mut drive_info = DriveInfo {
                            letter: drive_letter.clone(),
                            name: String::new(),
                            drive_type: String::new(),
                            total_bytes: 0,
                            free_bytes: 0,
                            icon_data: None,
                        };

                        // ドライブタイプを取得
                        let drive_type_code = GetDriveTypeW(PCWSTR::from_raw(
                            format!("{}\0", drive_path)
                                .encode_utf16()
                                .collect::<Vec<u16>>()
                                .as_ptr(),
                        ));

                        drive_info.drive_type = match drive_type_code {
                            DRIVE_REMOVABLE => "リムーバブル".to_string(),
                            DRIVE_FIXED => "ハードディスク".to_string(),
                            DRIVE_REMOTE => "ネットワーク".to_string(),
                            DRIVE_CDROM => "CD-ROM".to_string(),
                            DRIVE_RAMDISK => "RAMディスク".to_string(),
                            _ => "不明".to_string(),
                        };

                        // ボリューム情報を取得
                        let mut volume_name = vec![0u16; 256];
                        let mut file_system = vec![0u16; 256];
                        let mut serial_number = 0u32;
                        let mut max_component_len = 0u32;
                        let mut file_system_flags = 0u32;

                        if GetVolumeInformationW(
                            PCWSTR::from_raw(
                                format!("{}\0", drive_path)
                                    .encode_utf16()
                                    .collect::<Vec<u16>>()
                                    .as_ptr(),
                            ),
                            Some(&mut volume_name),
                            Some(&mut serial_number),
                            Some(&mut max_component_len),
                            Some(&mut file_system_flags),
                            Some(&mut file_system),
                        )
                        .is_ok()
                        {
                            let volume_name_str = String::from_utf16_lossy(&volume_name)
                                .trim_end_matches('\0')
                                .to_string();
                            drive_info.name = if volume_name_str.is_empty() {
                                format!("ローカルディスク ({})", drive_letter)
                            } else {
                                format!("{} ({})", volume_name_str, drive_letter)
                            };
                        } else {
                            drive_info.name = format!("ドライブ ({})", drive_letter);
                        }

                        // 空き容量を取得
                        let mut free_bytes_available = 0u64;
                        let mut total_bytes = 0u64;
                        let mut total_free_bytes = 0u64;

                        if GetDiskFreeSpaceExW(
                            PCWSTR::from_raw(
                                format!("{}\0", drive_path)
                                    .encode_utf16()
                                    .collect::<Vec<u16>>()
                                    .as_ptr(),
                            ),
                            Some(&mut free_bytes_available),
                            Some(&mut total_bytes),
                            Some(&mut total_free_bytes),
                        )
                        .is_ok()
                        {
                            drive_info.total_bytes = total_bytes;
                            drive_info.free_bytes = total_free_bytes;
                        }

                        self.drives.push(drive_info);
                    }
                }
            }
        }

        #[cfg(not(windows))]
        {
            // Windows以外の環境ではダミーデータ
            self.drives.push(DriveInfo {
                letter: "C:".to_string(),
                name: "ローカルディスク (C:)".to_string(),
                drive_type: "ハードディスク".to_string(),
                total_bytes: 500_000_000_000,
                free_bytes: 250_000_000_000,
                icon_data: None,
            });
        }

        self.status = format!("{}個のドライブを検出しました", self.drives.len());
    }

    fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

impl eframe::App for NyashExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // メニューバー
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("ファイル", |ui| {
                    if ui.button("更新").clicked() {
                        self.refresh_drives();
                    }
                    ui.separator();
                    if ui.button("終了").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("表示", |ui| {
                    if ui.button("大きいアイコン").clicked() {
                        self.status = "表示モード: 大きいアイコン".to_string();
                    }
                    if ui.button("詳細").clicked() {
                        self.status = "表示モード: 詳細".to_string();
                    }
                });

                ui.menu_button("ヘルプ", |ui| {
                    if ui.button("Nyash Explorerについて").clicked() {
                        self.status = "Nyash Explorer - Everything is Box! 🐱".to_string();
                    }
                });
            });
        });

        // ツールバー
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("🔄 更新").clicked() {
                    self.refresh_drives();
                }
                ui.separator();
                ui.label("Nyash Explorer - ドライブ情報ビューアー");
            });
        });

        // ステータスバー
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("ドライブ数: {}", self.drives.len()));
                });
            });
        });

        // メインパネル - ドライブ一覧
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("💾 ドライブ一覧");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, drive) in self.drives.iter().enumerate() {
                    let is_selected = self.selected_drive == Some(index);

                    ui.group(|ui| {
                        let response = ui.allocate_response(
                            egui::vec2(ui.available_width(), 100.0),
                            egui::Sense::click(),
                        );

                        if response.clicked() {
                            self.selected_drive = Some(index);
                            self.status = format!("{} を選択しました", drive.name);
                        }

                        // 背景色
                        if is_selected {
                            ui.painter().rect_filled(
                                response.rect,
                                0.0,
                                egui::Color32::from_rgb(100, 149, 237).gamma_multiply(0.2),
                            );
                        }

                        ui.allocate_ui_at_rect(response.rect, |ui| {
                            ui.horizontal(|ui| {
                                // ドライブアイコン（仮）
                                ui.vertical(|ui| {
                                    ui.add_space(10.0);
                                    let icon_text = match drive.drive_type.as_str() {
                                        "ハードディスク" => "💾",
                                        "リムーバブル" => "💿",
                                        "CD-ROM" => "💿",
                                        "ネットワーク" => "🌐",
                                        _ => "📁",
                                    };
                                    ui.label(egui::RichText::new(icon_text).size(40.0));
                                });

                                ui.add_space(20.0);

                                // ドライブ情報
                                ui.vertical(|ui| {
                                    ui.add_space(10.0);
                                    ui.label(egui::RichText::new(&drive.name).size(16.0).strong());
                                    ui.label(format!("種類: {}", drive.drive_type));

                                    if drive.total_bytes > 0 {
                                        let used_bytes = drive.total_bytes - drive.free_bytes;
                                        let usage_percent =
                                            (used_bytes as f32 / drive.total_bytes as f32) * 100.0;

                                        ui.horizontal(|ui| {
                                            ui.label(format!(
                                                "使用領域: {} / {} ({:.1}%)",
                                                Self::format_bytes(used_bytes),
                                                Self::format_bytes(drive.total_bytes),
                                                usage_percent
                                            ));
                                        });

                                        // 使用率バー
                                        let bar_width = 200.0;
                                        let bar_height = 10.0;
                                        let (rect, _response) = ui.allocate_exact_size(
                                            egui::vec2(bar_width, bar_height),
                                            egui::Sense::hover(),
                                        );

                                        // 背景
                                        ui.painter().rect_filled(
                                            rect,
                                            2.0,
                                            egui::Color32::from_gray(60),
                                        );

                                        // 使用領域
                                        let used_width = bar_width * (usage_percent / 100.0);
                                        let used_rect = egui::Rect::from_min_size(
                                            rect.min,
                                            egui::vec2(used_width, bar_height),
                                        );
                                        let color = if usage_percent > 90.0 {
                                            egui::Color32::from_rgb(255, 0, 0)
                                        } else if usage_percent > 75.0 {
                                            egui::Color32::from_rgb(255, 165, 0)
                                        } else {
                                            egui::Color32::from_rgb(0, 128, 255)
                                        };
                                        ui.painter().rect_filled(used_rect, 2.0, color);
                                    }
                                });
                            });
                        });
                    });

                    ui.add_space(5.0);
                }
            });

            // クイックアクション
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("🐱 Nyashについて").clicked() {
                    self.status = "Nyash - Everything is Box! Windows APIも吸収できる化け物言語！"
                        .to_string();
                }

                if ui.button("📊 システム情報").clicked() {
                    let total: u64 = self.drives.iter().map(|d| d.total_bytes).sum();
                    let free: u64 = self.drives.iter().map(|d| d.free_bytes).sum();
                    self.status = format!(
                        "総容量: {} / 空き容量: {}",
                        Self::format_bytes(total),
                        Self::format_bytes(free)
                    );
                }
            });
        });
    }
}
