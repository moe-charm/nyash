#![cfg(feature = "gui")]

/*! 🖼️ EguiBox - デスクトップGUIアプリBox
 * Everything is Box哲学によるGUIフレームワーク統合
 * 「なんでもBoxにできる」化け物言語の第一歩！
 *
 * ## 📝 概要  
 * Rustの人気GUI框架eframeを使ったネイティブデスクトップアプリ作成。
 * Nyashコードから直接GUI操作が可能！
 *
 * ## 🛠️ 利用可能メソッド
 * - `setTitle(title)` - ウィンドウタイトル設定
 * - `setSize(width, height)` - ウィンドウサイズ設定  
 * - `run()` - GUIアプリ実行開始
 * - `addText(text)` - テキスト表示追加
 * - `addButton(label)` - ボタン追加
 * - `close()` - ウィンドウ閉じる
 *
 * ## 💡 使用例
 * ```nyash  
 * // 基本的なGUIアプリ
 * local app
 * app = new EguiBox()
 * app.setTitle("Nyash GUI Demo")
 * app.setSize(800, 600)
 * app.addText("Welcome to Nyash!")
 * app.addButton("Click Me")
 * app.run()  // GUIアプリ開始
 * ```
 *
 * ## ⚠️ 注意
 * - デスクトップ環境でのみ利用可能（WASM環境では無効）
 * - `run()`はブロッキング動作（アプリ終了まで制御を返さない）
 */

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use crate::interpreter::RuntimeError;
use eframe::{self, egui, epaint::Vec2};
use std::any::Any;
use std::sync::{Arc, RwLock};

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
    base: BoxBase,
    title: String,
    size: Vec2,
    app_state: RwLock<Box<dyn Any + Send + Sync>>,
    update_fn: Option<Arc<dyn Fn(&mut Box<dyn Any + Send + Sync>, &egui::Context) + Send + Sync>>,
}

impl std::fmt::Debug for EguiBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiBox")
            .field("title", &self.title)
            .field("size", &self.size)
            .finish()
    }
}

impl Clone for EguiBox {
    fn clone(&self) -> Self {
        // Note: This is a simplified clone that doesn't preserve app_state
        // Complex Any+Send state and function pointers are difficult to clone properly
        Self {
            base: BoxBase::new(), // New unique ID for clone
            title: self.title.clone(),
            size: self.size,
            app_state: RwLock::new(Box::new(()) as Box<dyn Any + Send + Sync>),
            update_fn: self.update_fn.clone(), // Arc is cloneable
        }
    }
}

impl EguiBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            title: "Nyash GUI Application".to_string(),
            size: Vec2::new(800.0, 600.0),
            app_state: RwLock::new(Box::new(()) as Box<dyn Any + Send + Sync>),
            update_fn: None,
        }
    }

    /// アプリケーション状態を設定
    pub fn set_app_state<T: Any + Send + Sync + 'static>(&mut self, state: T) {
        *self.app_state.write().unwrap() = Box::new(state);
    }

    /// 更新関数を設定
    pub fn set_update_fn<F>(&mut self, f: F)
    where
        F: Fn(&mut Box<dyn Any + Send + Sync>, &egui::Context) + Send + Sync + 'static,
    {
        self.update_fn = Some(Arc::new(f));
    }
}

// NyashApp - eframe::Appを実装する内部構造体
struct NyashApp {
    app_state: Arc<RwLock<Box<dyn Any + Send + Sync>>>,
    update_fn: Arc<dyn Fn(&mut Box<dyn Any + Send + Sync>, &egui::Context) + Send + Sync>,
}

impl eframe::App for NyashApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(mut state) = self.app_state.write() {
            (self.update_fn)(&mut *state, ctx);
        }
    }
}

impl BoxCore for EguiBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "EguiBox('{}', {}x{})",
            self.title, self.size.x, self.size.y
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for EguiBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

impl NyashBox for EguiBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!(
            "EguiBox('{}', {}x{})",
            self.title, self.size.x, self.size.y
        ))
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
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
}

// EguiBoxのメソッド実装（実際にはインタープリターから呼ばれない）
impl EguiBox {
    pub fn run_gui(&self) -> Result<(), RuntimeError> {
        if let Some(update_fn) = &self.update_fn {
            // Create a new Arc<RwLock> with the current state for thread safety
            let state_snapshot = self.app_state.read().unwrap();
            // Note: This is a simplified approach - in a full implementation,
            // we would need a more sophisticated state sharing mechanism
            let app_state = Arc::new(RwLock::new(Box::new(()) as Box<dyn Any + Send + Sync>));
            drop(state_snapshot);

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
            let _ = eframe::run_native(&self.title, options, Box::new(|_cc| Ok(Box::new(app))));

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
