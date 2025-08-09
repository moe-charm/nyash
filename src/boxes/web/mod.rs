/*!
 * Web Boxes Module - ブラウザ専用Box群
 * 
 * WebAssembly環境専用のBox群を管理
 * HTML5 APIs、DOM操作、Canvas描画等をNyashから利用可能にする
 */

#[cfg(target_arch = "wasm32")]
pub mod web_display_box;

#[cfg(target_arch = "wasm32")]
pub mod web_console_box;

#[cfg(target_arch = "wasm32")]
pub mod web_canvas_box;

#[cfg(target_arch = "wasm32")]
pub use web_display_box::WebDisplayBox;

#[cfg(target_arch = "wasm32")]
pub use web_console_box::WebConsoleBox;

#[cfg(target_arch = "wasm32")]
pub use web_canvas_box::WebCanvasBox;