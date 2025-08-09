// Nyash Box Implementations Module
// Everything is Box哲学に基づく各Box型の実装

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