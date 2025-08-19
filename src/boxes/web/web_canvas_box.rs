/*!
 * WebCanvasBox - ブラウザCanvas完全制御Box
 * 
 * WebAssembly環境でHTML5 Canvasの完全制御
 * ピクセルの世界を制圧する革命的Box！
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::{
    HtmlCanvasElement, 
    CanvasRenderingContext2d,
};

// 🎨 Browser Canvas complete control Box
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct WebCanvasBox {
    base: BoxBase,
    canvas_id: String,
    width: u32,
    height: u32,
}

#[cfg(target_arch = "wasm32")]
impl WebCanvasBox {
    pub fn new(canvas_id: String, width: u32, height: u32) -> Self {
        let instance = Self { 
            base: BoxBase::new(),
            canvas_id: canvas_id.clone(),
            width,
            height,
        };
        
        // キャンバス要素を初期化
        if let Some(canvas) = instance.get_canvas_element() {
            canvas.set_width(width);
            canvas.set_height(height);
        }
        
        instance
    }
    
    /// Canvas要素を取得
    fn get_canvas_element(&self) -> Option<HtmlCanvasElement> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let element = document.get_element_by_id(&self.canvas_id)?;
        element.dyn_into::<HtmlCanvasElement>().ok()
    }
    
    /// 2Dレンダリングコンテキストを取得
    fn get_2d_context(&self) -> Option<CanvasRenderingContext2d> {
        let canvas = self.get_canvas_element()?;
        canvas
            .get_context("2d")
            .ok()?
            .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())
    }
    
    /// キャンバスをクリア
    pub fn clear(&self) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.clear_rect(0.0, 0.0, self.width as f64, self.height as f64);
        }
    }
    
    /// 塗りつぶし色を設定
    pub fn set_fill_style(&self, color: &str) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(color));
        }
    }
    
    /// 線の色を設定
    pub fn set_stroke_style(&self, color: &str) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(color));
        }
    }
    
    /// 線の太さを設定
    pub fn set_line_width(&self, width: f64) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_line_width(width);
        }
    }
    
    /// 塗りつぶし矩形を描画
    pub fn fill_rect(&self, x: f64, y: f64, width: f64, height: f64, color: &str) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(color));
            ctx.fill_rect(x, y, width, height);
        }
    }
    
    /// 枠線矩形を描画
    pub fn stroke_rect(&self, x: f64, y: f64, width: f64, height: f64, color: &str, line_width: f64) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(color));
            ctx.set_line_width(line_width);
            ctx.stroke_rect(x, y, width, height);
        }
    }
    
    /// 塗りつぶし円を描画
    pub fn fill_circle(&self, x: f64, y: f64, radius: f64, color: &str) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(color));
            ctx.begin_path();
            ctx.arc(x, y, radius, 0.0, 2.0 * std::f64::consts::PI).unwrap_or_default();
            ctx.fill();
        }
    }
    
    /// 枠線円を描画
    pub fn stroke_circle(&self, x: f64, y: f64, radius: f64, color: &str, line_width: f64) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(color));
            ctx.set_line_width(line_width);
            ctx.begin_path();
            ctx.arc(x, y, radius, 0.0, 2.0 * std::f64::consts::PI).unwrap_or_default();
            ctx.stroke();
        }
    }
    
    /// 直線を描画
    pub fn draw_line(&self, x1: f64, y1: f64, x2: f64, y2: f64, color: &str, line_width: f64) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(color));
            ctx.set_line_width(line_width);
            ctx.begin_path();
            ctx.move_to(x1, y1);
            ctx.line_to(x2, y2);
            ctx.stroke();
        }
    }
    
    /// テキストを描画（塗りつぶし）
    pub fn fill_text(&self, text: &str, x: f64, y: f64, font: &str, color: &str) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_font(font);
            ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(color));
            ctx.fill_text(text, x, y).unwrap_or_default();
        }
    }
    
    /// テキストを描画（枠線）
    pub fn stroke_text(&self, text: &str, x: f64, y: f64, font: &str, color: &str, line_width: f64) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_font(font);
            ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(color));
            ctx.set_line_width(line_width);
            ctx.stroke_text(text, x, y).unwrap_or_default();
        }
    }
    
    /// パス描画開始
    pub fn begin_path(&self) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.begin_path();
        }
    }
    
    /// パスを指定位置に移動
    pub fn move_to(&self, x: f64, y: f64) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.move_to(x, y);
        }
    }
    
    /// パスに直線を追加
    pub fn line_to(&self, x: f64, y: f64) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.line_to(x, y);
        }
    }
    
    /// パスを閉じる
    pub fn close_path(&self) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.close_path();
        }
    }
    
    /// パスを塗りつぶし
    pub fn fill(&self, color: &str) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(color));
            ctx.fill();
        }
    }
    
    /// パスを枠線描画
    pub fn stroke(&self, color: &str, line_width: f64) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.set_stroke_style(&wasm_bindgen::JsValue::from_str(color));
            ctx.set_line_width(line_width);
            ctx.stroke();
        }
    }
    
    /// 現在の描画状態を保存
    pub fn save(&self) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.save();
        }
    }
    
    /// 描画状態を復元
    pub fn restore(&self) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.restore();
        }
    }
    
    /// 座標系を回転
    pub fn rotate(&self, angle: f64) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.rotate(angle).unwrap_or_default();
        }
    }
    
    /// 座標系をスケール
    pub fn scale(&self, x: f64, y: f64) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.scale(x, y).unwrap_or_default();
        }
    }
    
    /// 座標系を平行移動
    pub fn translate(&self, x: f64, y: f64) {
        if let Some(ctx) = self.get_2d_context() {
            ctx.translate(x, y).unwrap_or_default();
        }
    }
    
    /// キャンバスのサイズを取得
    pub fn get_width(&self) -> u32 {
        self.width
    }
    
    pub fn get_height(&self) -> u32 {
        self.height
    }
    
    /// キャンバスのサイズを変更
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        
        if let Some(canvas) = self.get_canvas_element() {
            canvas.set_width(width);
            canvas.set_height(height);
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl BoxCore for WebCanvasBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "WebCanvasBox({}, {}x{})", self.canvas_id, self.width, self.height)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(target_arch = "wasm32")]
impl NyashBox for WebCanvasBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!(
            "WebCanvasBox({}, {}x{})", 
            self.canvas_id, 
            self.width, 
            self.height
        ))
    }


    fn type_name(&self) -> &'static str {
        "WebCanvasBox"
    }
    

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_canvas) = other.as_any().downcast_ref::<WebCanvasBox>() {
            BoolBox::new(self.base.id == other_canvas.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl std::fmt::Display for WebCanvasBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}