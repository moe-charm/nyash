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
// 🎯 Phase 3リファクタリング: 基本Box実装を分離したモジュール
pub mod basic;

// 🎯 Phase 4リファクタリング: 算術Box実装を分離したモジュール
pub mod arithmetic;

pub mod bool_box;
pub mod debug_box;
pub mod integer_box;
pub mod math_box;
pub mod random_box;
pub mod string_box;
pub mod time_box;
// These boxes use web APIs that require special handling in WASM
pub mod aot_compiler_box;
pub mod aot_config_box;
#[cfg(not(target_arch = "wasm32"))]
pub mod audio_box;
#[cfg(not(target_arch = "wasm32"))]
pub mod canvas_event_box;
#[cfg(not(target_arch = "wasm32"))]
pub mod canvas_loop_box;
pub mod console_box;
pub mod debug_config_box;
pub mod function_box;
pub mod gc_config_box;
// ARCHIVED: JIT Box modules moved to archive/jit-cranelift/ during Phase 15
// pub mod jit_config_box;
// pub mod jit_events_box;
// pub mod jit_hostcall_registry_box;
// pub mod jit_policy_box;
// pub mod jit_stats_box;
// pub mod jit_strict_box;
pub mod map_box;
#[cfg(not(target_arch = "wasm32"))]
pub mod qr_box;
pub mod ref_cell_box;
pub mod sound_box;
pub mod task_group_box;
#[cfg(not(target_arch = "wasm32"))]
pub mod timer_box;
pub mod token_box;

// Web専用Box群（ブラウザ環境でのみ利用可能）
#[cfg(target_arch = "wasm32")]
pub mod web;

// GUI Box（条件付きコンパイル）
#[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
pub mod egui_box;

// 共通で使う型とトレイトを再エクスポート
// pub use string_box::StringBox; // レガシー実装、box_trait::StringBoxを使用すること
// pub use integer_box::IntegerBox; // レガシー実装、box_trait::IntegerBoxを使用すること
// pub use bool_box::BoolBox; // レガシー実装、box_trait::BoolBoxを使用すること
pub use aot_compiler_box::AotCompilerBox;
pub use aot_config_box::AotConfigBox;
#[cfg(not(target_arch = "wasm32"))]
pub use audio_box::AudioBox;
#[cfg(not(target_arch = "wasm32"))]
pub use canvas_event_box::CanvasEventBox;
#[cfg(not(target_arch = "wasm32"))]
pub use canvas_loop_box::CanvasLoopBox;
pub use console_box::ConsoleBox;
pub use debug_box::DebugBox;
// ARCHIVED: JIT Box imports moved to archive/jit-cranelift/ during Phase 15
// pub use jit_config_box::JitConfigBox;
// pub use jit_events_box::JitEventsBox;
// pub use jit_hostcall_registry_box::JitHostcallRegistryBox;
// pub use jit_policy_box::JitPolicyBox;
// pub use jit_stats_box::JitStatsBox;
// pub use jit_strict_box::JitStrictBox;
pub use map_box::MapBox;
pub use math_box::{FloatBox, MathBox};
#[cfg(not(target_arch = "wasm32"))]
pub use qr_box::QRBox;
pub use random_box::RandomBox;
pub use sound_box::SoundBox;
pub use task_group_box::TaskGroupBox;
pub use time_box::{DateTimeBox, TimeBox};
#[cfg(not(target_arch = "wasm32"))]
pub use timer_box::TimerBox;
pub use token_box::TokenBox;

// EguiBoxの再エクスポート（非WASM環境のみ）
#[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
pub use egui_box::EguiBox;

// Web Box群の再エクスポート（WASM環境のみ）
#[cfg(target_arch = "wasm32")]
pub use web::{WebCanvasBox, WebConsoleBox, WebDisplayBox};

pub mod null_box;
pub mod missing_box;

// High-priority Box types
pub mod array;
pub mod buffer;
pub mod file;
pub mod future;
pub mod http;
pub mod http_message_box;
pub mod http_server_box;
pub mod json;
pub mod regex;
pub mod result;
pub mod socket_box;
pub mod stream;

// P2P通信Box群 (NEW! - Completely rewritten)
pub mod intent_box;
#[cfg(feature = "interpreter-legacy")]
pub mod p2p_box;

// null関数も再エクスポート
pub use null_box::{null, NullBox};
pub use missing_box::MissingBox;

// High-priority Box types re-export
pub use array::ArrayBox;
pub use buffer::BufferBox;
pub use file::FileBox;
pub use future::{FutureBox, FutureWeak, NyashFutureBox};
pub use http::HttpClientBox;
pub use http_message_box::{HTTPRequestBox, HTTPResponseBox};
pub use http_server_box::HTTPServerBox;
pub use json::JSONBox;
pub use regex::RegexBox;
pub use result::{NyashResultBox, ResultBox};
pub use socket_box::SocketBox;
pub use stream::{NyashStreamBox, StreamBox};

// P2P通信Boxの再エクスポート
pub use intent_box::IntentBox;
#[cfg(feature = "interpreter-legacy")]
pub use p2p_box::P2PBox;
