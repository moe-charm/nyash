/*!
 * CanvasLoopBox - アニメーションループ管理Box
 *
 * ## 📝 概要
 * ゲームや動的コンテンツのためのアニメーションループを
 * 管理するBox。requestAnimationFrame、フレームレート制御、
 * ループ状態管理を統一的に提供。
 *
 * ## 🛠️ 利用可能メソッド
 *
 * ### 🎮 ループ制御
 * - `start(callback)` - アニメーションループ開始
 * - `stop()` - アニメーションループ停止
 * - `pause()` - アニメーションループ一時停止
 * - `resume()` - アニメーションループ再開
 *
 * ### 📊 フレーム情報
 * - `getFPS()` - 現在のFPS取得
 * - `getFrameCount()` - 総フレーム数取得
 * - `getDeltaTime()` - 前フレームからの経過時間
 * - `setTargetFPS(fps)` - 目標FPS設定
 *
 * ### ⏱️ 時間管理
 * - `getElapsedTime()` - ループ開始からの経過時間
 * - `reset()` - タイマーリセット
 *
 * ## 💡 使用例
 * ```nyash
 * local loop, canvas, ball_x, ball_y
 * loop = new CanvasLoopBox()
 * canvas = new WebCanvasBox("game-canvas", 800, 600)
 * ball_x = 400
 * ball_y = 300
 *
 * // ゲームループ
 * loop.start(function(deltaTime) {
 *     // 更新処理
 *     ball_x = ball_x + 100 * deltaTime  // 100px/秒で移動
 *     
 *     // 描画処理
 *     canvas.clear()
 *     canvas.fillCircle(ball_x, ball_y, 20, "red")
 *     
 *     // FPS表示
 *     canvas.fillText("FPS: " + loop.getFPS(), 10, 30, "16px Arial", "black")
 * })
 * ```
 */

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use crate::boxes::TimerBox;
use std::any::Any;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// アニメーションループ管理Box
#[derive(Debug, Clone)]
pub struct CanvasLoopBox {
    base: BoxBase,
    is_running: bool,
    is_paused: bool,
    frame_count: u64,
    last_frame_time: f64,
    start_time: f64,
    fps: f64,
    target_fps: Option<f64>,
    delta_time: f64,
    timer: TimerBox,
    #[cfg(target_arch = "wasm32")]
    animation_id: Option<i32>,
}

impl CanvasLoopBox {
    pub fn new() -> Self {
        let timer = TimerBox::new();
        let current_time = timer.now();

        Self {
            base: BoxBase::new(),
            is_running: false,
            is_paused: false,
            frame_count: 0,
            last_frame_time: current_time,
            start_time: current_time,
            fps: 0.0,
            target_fps: None,
            delta_time: 0.0,
            timer,
            #[cfg(target_arch = "wasm32")]
            animation_id: None,
        }
    }

    /// アニメーションループを開始
    #[cfg(target_arch = "wasm32")]
    pub fn start(&mut self, callback: js_sys::Function) {
        if self.is_running {
            return;
        }

        self.is_running = true;
        self.is_paused = false;
        self.start_time = self.timer.now();
        self.last_frame_time = self.start_time;
        self.frame_count = 0;

        // アニメーションフレーム用のクロージャを作成
        let closure = Closure::wrap(Box::new(move |time: f64| {
            // ここでフレーム処理を実行
            callback
                .call1(&JsValue::NULL, &JsValue::from_f64(time))
                .unwrap_or_default();
        }) as Box<dyn FnMut(f64)>);

        let id = self
            .timer
            .request_animation_frame(closure.as_ref().unchecked_ref());
        self.animation_id = Some(id);

        closure.forget(); // クロージャの所有権を手放す
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Non-WASM環境用のダミー実装
    pub fn start(&mut self) {
        println!("CanvasLoopBox: Animation loop not supported in non-WASM environment");
        self.is_running = true;
    }

    /// アニメーションループを停止
    pub fn stop(&mut self) {
        if !self.is_running {
            return;
        }

        self.is_running = false;
        self.is_paused = false;

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(id) = self.animation_id {
                self.timer.cancel_animation_frame(id);
                self.animation_id = None;
            }
        }
    }

    /// アニメーションループを一時停止
    pub fn pause(&mut self) {
        if !self.is_running || self.is_paused {
            return;
        }

        self.is_paused = true;

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(id) = self.animation_id {
                self.timer.cancel_animation_frame(id);
                self.animation_id = None;
            }
        }
    }

    /// アニメーションループを再開
    #[cfg(target_arch = "wasm32")]
    pub fn resume(&mut self, callback: js_sys::Function) {
        if !self.is_running || !self.is_paused {
            return;
        }

        self.is_paused = false;
        self.last_frame_time = self.timer.now(); // 時間をリセット

        let closure = Closure::wrap(Box::new(move |time: f64| {
            callback
                .call1(&JsValue::NULL, &JsValue::from_f64(time))
                .unwrap_or_default();
        }) as Box<dyn FnMut(f64)>);

        let id = self
            .timer
            .request_animation_frame(closure.as_ref().unchecked_ref());
        self.animation_id = Some(id);

        closure.forget();
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn resume(&mut self) {
        println!("CanvasLoopBox: Resume not supported in non-WASM environment");
        self.is_paused = false;
    }

    /// フレーム更新処理（各フレームで呼び出される）
    pub fn update_frame(&mut self) {
        if !self.is_running || self.is_paused {
            return;
        }

        let current_time = self.timer.now();
        self.delta_time = (current_time - self.last_frame_time) / 1000.0; // 秒単位
        self.last_frame_time = current_time;
        self.frame_count += 1;

        // FPS計算（1秒間の移動平均）
        if self.delta_time > 0.0 {
            let instant_fps = 1.0 / self.delta_time;
            // 簡単な移動平均でFPSを滑らかにする
            self.fps = self.fps * 0.9 + instant_fps * 0.1;
        }
    }

    /// 現在のFPSを取得
    pub fn get_fps(&self) -> f64 {
        self.fps
    }

    /// 総フレーム数を取得
    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }

    /// 前フレームからの経過時間（秒）を取得
    pub fn get_delta_time(&self) -> f64 {
        self.delta_time
    }

    /// ループ開始からの経過時間（秒）を取得
    pub fn get_elapsed_time(&self) -> f64 {
        if self.is_running {
            (self.timer.now() - self.start_time) / 1000.0
        } else {
            0.0
        }
    }

    /// 目標FPSを設定
    pub fn set_target_fps(&mut self, fps: f64) {
        if fps > 0.0 {
            self.target_fps = Some(fps);
        } else {
            self.target_fps = None;
        }
    }

    /// 実行状態を確認
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// 一時停止状態を確認
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// タイマーをリセット
    pub fn reset(&mut self) {
        let current_time = self.timer.now();
        self.start_time = current_time;
        self.last_frame_time = current_time;
        self.frame_count = 0;
        self.fps = 0.0;
        self.delta_time = 0.0;
    }
}

impl BoxCore for CanvasLoopBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "CanvasLoopBox(running={}, fps={:.1})",
            self.is_running, self.fps
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for CanvasLoopBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!(
            "CanvasLoopBox(running={}, fps={:.1})",
            self.is_running, self.fps
        ))
    }

    fn type_name(&self) -> &'static str {
        "CanvasLoopBox"
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_loop) = other.as_any().downcast_ref::<CanvasLoopBox>() {
            BoolBox::new(self.base.id == other_loop.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for CanvasLoopBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}
