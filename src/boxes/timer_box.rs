/*!
 * TimerBox - JavaScript風タイマー機能Box
 *
 * ## 📝 概要
 * setTimeout/setInterval/requestAnimationFrameをNyashから利用可能にするBox。
 * アニメーション、遅延実行、定期実行を統一的に管理。
 *
 * ## 🛠️ 利用可能メソッド
 *
 * ### ⏱️ 基本タイマー
 * - `setTimeout(callback, delay)` - 指定時間後に1回実行
 * - `setInterval(callback, interval)` - 指定間隔で繰り返し実行
 * - `clearTimeout(id)` - タイマーをキャンセル
 * - `clearInterval(id)` - インターバルをキャンセル
 *
 * ### 🎮 アニメーション
 * - `requestAnimationFrame(callback)` - 次フレームで実行
 * - `cancelAnimationFrame(id)` - アニメーションをキャンセル
 *
 * ### 📊 時間測定
 * - `now()` - 現在時刻（ミリ秒）
 * - `performance()` - 高精度時刻測定
 *
 * ## 💡 使用例
 * ```nyash
 * local timer, id
 * timer = new TimerBox()
 *
 * // 1秒後に実行
 * id = timer.setTimeout(function() {
 *     print("Hello after 1 second!")
 * }, 1000)
 *
 * // 500msごとに実行
 * id = timer.setInterval(function() {
 *     print("Tick every 500ms")
 * }, 500)
 *
 * // アニメーションループ
 * timer.requestAnimationFrame(function() {
 *     // 描画処理
 *     canvas.clear()
 *     canvas.drawRect(x, y, 50, 50)
 * })
 * ```
 */

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::window;

/// タイマー管理Box
#[derive(Debug, Clone)]
pub struct TimerBox {
    base: BoxBase,
}

impl TimerBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
        }
    }

    /// 現在時刻をミリ秒で取得
    pub fn now(&self) -> f64 {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = window() {
                if let Ok(perf) = window.performance() {
                    perf.now()
                } else {
                    js_sys::Date::now()
                }
            } else {
                js_sys::Date::now()
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64
        }
    }

    /// 高精度時刻測定
    pub fn performance_now(&self) -> f64 {
        self.now()
    }

    #[cfg(target_arch = "wasm32")]
    /// setTimeout相当の遅延実行
    pub fn set_timeout(&self, callback: &js_sys::Function, delay: i32) -> i32 {
        if let Some(window) = window() {
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(callback, delay)
                .unwrap_or(-1)
        } else {
            -1
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// setInterval相当の定期実行
    pub fn set_interval(&self, callback: &js_sys::Function, interval: i32) -> i32 {
        if let Some(window) = window() {
            window
                .set_interval_with_callback_and_timeout_and_arguments_0(callback, interval)
                .unwrap_or(-1)
        } else {
            -1
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// clearTimeout相当のタイマーキャンセル
    pub fn clear_timeout(&self, id: i32) {
        if let Some(window) = window() {
            window.clear_timeout_with_handle(id);
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// clearInterval相当のインターバルキャンセル
    pub fn clear_interval(&self, id: i32) {
        if let Some(window) = window() {
            window.clear_interval_with_handle(id);
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// requestAnimationFrame相当のアニメーション実行
    pub fn request_animation_frame(&self, callback: &js_sys::Function) -> i32 {
        if let Some(window) = window() {
            window.request_animation_frame(callback).unwrap_or(-1)
        } else {
            -1
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// cancelAnimationFrame相当のアニメーションキャンセル
    pub fn cancel_animation_frame(&self, id: i32) {
        if let Some(window) = window() {
            window.cancel_animation_frame(id).unwrap_or_default();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Non-WASM環境用のダミー実装
    pub fn set_timeout(&self, _delay: i32) -> i32 {
        println!("TimerBox: setTimeout not supported in non-WASM environment");
        -1
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_interval(&self, _interval: i32) -> i32 {
        println!("TimerBox: setInterval not supported in non-WASM environment");
        -1
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear_timeout(&self, _id: i32) {
        println!("TimerBox: clearTimeout not supported in non-WASM environment");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear_interval(&self, _id: i32) {
        println!("TimerBox: clearInterval not supported in non-WASM environment");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn request_animation_frame(&self) -> i32 {
        println!("TimerBox: requestAnimationFrame not supported in non-WASM environment");
        -1
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn cancel_animation_frame(&self, _id: i32) {
        println!("TimerBox: cancelAnimationFrame not supported in non-WASM environment");
    }
}

impl BoxCore for TimerBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TimerBox(id={})", self.base.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for TimerBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("TimerBox(id={})", self.base.id))
    }

    fn type_name(&self) -> &'static str {
        "TimerBox"
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_timer) = other.as_any().downcast_ref::<TimerBox>() {
            BoolBox::new(self.base.id == other_timer.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for TimerBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}
