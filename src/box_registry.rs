/*!
 * Built-in Box Registry
 *
 * This module contains the registry of built-in box types and related utilities.
 * This is separated from box_core.rs because it changes more frequently
 * (when adding new built-in boxes) and doesn't need to trigger recompilation
 * of files that only import the core traits.
 */

/// 🔥 Phase 8.8: pack透明化システム - ビルトインBox判定リスト
/// ユーザーは`pack`を一切意識せず、`from BuiltinBox()`で自動的に内部のpack機能が呼ばれる
pub const BUILTIN_BOXES: &[&str] = &[
    "StringBox",
    "IntegerBox",
    "BoolBox",
    "NullBox",
    "ArrayBox",
    "MapBox",
    "MissingBox",
    "FileBox",
    "ResultBox",
    "FutureBox",
    "ChannelBox",
    "MathBox",
    "FloatBox",
    "TimeBox",
    "DateTimeBox",
    "TimerBox",
    "RandomBox",
    "SoundBox",
    "DebugBox",
    "MethodBox",
    "ConsoleBox",
    "BufferBox",
    "RegexBox",
    "JSONBox",
    "StreamBox",
    "HTTPClientBox",
    "IntentBox",
    "P2PBox",
    "SocketBox",
    "HTTPServerBox",
    "HTTPRequestBox",
    "HTTPResponseBox",
];

/// 🔥 ビルトインBox判定関数 - pack透明化システムの核心
/// ユーザー側: `from StringBox()` → 内部的に `StringBox.pack()` 自動呼び出し
pub fn is_builtin_box(box_name: &str) -> bool {
    BUILTIN_BOXES.contains(&box_name)
}
