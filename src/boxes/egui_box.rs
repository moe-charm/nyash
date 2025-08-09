// Nyash EguiBox Implementation
// Everything is Box哲学によるGUIフレームワーク統合
// 「なんでもBoxにできる」化け物言語の第一歩！

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use crate::interpreter::RuntimeError;
use std::any::Any;
use std::sync::{Arc, Mutex};
use eframe::{self, epaint::Vec2};

/// EguiBox - GUI アプリケーションを包むBox
/// 
/// # 使用例
/// ```nyash
/// app = new EguiBox()
/// app.setTitle("My Nyash App")
/// app.setSize(800, 600)
/// app.run()
/// ```
pub struct EguiBox {
    title: String,
    size: Vec2,
    app_state: Arc<Mutex<Box<dyn Any + Send>>>,
    update_fn: Option<Arc<dyn Fn(&mut Box<dyn Any + Send>, &egui::Context) + Send + Sync>>,
}

impl std::fmt::Debug for EguiBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiBox")
            .field("title", &self.title)
            .field("size", &self.size)
            .finish()
    }
}

impl EguiBox {
    pub fn new() -> Self {
        Self {
            title: "Nyash GUI Application".to_string(),
            size: Vec2::new(800.0, 600.0),
            app_state: Arc::new(Mutex::new(Box::new(()) as Box<dyn Any + Send>)),
            update_fn: None,
        }
    }
    
    /// アプリケーション状態を設定
    pub fn set_app_state<T: Any + Send + 'static>(&mut self, state: T) {
        self.app_state = Arc::new(Mutex::new(Box::new(state)));
    }
    
    /// 更新関数を設定
    pub fn set_update_fn<F>(&mut self, f: F) 
    where 
        F: Fn(&mut Box<dyn Any + Send>, &egui::Context) + Send + Sync + 'static
    {
        self.update_fn = Some(Arc::new(f));
    }
}

// NyashApp - eframe::Appを実装する内部構造体
struct NyashApp {
    app_state: Arc<Mutex<Box<dyn Any + Send>>>,
    update_fn: Arc<dyn Fn(&mut Box<dyn Any + Send>, &egui::Context) + Send + Sync>,
}

impl eframe::App for NyashApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(mut state) = self.app_state.lock() {
            (self.update_fn)(&mut *state, ctx);
        }
    }
}

impl NyashBox for EguiBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(
            format!("EguiBox('{}', {}x{})", self.title, self.size.x, self.size.y)
        )
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // GUI Boxはクローン不可（単一インスタンス）
        Box::new(Self {
            title: self.title.clone(),
            size: self.size,
            app_state: Arc::new(Mutex::new(Box::new(()) as Box<dyn Any + Send>)),
            update_fn: None,
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_egui) = other.as_any().downcast_ref::<EguiBox>() {
            BoolBox::new(self.title == other_egui.title && self.size == other_egui.size)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn type_name(&self) -> &'static str {
        "EguiBox"
    }
    
    fn box_id(&self) -> u64 {
        // 簡易的なIDとしてポインタアドレスを使用
        self as *const _ as u64
    }
}

// EguiBoxのメソッド実装（実際にはインタープリターから呼ばれない）
impl EguiBox {
    pub fn run_gui(&self) -> Result<(), RuntimeError> {
        if let Some(update_fn) = &self.update_fn {
            let app_state = Arc::clone(&self.app_state);
            let update_fn = Arc::clone(update_fn);
            
            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size(self.size)
                    .with_title(&self.title),
                ..Default::default()
            };
            
            let app = NyashApp {
                app_state,
                update_fn,
            };
            
            // 注意: これはブロッキング呼び出し
            let _ = eframe::run_native(
                &self.title,
                options,
                Box::new(|_cc| Ok(Box::new(app))),
            );
            
            Ok(())
        } else {
            Err(RuntimeError::InvalidOperation {
                message: "No update function set for EguiBox".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_egui_box_creation() {
        let gui = EguiBox::new();
        assert_eq!(gui.title, "Nyash GUI Application");
        assert_eq!(gui.size, Vec2::new(800.0, 600.0));
    }
    
    #[test]
    fn test_egui_box_to_string() {
        let gui = EguiBox::new();
        let s = gui.to_string_box();
        assert_eq!(s.value, "EguiBox('Nyash GUI Application', 800x600)");
    }
}