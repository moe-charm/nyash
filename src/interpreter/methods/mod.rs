/*!
 * Box Methods Module Organization
 * 
 * 旧box_methods.rsを機能別に分割したモジュール群
 * 保守性と可読性の向上を目的とした再構成
 * 
 * Current implementation:
 * - basic_methods: StringBox, IntegerBox, BoolBox, FloatBox 
 * - collection_methods: ArrayBox, MapBox
 * - io_methods: FileBox, ResultBox ✅ IMPLEMENTED
 * Future modules (planned):
 * - system_methods: TimeBox, DateTimeBox, TimerBox, DebugBox
 * - math_methods: MathBox, RandomBox
 * - async_methods: FutureBox, ChannelBox
 * - web_methods: WebDisplayBox, WebConsoleBox, WebCanvasBox
 * - special_methods: MethodBox, SoundBox
 */

pub mod basic_methods;      // StringBox, IntegerBox, BoolBox, FloatBox
pub mod collection_methods; // ArrayBox, MapBox
pub mod io_methods;         // FileBox, ResultBox

// Re-export methods for easy access
pub use basic_methods::*;
pub use collection_methods::*;
pub use io_methods::*;