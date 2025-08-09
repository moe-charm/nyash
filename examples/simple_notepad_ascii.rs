// Nyash + egui Windows Notepad App - ASCII Only Version
// Simple GUI application with text input functionality

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 480.0])
            .with_title("Nyash Notepad - ASCII Version"),
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
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.text.clear();
                        self.status = "New file created".to_string();
                    }
                    if ui.button("Clear").clicked() {
                        self.text.clear();
                        self.status = "Text cleared".to_string();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    if ui.button("Select All").clicked() {
                        self.status = "Select All (not implemented)".to_string();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About Nyash").clicked() {
                        self.status = "Nyash - Everything is Box! (^-^)".to_string();
                    }
                });
            });
        });
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Characters: {}", self.text.chars().count()));
                });
            });
        });
        
        // Main text editor
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title
            ui.heading("=== Nyash Text Editor ===");
            
            // Toolbar without emojis
            ui.horizontal(|ui| {
                if ui.button("[X] Clear").clicked() {
                    self.text.clear();
                    self.status = "Text cleared".to_string();
                }
                
                ui.separator();
                
                if ui.button("[C] Copy").clicked() {
                    ui.output_mut(|o| o.copied_text = self.text.clone());
                    self.status = "Text copied to clipboard".to_string();
                }
                
                if ui.button("[X] Cut").clicked() {
                    ui.output_mut(|o| o.copied_text = self.text.clone());
                    self.text.clear();
                    self.status = "Text cut to clipboard".to_string();
                }
                
                if ui.button("[V] Paste").clicked() {
                    self.status = "Paste (simplified version)".to_string();
                }
                
                ui.separator();
                
                if ui.button("[?] Help").clicked() {
                    self.status = "Nyash Notepad v1.0 - Everything is Box!".to_string();
                }
            });
            
            ui.separator();
            
            // Text editor body
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.text)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20)
                        .hint_text("Type your text here... nya!")
                );
            });
            
            // Sample buttons
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Quick Insert: ");
                
                if ui.button("Nyash Sample Code").clicked() {
                    self.text.push_str("\n// Nyash - Everything is Box!\n");
                    self.text.push_str("box HelloWorld {\n");
                    self.text.push_str("    init { message }\n");
                    self.text.push_str("    \n");
                    self.text.push_str("    HelloWorld() {\n");
                    self.text.push_str("        me.message = \"Hello, Nyash World! nya!\"\n");
                    self.text.push_str("    }\n");
                    self.text.push_str("    \n");
                    self.text.push_str("    greet() {\n");
                    self.text.push_str("        print(me.message)\n");
                    self.text.push_str("    }\n");
                    self.text.push_str("}\n\n");
                    self.text.push_str("// Usage:\n");
                    self.text.push_str("local hello\n");
                    self.text.push_str("hello = new HelloWorld()\n");
                    self.text.push_str("hello.greet()\n");
                    self.status = "Nyash sample code inserted".to_string();
                }
                
                if ui.button("Current Time").clicked() {
                    let now = chrono::Local::now();
                    self.text.push_str(&format!("\n[{}]\n", now.format("%Y-%m-%d %H:%M:%S")));
                    self.status = "Timestamp inserted".to_string();
                }
                
                if ui.button("ASCII Art Cat").clicked() {
                    self.text.push_str("\n");
                    self.text.push_str("    /\\_/\\  \n");
                    self.text.push_str("   ( o.o ) \n");
                    self.text.push_str("    > ^ <  \n");
                    self.text.push_str("   Nyash!  \n");
                    self.text.push_str("\n");
                    self.status = "ASCII cat inserted - nya!".to_string();
                }
            });
        });
    }
}