/*!
 * Parser Declarations Module
 * 
 * 宣言（Declaration）の解析を担当するモジュール群
 * Box定義、関数定義、use文などの宣言を処理
 */

pub mod box_definition;
pub mod static_box;
pub mod dependency_helpers;

// Re-export commonly used items
pub use box_definition::*;
pub use static_box::*;
pub use dependency_helpers::*;