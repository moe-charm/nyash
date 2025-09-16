/*! 🚫 NullBox - NULL値表現Box
 *
 * ## 📝 概要
 * null/void値を表現する特別なBox。
 * JavaScript null、Python None、C# nullと同等の機能を提供。
 * NULL安全プログラミングをサポート。
 *
 * ## 🛠️ 利用可能メソッド
 * - `isNull()` - null判定 (常にtrue)
 * - `isNotNull()` - 非null判定 (常にfalse)
 * - `toString()` - 文字列変換 ("null")
 * - `equals(other)` - 等価比較 (他のnullとのみtrue)
 *
 * ## 🛡️ 静的メソッド (null安全機能)
 * - `NullBox.checkNull(value)` - 値のnull判定
 * - `NullBox.checkNotNull(value)` - 値の非null判定
 * - `NullBox.getOrDefault(value, default)` - null時デフォルト値取得
 *
 * ## 💡 使用例
 * ```nyash
 * local user, name, default_name
 *
 * // null値の作成と判定
 * user = null
 * if (user == null) {
 *     print("User is null")
 * }
 *
 * // null安全な値取得
 * name = getUsername()  // null の可能性
 * default_name = NullBox.getOrDefault(name, "Anonymous")
 * print("Hello, " + default_name)
 * ```
 *
 * ## 🎮 実用例 - null安全プログラミング
 * ```nyash
 * static box UserManager {
 *     init { current_user }
 *     
 *     main() {
 *         me.current_user = null
 *         
 *         // null安全なログイン処理
 *         me.loginUser("alice")
 *         me.displayUserInfo()
 *     }
 *     
 *     loginUser(username) {
 *         if (username == null or username == "") {
 *             print("Error: Invalid username")
 *             return
 *         }
 *         me.current_user = new User(username)
 *     }
 *     
 *     displayUserInfo() {
 *         if (me.current_user == null) {
 *             print("No user logged in")
 *         } else {
 *             print("Current user: " + me.current_user.name)
 *         }
 *     }
 * }
 * ```
 *
 * ## 🔍 デバッグ活用
 * ```nyash
 * local data, result
 * data = fetchDataFromAPI()  // null になる可能性
 *
 * // null チェック付きデバッグ
 * if (NullBox.checkNull(data)) {
 *     print("Warning: API returned null data")
 *     result = NullBox.getOrDefault(data, "default_data")
 * } else {
 *     result = data.process()
 * }
 * ```
 *
 * ## ⚠️ 重要な特徴
 * - `null == null` は常にtrue
 * - `null.toString()` は "null"
 * - 全てのNullBoxインスタンスは論理的に等価
 * - メソッド呼び出し時のnullチェックでNullPointerException防止
 */

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;
use std::fmt::{Debug, Display};

/// null値を表現するBox
#[derive(Debug, Clone)]
pub struct NullBox {
    base: BoxBase,
}

impl NullBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
        }
    }

    /// null値かどうかを判定
    pub fn is_null(&self) -> bool {
        true // NullBoxは常にnull
    }

    /// 値がnullでないかを判定
    pub fn is_not_null(&self) -> bool {
        false // NullBoxは常にnull
    }

    /// 他の値がnullかどうかを判定
    pub fn check_null(value: &dyn NyashBox) -> bool {
        value.as_any().downcast_ref::<NullBox>().is_some()
    }

    /// 他の値がnullでないかを判定
    pub fn check_not_null(value: &dyn NyashBox) -> bool {
        !Self::check_null(value)
    }

    /// null安全な値の取得
    pub fn get_or_default(value: &dyn NyashBox, default: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if Self::check_null(value) {
            default
        } else {
            value.clone_box()
        }
    }
}

impl BoxCore for NullBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "null")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for NullBox {
    fn type_name(&self) -> &'static str {
        "NullBox"
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new("null")
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        // すべてのNullBoxは等しい
        BoolBox::new(other.as_any().downcast_ref::<NullBox>().is_some())
    }
}

impl Display for NullBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// グローバルnullインスタンス用の関数
pub fn null() -> Box<dyn NyashBox> {
    Box::new(NullBox::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_trait::IntegerBox;

    #[test]
    fn test_null_creation() {
        let null_box = NullBox::new();
        assert!(null_box.is_null());
        assert!(!null_box.is_not_null());
        assert_eq!(null_box.to_string_box().value, "null");
    }

    #[test]
    fn test_null_check() {
        let null_box = null();
        let int_box = Box::new(IntegerBox::new(42));

        assert!(NullBox::check_null(null_box.as_ref()));
        assert!(!NullBox::check_null(int_box.as_ref()));

        assert!(!NullBox::check_not_null(null_box.as_ref()));
        assert!(NullBox::check_not_null(int_box.as_ref()));
    }

    #[test]
    fn test_null_equality() {
        let null1 = NullBox::new();
        let null2 = NullBox::new();
        let int_box = IntegerBox::new(42);

        assert!(null1.equals(&null2).value);
        assert!(!null1.equals(&int_box).value);
    }

    #[test]
    fn test_get_or_default() {
        let null_box = null();
        let default_value = Box::new(IntegerBox::new(100));
        let actual_value = Box::new(IntegerBox::new(42));

        // nullの場合はデフォルト値を返す
        let result1 = NullBox::get_or_default(null_box.as_ref(), default_value.clone());
        assert_eq!(result1.to_string_box().value, "100");

        // null以外の場合は元の値を返す
        let result2 = NullBox::get_or_default(actual_value.as_ref(), default_value);
        assert_eq!(result2.to_string_box().value, "42");
    }
}
