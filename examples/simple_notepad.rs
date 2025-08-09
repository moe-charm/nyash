// Nyash + egui でWindowsメモ帳アプリ
// テキスト入力機能付きのシンプルなGUIアプリケーション

use eframe::egui;

fn main() -> eframe::Result {
    // Windows用の設定
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 480.0])
            .with_title("Nyash Notepad"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Nyash Notepad",
        options,
        Box::new(|_cc| Ok(Box::new(NyashNotepad::default()))),
    )
}

#[derive(Default)]
struct NyashNotepad {
    text: String,
    status: String,
}

impl eframe::App for NyashNotepad {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // メニューバー
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("ファイル", |ui| {
                    if ui.button("New").clicked() {
                        self.text.clear();
                        self.status = "Newファイルを作成しました".to_string();
                    }
                    if ui.button("クリア").clicked() {
                        self.text.clear();
                        self.status = "Text cleared".to_string();
                    }
                    ui.separator();
                    if ui.button("終了").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("編集", |ui| {
                    if ui.button("すべて選択").clicked() {
                        // TODO: テキストエリア全選択
                        self.status = "すべて選択（未実装）".to_string();
                    }
                });
                
                ui.menu_button("ヘルプ", |ui| {
                    if ui.button("Nyashについて").clicked() {
                        self.status = "Nyash - Everything is Box! 🐱".to_string();
                    }
                });
            });
        });
        
        // ステータスバー
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("文字数: {}", self.text.chars().count()));
                });
            });
        });
        
        // メインのテキストエディタ
        egui::CentralPanel::default().show(ctx, |ui| {
            // ツールバー
            ui.horizontal(|ui| {
                if ui.button("🗒️ クリア").clicked() {
                    self.text.clear();
                    self.status = "テキストをクリアしました".to_string();
                }
                
                ui.separator();
                
                if ui.button("📋 コピー").clicked() {
                    ui.output_mut(|o| o.copied_text = self.text.clone());
                    self.status = "テキストをコピーしました".to_string();
                }
                
                if ui.button("✂️ カット").clicked() {
                    ui.output_mut(|o| o.copied_text = self.text.clone());
                    self.text.clear();
                    self.status = "テキストをカットしました".to_string();
                }
                
                if ui.button("📄 ペースト").clicked() {
                    // egui 0.29ではクリップボードAPIが変更されている
                    self.status = "ペースト機能（簡易版）".to_string();
                }
            });
            
            ui.separator();
            
            // テキストエディタ本体
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.text)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20)
                        .hint_text("ここにテキストを入力してください... にゃ！")
                );
            });
            
            // サンプルボタン
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Nyashサンプル挿入").clicked() {
                    self.text.push_str("\n// Nyash - Everything is Box!\n");
                    self.text.push_str("box HelloWorld {\n");
                    self.text.push_str("    init { message }\n");
                    self.text.push_str("    \n");
                    self.text.push_str("    HelloWorld() {\n");
                    self.text.push_str("        me.message = \"Hello, Nyash World! にゃ！\"\n");
                    self.text.push_str("    }\n");
                    self.text.push_str("}\n");
                    self.status = "Nyashサンプルコードを挿入しました".to_string();
                }
                
                if ui.button("時刻挿入").clicked() {
                    let now = chrono::Local::now();
                    self.text.push_str(&format!("\n{}\n", now.format("%Y-%m-%d %H:%M:%S")));
                    self.status = "現在時刻を挿入しました".to_string();
                }
            });
        });
    }
}