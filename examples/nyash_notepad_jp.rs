// Nyash + egui Windows Notepad App - Japanese Font Support
// 日本語フォント対応版のGUIメモ帳アプリ

use eframe::egui::{self, FontFamily};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Nyash Notepad - にゃっしゅメモ帳"),
        ..Default::default()
    };

    eframe::run_native(
        "Nyash Notepad JP",
        options,
        Box::new(|cc| {
            // 日本語フォントを設定
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(NyashNotepad::default()))
        }),
    )
}

// フォント設定用の関数
fn setup_custom_fonts(ctx: &egui::Context) {
    // フォント設定を取得
    let mut fonts = egui::FontDefinitions::default();

    // 日本語フォント（可変ウェイト）を追加
    fonts.font_data.insert(
        "noto_sans_jp".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/NotoSansJP-VariableFont_wght.ttf"))
            .into(),
    );

    // フォントファミリーに追加
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans_jp".to_owned()); // 一番優先度高く追加

    // モノスペースフォントにも日本語フォントを追加
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("noto_sans_jp".to_owned());

    // フォント設定を適用
    ctx.set_fonts(fonts);
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
                    if ui.button("新規作成").clicked() {
                        self.text.clear();
                        self.status = "新規ファイルを作成しました".to_string();
                    }
                    if ui.button("テキストクリア").clicked() {
                        self.text.clear();
                        self.status = "テキストをクリアしました".to_string();
                    }
                    ui.separator();
                    if ui.button("終了").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("編集", |ui| {
                    if ui.button("すべて選択").clicked() {
                        self.status = "すべて選択（未実装）".to_string();
                    }
                    if ui.button("検索").clicked() {
                        self.status = "検索機能（未実装）".to_string();
                    }
                });

                ui.menu_button("ヘルプ", |ui| {
                    if ui.button("Nyashについて").clicked() {
                        self.status = "Nyash - Everything is Box! 🐱".to_string();
                    }
                    if ui.button("使い方").clicked() {
                        self.status =
                            "テキストを入力して、にゃっしゅプログラムを書こう！".to_string();
                    }
                });
            });
        });

        // ステータスバー
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "文字数: {} | 行数: {}",
                        self.text.chars().count(),
                        self.text.lines().count()
                    ));
                });
            });
        });

        // メインのテキストエディタ
        egui::CentralPanel::default().show(ctx, |ui| {
            // ツールバー
            ui.horizontal(|ui| {
                if ui.button("🗑️ クリア").clicked() {
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
                    self.status = "ペースト機能（簡易版）".to_string();
                }

                ui.separator();

                if ui.button("🔤 フォント大").clicked() {
                    ctx.set_zoom_factor(ctx.zoom_factor() * 1.1);
                    self.status = "フォントサイズを拡大しました".to_string();
                }

                if ui.button("🔡 フォント小").clicked() {
                    ctx.set_zoom_factor(ctx.zoom_factor() * 0.9);
                    self.status = "フォントサイズを縮小しました".to_string();
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
                        .hint_text("ここにテキストを入力してください... にゃ！🐱"),
                );
            });

            // サンプルボタン
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("クイック挿入: ");

                if ui.button("📝 Nyashサンプル").clicked() {
                    self.text.push_str(
                        "\n// Nyash - Everything is Box! すべてがBoxの世界へようこそ！\n",
                    );
                    self.text.push_str("box こんにちは世界 {\n");
                    self.text.push_str("    init { メッセージ }\n");
                    self.text.push_str("    \n");
                    self.text.push_str("    こんにちは世界() {\n");
                    self.text.push_str(
                        "        me.メッセージ = \"こんにちは、Nyashの世界！にゃ〜！🐱\"\n",
                    );
                    self.text.push_str("    }\n");
                    self.text.push_str("    \n");
                    self.text.push_str("    挨拶() {\n");
                    self.text.push_str("        print(me.メッセージ)\n");
                    self.text.push_str("    }\n");
                    self.text.push_str("}\n\n");
                    self.text.push_str("// 使い方:\n");
                    self.text.push_str("local hello\n");
                    self.text.push_str("hello = new こんにちは世界()\n");
                    self.text.push_str("hello.挨拶()\n");
                    self.status = "Nyashサンプルコードを挿入しました".to_string();
                }

                if ui.button("🕐 現在時刻").clicked() {
                    let now = chrono::Local::now();
                    self.text.push_str(&format!(
                        "\n// 挿入時刻: {}\n",
                        now.format("%Y年%m月%d日 %H時%M分%S秒")
                    ));
                    self.status = "現在時刻を挿入しました".to_string();
                }

                if ui.button("🐱 ASCIIにゃんこ").clicked() {
                    self.text.push_str("\n/*\n");
                    self.text.push_str("    /\\_/\\  \n");
                    self.text.push_str("   ( o.o ) < にゃ〜！\n");
                    self.text.push_str("    > ^ <  \n");
                    self.text.push_str("   Nyash!  \n");
                    self.text.push_str("*/\n");
                    self.status = "にゃんこを挿入しました - にゃ！".to_string();
                }
            });
        });
    }
}
