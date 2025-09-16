/*! 🎲 RandomBox - 乱数生成Box
 *
 * ## 📝 概要
 * 高品質な乱数生成機能を提供するBox。
 * ゲーム開発、統計処理、テストデータ生成に最適。
 *
 * ## 🛠️ 利用可能メソッド
 *
 * ### 🔢 基本乱数
 * - `random()` - 0.0～1.0の浮動小数点乱数
 * - `randInt(min, max)` - 指定範囲の整数乱数
 * - `randBool()` - true/falseのランダム選択
 * - `seed(value)` - 乱数種を設定（再現可能な乱数）
 *
 * ### 🎯 選択・配列操作
 * - `choice(array)` - 配列からランダム選択
 * - `shuffle(array)` - 配列をシャッフル
 *
 * ### 🎨 生成
 * - `randString(length)` - ランダム文字列生成
 * - `probability(prob)` - 指定確率でtrue
 *
 * ## 💡 使用例
 * ```nyash
 * local random, result, dice, array
 * random = new RandomBox()
 *
 * // 基本的な乱数
 * result = random.random()      // 0.0～1.0
 * dice = random.randInt(1, 6)   // サイコロ(1-6)
 * result = random.randBool()    // true or false
 *
 * // 配列関連
 * array = ["apple", "banana", "cherry"]
 * result = random.choice(array)     // ランダム選択
 * array = random.shuffle(array)     // シャッフル
 *
 * // ゲーム用途
 * local password, critical_hit
 * password = random.randString(8)       // 8文字のランダム文字列
 * critical_hit = random.probability(0.1)  // 10%でクリティカル
 * ```
 *
 * ## 🎮 実用例
 * ```nyash
 * // RPGダメージ計算
 * local damage, is_critical
 * damage = random.randInt(10, 20)        // 基本ダメージ10-20
 * is_critical = random.probability(0.15) // 15%でクリティカル
 * if (is_critical) {
 *     damage = damage * 2
 * }
 *
 * // テストデータ生成
 * local users, user_id, user_name
 * users = []
 * loop(i < 10) {
 *     user_id = random.randInt(1000, 9999)
 *     user_name = "user_" + random.randString(5)
 *     users.push(user_name + ":" + user_id)
 * }
 * ```
 *
 * ## ⚠️ 注意
 * - 暗号学的に安全な乱数ではない（セキュリティ用途非推奨）
 * - seed()で同じ値を設定すると同じ乱数列を生成（テスト用）
 * - 大きな配列のshuffleは処理時間が長い場合あり
 */

use crate::box_trait::{BoolBox, BoxBase, BoxCore, IntegerBox, NyashBox, StringBox};
use crate::boxes::{ArrayBox, FloatBox};
use std::any::Any;
use std::fmt::{Debug, Display};
use std::sync::RwLock;

/// 乱数生成を提供するBox
#[derive(Debug)]
pub struct RandomBox {
    // 簡易線形合同法による疑似乱数生成器
    seed: RwLock<u64>,
    base: BoxBase,
}

impl Clone for RandomBox {
    fn clone(&self) -> Self {
        let seed_val = *self.seed.read().unwrap();

        Self {
            seed: RwLock::new(seed_val),
            base: BoxBase::new(), // New unique ID for clone
        }
    }
}

impl RandomBox {
    pub fn new() -> Self {
        // 現在時刻を種として使用
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        Self {
            seed: RwLock::new(seed),
            base: BoxBase::new(),
        }
    }

    /// 種を設定
    pub fn seed(&self, new_seed: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = new_seed.as_any().downcast_ref::<IntegerBox>() {
            *self.seed.write().unwrap() = int_box.value as u64;
            Box::new(StringBox::new("Seed set"))
        } else {
            Box::new(StringBox::new("Error: seed() requires integer input"))
        }
    }

    /// 次の乱数を生成（線形合同法）
    fn next_random(&self) -> u64 {
        let mut seed = self.seed.write().unwrap();
        // 線形合同法の定数（Numerical Recipes より）
        *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        *seed
    }

    /// 0.0-1.0の浮動小数点乱数
    pub fn random(&self) -> Box<dyn NyashBox> {
        let r = self.next_random();
        let normalized = (r as f64) / (u64::MAX as f64);
        Box::new(FloatBox::new(normalized))
    }

    /// 指定範囲の整数乱数
    pub fn randInt(&self, min: Box<dyn NyashBox>, max: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let (Some(min_int), Some(max_int)) = (
            min.as_any().downcast_ref::<IntegerBox>(),
            max.as_any().downcast_ref::<IntegerBox>(),
        ) {
            if min_int.value > max_int.value {
                return Box::new(StringBox::new("Error: min must be <= max"));
            }

            let range = (max_int.value - min_int.value + 1) as u64;
            let r = self.next_random() % range;
            Box::new(IntegerBox::new(min_int.value + r as i64))
        } else {
            Box::new(StringBox::new(
                "Error: randInt() requires two integer inputs",
            ))
        }
    }

    /// true/falseのランダム選択
    pub fn randBool(&self) -> Box<dyn NyashBox> {
        let r = self.next_random();
        Box::new(BoolBox::new(r % 2 == 0))
    }

    /// 配列からランダム選択
    pub fn choice(&self, array: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(array_box) = array.as_any().downcast_ref::<ArrayBox>() {
            let length = array_box
                .length()
                .to_string_box()
                .value
                .parse::<i64>()
                .unwrap_or(0);
            if length == 0 {
                return Box::new(StringBox::new("Error: cannot choose from empty array"));
            }

            let index = self.next_random() % (length as u64);
            // 新しいArrayBox.get()は既にBox<dyn NyashBox>を返すので、直接使用
            array_box.get(Box::new(IntegerBox::new(index as i64)))
        } else {
            Box::new(StringBox::new("Error: choice() requires array input"))
        }
    }

    /// 配列をシャッフル
    pub fn shuffle(&self, array: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(array_box) = array.as_any().downcast_ref::<ArrayBox>() {
            let length = array_box
                .length()
                .to_string_box()
                .value
                .parse::<i64>()
                .unwrap_or(0);
            if length <= 1 {
                return array;
            }

            // 新しい配列を作成
            let shuffled = ArrayBox::new();

            // 元の配列の要素を全て新しい配列にコピー
            for i in 0..length {
                let element = array_box.get(Box::new(IntegerBox::new(i as i64)));
                // NullBoxでなければ追加
                if element.type_name() != "NullBox" {
                    shuffled.push(element);
                }
            }

            // 簡易シャッフル実装（完全なFisher-Yatesは複雑なので）
            // 代わりに、元の配列からランダムに選んで新しい配列を作る
            let result = ArrayBox::new();
            let mut remaining_indices: Vec<usize> = (0..length as usize).collect();

            while !remaining_indices.is_empty() {
                let random_idx = (self.next_random() % remaining_indices.len() as u64) as usize;
                let actual_idx = remaining_indices.remove(random_idx);
                let element = array_box.get(Box::new(IntegerBox::new(actual_idx as i64)));
                // NullBoxでなければ追加
                if element.type_name() != "NullBox" {
                    result.push(element);
                }
            }

            Box::new(result)
        } else {
            Box::new(StringBox::new("Error: shuffle() requires array input"))
        }
    }

    /// ランダムな文字列生成
    pub fn randString(&self, length: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(len_int) = length.as_any().downcast_ref::<IntegerBox>() {
            if len_int.value < 0 {
                return Box::new(StringBox::new("Error: length must be positive"));
            }

            let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            let char_vec: Vec<char> = chars.chars().collect();
            let mut result = String::new();

            for _ in 0..len_int.value {
                let index = self.next_random() % (char_vec.len() as u64);
                result.push(char_vec[index as usize]);
            }

            Box::new(StringBox::new(&result))
        } else {
            Box::new(StringBox::new(
                "Error: randString() requires integer length",
            ))
        }
    }

    /// 指定確率でtrue
    pub fn probability(&self, prob: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(float_box) = prob.as_any().downcast_ref::<FloatBox>() {
            if float_box.value < 0.0 || float_box.value > 1.0 {
                return Box::new(StringBox::new("Error: probability must be 0.0-1.0"));
            }

            let r = self.next_random() as f64 / u64::MAX as f64;
            Box::new(BoolBox::new(r < float_box.value))
        } else if let Some(int_box) = prob.as_any().downcast_ref::<IntegerBox>() {
            let prob_val = int_box.value as f64;
            if prob_val < 0.0 || prob_val > 1.0 {
                return Box::new(StringBox::new("Error: probability must be 0.0-1.0"));
            }

            let r = self.next_random() as f64 / u64::MAX as f64;
            Box::new(BoolBox::new(r < prob_val))
        } else {
            Box::new(StringBox::new(
                "Error: probability() requires numeric input",
            ))
        }
    }
}

impl NyashBox for RandomBox {
    fn type_name(&self) -> &'static str {
        "RandomBox"
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new("RandomBox()")
    }

    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_random) = other.as_any().downcast_ref::<RandomBox>() {
            BoolBox::new(self.base.id == other_random.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl BoxCore for RandomBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RandomBox()")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Display for RandomBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}
