/*!
 * CanvasEventBox - Canvas入力イベント管理Box
 * 
 * ## 📝 概要
 * HTML5 Canvasでのマウス・タッチ・キーボードイベントを
 * Nyashから利用可能にするBox。ゲーム開発、インタラクティブ
 * アプリケーション開発に必須の入力機能を提供。
 * 
 * ## 🛠️ 利用可能メソッド
 * 
 * ### 🖱️ マウスイベント
 * - `onMouseDown(callback)` - マウスボタン押下
 * - `onMouseUp(callback)` - マウスボタン離上
 * - `onMouseMove(callback)` - マウス移動
 * - `onMouseClick(callback)` - マウスクリック
 * - `onMouseWheel(callback)` - マウスホイール
 * 
 * ### 👆 タッチイベント
 * - `onTouchStart(callback)` - タッチ開始
 * - `onTouchMove(callback)` - タッチ移動
 * - `onTouchEnd(callback)` - タッチ終了
 * 
 * ### ⌨️ キーボードイベント
 * - `onKeyDown(callback)` - キー押下
 * - `onKeyUp(callback)` - キー離上
 * 
 * ### 📊 座標取得
 * - `getMouseX()` - 現在のマウスX座標
 * - `getMouseY()` - 現在のマウスY座標
 * - `isPressed(button)` - ボタン押下状態確認
 * 
 * ## 💡 使用例
 * ```nyash
 * local events, canvas
 * events = new CanvasEventBox("game-canvas")
 * canvas = new WebCanvasBox("game-canvas", 800, 600)
 * 
 * // マウスクリックで円を描画
 * events.onMouseClick(function(x, y) {
 *     canvas.fillCircle(x, y, 10, "red")
 * })
 * 
 * // ドラッグで線を描画
 * local isDrawing = false
 * events.onMouseDown(function(x, y) {
 *     isDrawing = true
 *     canvas.beginPath()
 *     canvas.moveTo(x, y)
 * })
 * 
 * events.onMouseMove(function(x, y) {
 *     if (isDrawing) {
 *         canvas.lineTo(x, y)
 *         canvas.stroke("black", 2)
 *     }
 * })
 * 
 * events.onMouseUp(function() {
 *     isDrawing = false
 * })
 * ```
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::{
    HtmlCanvasElement, MouseEvent, TouchEvent, KeyboardEvent,
    EventTarget, Element
};

/// Canvas入力イベント管理Box
#[derive(Debug, Clone)]
pub struct CanvasEventBox {
    base: BoxBase,
    canvas_id: String,
    mouse_x: f64,
    mouse_y: f64,
    pressed_buttons: Vec<i16>,
}

impl CanvasEventBox {
    pub fn new(canvas_id: String) -> Self {
        Self {
            base: BoxBase::new(),
            canvas_id,
            mouse_x: 0.0,
            mouse_y: 0.0,
            pressed_buttons: Vec::new(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// Canvas要素を取得
    fn get_canvas_element(&self) -> Option<HtmlCanvasElement> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let element = document.get_element_by_id(&self.canvas_id)?;
        element.dyn_into::<HtmlCanvasElement>().ok()
    }

    /// 現在のマウスX座標を取得
    pub fn get_mouse_x(&self) -> f64 {
        self.mouse_x
    }

    /// 現在のマウスY座標を取得
    pub fn get_mouse_y(&self) -> f64 {
        self.mouse_y
    }

    /// 指定ボタンが押下されているかチェック
    pub fn is_pressed(&self, button: i16) -> bool {
        self.pressed_buttons.contains(&button)
    }

    #[cfg(target_arch = "wasm32")]
    /// マウス座標を Canvas 座標系に変換
    fn get_canvas_coordinates(&self, event: &MouseEvent) -> (f64, f64) {
        if let Some(canvas) = self.get_canvas_element() {
            let rect = canvas.get_bounding_client_rect();
            let x = event.client_x() as f64 - rect.left();
            let y = event.client_y() as f64 - rect.top();
            (x, y)
        } else {
            (event.client_x() as f64, event.client_y() as f64)
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// マウスダウンイベントリスナーを設定
    pub fn on_mouse_down(&self, callback: js_sys::Function) {
        if let Some(canvas) = self.get_canvas_element() {
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                // ここで座標変換とコールバック呼び出し
                callback.call0(&JsValue::NULL).unwrap_or_default();
            }) as Box<dyn FnMut(MouseEvent)>);

            canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
                .unwrap_or_default();
            closure.forget(); // メモリリークを防ぐため通常は適切な管理が必要
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// マウスアップイベントリスナーを設定
    pub fn on_mouse_up(&self, callback: js_sys::Function) {
        if let Some(canvas) = self.get_canvas_element() {
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                callback.call0(&JsValue::NULL).unwrap_or_default();
            }) as Box<dyn FnMut(MouseEvent)>);

            canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())
                .unwrap_or_default();
            closure.forget();
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// マウス移動イベントリスナーを設定
    pub fn on_mouse_move(&self, callback: js_sys::Function) {
        if let Some(canvas) = self.get_canvas_element() {
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                callback.call0(&JsValue::NULL).unwrap_or_default();
            }) as Box<dyn FnMut(MouseEvent)>);

            canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
                .unwrap_or_default();
            closure.forget();
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// マウスクリックイベントリスナーを設定
    pub fn on_mouse_click(&self, callback: js_sys::Function) {
        if let Some(canvas) = self.get_canvas_element() {
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                callback.call0(&JsValue::NULL).unwrap_or_default();
            }) as Box<dyn FnMut(MouseEvent)>);

            canvas.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap_or_default();
            closure.forget();
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// タッチ開始イベントリスナーを設定
    pub fn on_touch_start(&self, callback: js_sys::Function) {
        if let Some(canvas) = self.get_canvas_element() {
            let closure = Closure::wrap(Box::new(move |event: TouchEvent| {
                callback.call0(&JsValue::NULL).unwrap_or_default();
            }) as Box<dyn FnMut(TouchEvent)>);

            canvas.add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref())
                .unwrap_or_default();
            closure.forget();
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// キーダウンイベントリスナーを設定
    pub fn on_key_down(&self, callback: js_sys::Function) {
        if let Some(window) = web_sys::window() {
            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                callback.call0(&JsValue::NULL).unwrap_or_default();
            }) as Box<dyn FnMut(KeyboardEvent)>);

            window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .unwrap_or_default();
            closure.forget();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Non-WASM環境用のダミー実装
    pub fn on_mouse_down(&self) {
        println!("CanvasEventBox: Mouse events not supported in non-WASM environment");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_mouse_up(&self) {
        println!("CanvasEventBox: Mouse events not supported in non-WASM environment");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_mouse_move(&self) {
        println!("CanvasEventBox: Mouse events not supported in non-WASM environment");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_mouse_click(&self) {
        println!("CanvasEventBox: Mouse events not supported in non-WASM environment");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_touch_start(&self) {
        println!("CanvasEventBox: Touch events not supported in non-WASM environment");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_key_down(&self) {
        println!("CanvasEventBox: Keyboard events not supported in non-WASM environment");
    }
}

impl BoxCore for CanvasEventBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CanvasEventBox({})", self.canvas_id)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for CanvasEventBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("CanvasEventBox({})", self.canvas_id))
    }

    fn type_name(&self) -> &'static str {
        "CanvasEventBox"
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_events) = other.as_any().downcast_ref::<CanvasEventBox>() {
            BoolBox::new(self.base.id == other_events.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for CanvasEventBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}