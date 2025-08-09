// Debug version to check input issues
use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init(); // Enable logging
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 480.0])
            .with_title("Debug Notepad"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Debug Notepad",
        options,
        Box::new(|_cc| Ok(Box::new(DebugApp::default()))),
    )
}

#[derive(Default)]
struct DebugApp {
    text: String,
    single_line: String,
    event_log: Vec<String>,
}

impl eframe::App for DebugApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Debug Text Input Test");
            
            // Single line input
            ui.horizontal(|ui| {
                ui.label("Single Line:");
                let response = ui.text_edit_singleline(&mut self.single_line);
                if response.changed() {
                    self.event_log.push(format!("Single line changed: '{}'", self.single_line));
                }
            });
            
            ui.separator();
            
            // Multi line input
            ui.label("Multi Line:");
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.text)
                    .desired_width(f32::INFINITY)
                    .desired_rows(10)
            );
            
            if response.changed() {
                self.event_log.push(format!("Multi line changed: {} chars", self.text.len()));
            }
            
            ui.separator();
            
            // Show input events
            ui.label("Event Log:");
            egui::ScrollArea::vertical()
                .max_height(100.0)
                .show(ui, |ui| {
                    for event in &self.event_log {
                        ui.label(event);
                    }
                });
            
            // Debug info
            ui.separator();
            ui.label(format!("Text length: {}", self.text.len()));
            ui.label(format!("Single line length: {}", self.single_line.len()));
            
            // Test buttons
            if ui.button("Add Test Text").clicked() {
                self.text.push_str("Test ");
                self.event_log.push("Button: Added test text".to_string());
            }
            
            if ui.button("Clear All").clicked() {
                self.text.clear();
                self.single_line.clear();
                self.event_log.push("Button: Cleared all".to_string());
            }
        });
    }
}