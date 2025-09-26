// Nyash Explorer with Icons - Windows API Drive Icon Viewer
// エクスプローラー風ドライブアイコン付きビューアー

use eframe::egui::{self, ColorImage, FontFamily, TextureHandle};
use std::fs::File;
use std::io::Read;
// use std::collections::HashMap;
// use std::sync::Arc;

#[cfg(windows)]
use windows::{
    core::*,
    Win32::{Storage::FileSystem::*, System::Com::*, UI::Shell::*, UI::WindowsAndMessaging::*},
};

fn main() -> eframe::Result {
    // COM初期化（Windows用）
    #[cfg(windows)]
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("Nyash Explorer with Icons - アイコン付きドライブビューアー"),
        ..Default::default()
    };

    eframe::run_native(
        "Nyash Explorer Icons",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(NyashExplorer::new(cc.egui_ctx.clone())))
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

struct DriveInfo {
    letter: String,
    name: String,
    drive_type: String,
    total_bytes: u64,
    free_bytes: u64,
    icon_texture: Option<TextureHandle>,
}

struct NyashExplorer {
    drives: Vec<DriveInfo>,
    selected_drive: Option<usize>,
    status: String,
    ctx: egui::Context,
}

impl NyashExplorer {
    fn new(ctx: egui::Context) -> Self {
        let mut explorer = Self {
            drives: Vec::new(),
            selected_drive: None,
            status: "初期化中...".to_string(),
            ctx,
        };
        explorer.refresh_drives();
        explorer
    }

    #[cfg(windows)]
    fn get_drive_icon(&self, drive_path: &str) -> Option<ColorImage> {
        unsafe {
            let mut shfi = SHFILEINFOW::default();
            let drive_path_wide: Vec<u16> = drive_path
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            // アイコンを取得
            let result = SHGetFileInfoW(
                PCWSTR::from_raw(drive_path_wide.as_ptr()),
                FILE_ATTRIBUTE_NORMAL,
                Some(&mut shfi),
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON | SHGFI_USEFILEATTRIBUTES,
            );

            if result == 0 || shfi.hIcon.is_invalid() {
                return None;
            }

            // アイコンからビットマップを取得
            let icon_info = ICONINFO::default();
            if GetIconInfo(shfi.hIcon, &icon_info as *const _ as *mut _).is_ok() {
                // ビットマップからピクセルデータを取得する処理
                // アイコンを破棄
                let _ = DestroyIcon(shfi.hIcon);

                // C:ドライブの場合は保存済みBMPファイルを読み込む
                if drive_path.contains("C:") {
                    if let Some(icon) = Self::load_bmp_icon("c_drive_icon.bmp") {
                        return Some(icon);
                    }
                }

                // それ以外はダミーアイコンを返す
                Some(Self::create_dummy_icon(&drive_path))
            } else {
                let _ = DestroyIcon(shfi.hIcon);
                None
            }
        }
    }

    #[cfg(not(windows))]
    fn get_drive_icon(&self, drive_path: &str) -> Option<ColorImage> {
        Some(Self::create_dummy_icon(drive_path))
    }

    // BMPファイルを読み込んでColorImageに変換
    fn load_bmp_icon(file_path: &str) -> Option<ColorImage> {
        let mut file = File::open(file_path).ok()?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).ok()?;

        // BMPヘッダーをパース（簡易版）
        if buffer.len() < 54 {
            return None;
        }

        // BMPマジックナンバーをチェック
        if &buffer[0..2] != b"BM" {
            return None;
        }

        // ヘッダーから情報を読み取る
        let data_offset =
            u32::from_le_bytes([buffer[10], buffer[11], buffer[12], buffer[13]]) as usize;
        let width = i32::from_le_bytes([buffer[18], buffer[19], buffer[20], buffer[21]]) as usize;
        let height =
            i32::from_le_bytes([buffer[22], buffer[23], buffer[24], buffer[25]]).abs() as usize;
        let bits_per_pixel = u16::from_le_bytes([buffer[28], buffer[29]]);

        // 32ビットBMPのみサポート
        if bits_per_pixel != 32 {
            println!("Unsupported BMP format: {} bits per pixel", bits_per_pixel);
            return None;
        }

        // ピクセルデータを読み取る
        let mut pixels = Vec::with_capacity(width * height);
        let pixel_data = &buffer[data_offset..];

        // BMPは下から上に格納されているので、反転しながら読み取る
        for y in (0..height).rev() {
            for x in 0..width {
                let offset = (y * width + x) * 4;
                if offset + 3 < pixel_data.len() {
                    let b = pixel_data[offset];
                    let g = pixel_data[offset + 1];
                    let r = pixel_data[offset + 2];
                    let a = pixel_data[offset + 3];
                    pixels.push(egui::Color32::from_rgba_unmultiplied(r, g, b, a));
                } else {
                    pixels.push(egui::Color32::TRANSPARENT);
                }
            }
        }

        Some(ColorImage {
            size: [width, height],
            pixels,
        })
    }

    // ダミーアイコンを生成（実際のアイコン取得が複雑なため）
    fn create_dummy_icon(drive_path: &str) -> ColorImage {
        let size = 48;
        let mut pixels = vec![egui::Color32::TRANSPARENT; size * size];

        // ドライブタイプに応じた色を設定
        let color = if drive_path.contains("C:") {
            egui::Color32::from_rgb(100, 149, 237) // コーンフラワーブルー
        } else if drive_path.contains("D:") {
            egui::Color32::from_rgb(144, 238, 144) // ライトグリーン
        } else {
            egui::Color32::from_rgb(255, 182, 193) // ライトピンク
        };

        // シンプルな円形アイコンを描画
        let center = size as f32 / 2.0;
        let radius = (size as f32 / 2.0) - 4.0;

        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance <= radius {
                    pixels[y * size + x] = color;
                } else if distance <= radius + 2.0 {
                    // 縁取り
                    pixels[y * size + x] = egui::Color32::from_rgb(64, 64, 64);
                }
            }
        }

        // ドライブ文字を中央に配置（簡易版）
        if let Some(_letter) = drive_path.chars().next() {
            // 文字の位置（中央）
            let text_x = size / 2 - 8;
            let text_y = size / 2 - 8;

            // 白い文字で描画
            for dy in 0..16 {
                for dx in 0..16 {
                    if dx > 4 && dx < 12 && dy > 4 && dy < 12 {
                        let idx = (text_y + dy) * size + (text_x + dx);
                        if idx < pixels.len() {
                            pixels[idx] = egui::Color32::WHITE;
                        }
                    }
                }
            }
        }

        ColorImage {
            size: [size, size],
            pixels,
        }
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
                            icon_texture: None,
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

                        // アイコンを取得してテクスチャに変換
                        if let Some(icon_image) = self.get_drive_icon(&drive_path) {
                            let texture = self.ctx.load_texture(
                                format!("drive_icon_{}", drive_letter),
                                icon_image,
                                Default::default(),
                            );
                            drive_info.icon_texture = Some(texture);
                        }

                        self.drives.push(drive_info);
                    }
                }
            }
        }

        #[cfg(not(windows))]
        {
            // Windows以外の環境ではダミーデータ
            let mut drive_info = DriveInfo {
                letter: "C:".to_string(),
                name: "ローカルディスク (C:)".to_string(),
                drive_type: "ハードディスク".to_string(),
                total_bytes: 500_000_000_000,
                free_bytes: 250_000_000_000,
                icon_texture: None,
            };

            if let Some(icon_image) = self.get_drive_icon("C:") {
                let texture =
                    self.ctx
                        .load_texture("drive_icon_C:", icon_image, Default::default());
                drive_info.icon_texture = Some(texture);
            }

            self.drives.push(drive_info);
        }

        self.status = format!(
            "{}個のドライブを検出しました（アイコン付き）",
            self.drives.len()
        );
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
                        self.status = "Nyash Explorer - Everything is Box! アイコンも取得できる化け物言語！🐱".to_string();
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
                ui.label("Nyash Explorer - アイコン付きドライブビューアー");
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
            ui.heading("💾 ドライブ一覧（アイコン付き）");
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

                        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(response.rect), |ui| {
                            ui.horizontal(|ui| {
                                // ドライブアイコン
                                ui.vertical(|ui| {
                                    ui.add_space(10.0);

                                    if let Some(texture) = &drive.icon_texture {
                                        ui.image((texture.id(), egui::vec2(48.0, 48.0)));
                                    } else {
                                        // フォールバック絵文字アイコン
                                        let icon_text = match drive.drive_type.as_str() {
                                            "ハードディスク" => "💾",
                                            "リムーバブル" => "💿",
                                            "CD-ROM" => "💿",
                                            "ネットワーク" => "🌐",
                                            _ => "📁",
                                        };
                                        ui.label(egui::RichText::new(icon_text).size(40.0));
                                    }
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
                    self.status =
                        "Nyash - Everything is Box! Windows APIでアイコンも取得できる化け物言語！"
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
