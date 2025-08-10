/*! 🎯 Nyash Box実装モジュール
 * Everything is Box哲学に基づく各Box型の実装
 * 
 * ## 📦 利用可能なBox一覧
 * 
 * ### 🔤 基本データ型Box
 * - **StringBox**: 文字列操作 - `"Hello".length()`, `str.split(",")`
 * - **IntegerBox**: 整数計算 - `42.add(8)`, `num.toString()`
 * - **BoolBox**: 真偽値 - `true.not()`, `flag.toString()`
 * 
 * ### 🧮 計算・ユーティリティBox  
 * - **MathBox**: 数学関数 - `Math.sin(x)`, `Math.random()`
 * - **TimeBox**: 時間操作 - `Time.now()`, `time.format()`
 * - **RandomBox**: 乱数生成 - `Random.int(10)`, `Random.choice(array)`
 * 
 * ### 🖥️ システム・IO Box
 * - **ConsoleBox**: コンソール出力 - `console.log()`, `console.error()`  
 * - **DebugBox**: デバッグ支援 - `debug.trace()`, `debug.memory()`
 * - **SoundBox**: 音声再生 - `sound.beep()`, `sound.play(file)`
 * 
 * ### 🗄️ コレクション・データBox
 * - **MapBox**: キー値ストレージ - `map.set(key, val)`, `map.get(key)`
 * - **NullBox**: NULL値表現 - `null.toString()` → "void"
 * 
 * ### 🖼️ GUI・グラフィックBox
 * - **EguiBox**: デスクトップGUI - `gui.setTitle()`, `gui.run()`
 * 
 * ### 🌐 Web専用Box (WASM環境)
 * - **WebDisplayBox**: HTML表示 - `display.show(html)`
 * - **WebConsoleBox**: ブラウザコンソール - `webConsole.log()`
 * - **WebCanvasBox**: Canvas描画 - `canvas.drawRect()`
 * 
 * ### 🔗 通信・ネットワークBox
 * - **SimpleIntentBox**: P2P通信 - `intent.send()`, `intent.on()`
 * 
 * ## 💡 使用例
 * ```nyash
 * // 基本的な使い方
 * local str, num, result
 * str = "Nyash"
 * num = 42
 * result = str.concat(" v") + num.toString()
 * 
 * // GUIアプリ作成
 * local app
 * app = new EguiBox()
 * app.setTitle("My App")
 * app.run()
 * ```
 */

// Nyashは意図的にJavaScript/TypeScriptスタイルのcamelCase命名規約を採用
#![allow(non_snake_case)]

// 各Boxモジュールを宣言
pub mod string_box;
pub mod integer_box;
pub mod bool_box;
pub mod math_box;
pub mod time_box;
pub mod debug_box;
pub mod random_box;
pub mod sound_box;
pub mod map_box;
pub mod console_box;

// Web専用Box群（ブラウザ環境でのみ利用可能）
#[cfg(target_arch = "wasm32")]
pub mod web;

// GUI Box（条件付きコンパイル）
#[cfg(not(target_arch = "wasm32"))]
pub mod egui_box;

// 共通で使う型とトレイトを再エクスポート
pub use string_box::StringBox;
pub use integer_box::IntegerBox;
pub use bool_box::BoolBox;
pub use math_box::MathBox;
pub use time_box::TimeBox;
pub use debug_box::DebugBox;
pub use random_box::RandomBox;
pub use sound_box::SoundBox;
pub use map_box::MapBox;
pub use console_box::ConsoleBox;

// EguiBoxの再エクスポート（非WASM環境のみ）
#[cfg(not(target_arch = "wasm32"))]
pub use egui_box::EguiBox;

// Web Box群の再エクスポート（WASM環境のみ）
#[cfg(target_arch = "wasm32")]
pub use web::{WebDisplayBox, WebConsoleBox, WebCanvasBox};

pub mod null_box;

// P2P通信Box群
// pub mod intent_box;
// pub mod intent_box_wrapper;
// pub mod p2p_box;

// 今後追加予定のBox型（コメントアウト）
// pub mod array_box;
// pub use array_box::ArrayBox;

// null関数も再エクスポート
pub use null_box::{NullBox, null};

// P2P通信Boxの再エクスポート
// pub use intent_box::IntentBox;
// pub use p2p_box::P2PBox;