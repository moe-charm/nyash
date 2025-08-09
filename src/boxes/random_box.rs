/*!
 * Nyash Random Box - Random number generation
 * 
 * 乱数生成を提供するBox型
 * Everything is Box哲学に基づく乱数ライブラリ
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, ArrayBox};
use crate::boxes::math_box::FloatBox;
use std::fmt::{Debug, Display};
use std::any::Any;
use std::sync::{Arc, Mutex};

/// 乱数生成を提供するBox
#[derive(Debug, Clone)]
pub struct RandomBox {
    // 簡易線形合同法による疑似乱数生成器
    seed: Arc<Mutex<u64>>,
    id: u64,
}

impl RandomBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        // 現在時刻を種として使用
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        Self {
            seed: Arc::new(Mutex::new(seed)),
            id,
        }
    }
    
    /// 種を設定
    pub fn seed(&self, new_seed: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = new_seed.as_any().downcast_ref::<IntegerBox>() {
            *self.seed.lock().unwrap() = int_box.value as u64;
            Box::new(StringBox::new("Seed set"))
        } else {
            Box::new(StringBox::new("Error: seed() requires integer input"))
        }
    }
    
    /// 次の乱数を生成（線形合同法）
    fn next_random(&self) -> u64 {
        let mut seed = self.seed.lock().unwrap();
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
            max.as_any().downcast_ref::<IntegerBox>()
        ) {
            if min_int.value > max_int.value {
                return Box::new(StringBox::new("Error: min must be <= max"));
            }
            
            let range = (max_int.value - min_int.value + 1) as u64;
            let r = self.next_random() % range;
            Box::new(IntegerBox::new(min_int.value + r as i64))
        } else {
            Box::new(StringBox::new("Error: randInt() requires two integer inputs"))
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
            let length = array_box.length().to_string_box().value.parse::<i64>().unwrap_or(0);
            if length == 0 {
                return Box::new(StringBox::new("Error: cannot choose from empty array"));
            }
            
            let index = self.next_random() % (length as u64);
            match array_box.get(index as usize) {
                Some(element) => element,
                None => Box::new(StringBox::new("Error: index out of bounds")),
            }
        } else {
            Box::new(StringBox::new("Error: choice() requires array input"))
        }
    }
    
    /// 配列をシャッフル
    pub fn shuffle(&self, array: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(array_box) = array.as_any().downcast_ref::<ArrayBox>() {
            let length = array_box.length().to_string_box().value.parse::<i64>().unwrap_or(0);
            if length <= 1 {
                return array;
            }
            
            // 新しい配列を作成
            let shuffled = ArrayBox::new();
            
            // 元の配列の要素を全て新しい配列にコピー
            for i in 0..length {
                if let Some(element) = array_box.get(i as usize) {
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
                if let Some(element) = array_box.get(actual_idx) {
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
            Box::new(StringBox::new("Error: randString() requires integer length"))
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
            Box::new(StringBox::new("Error: probability() requires numeric input"))
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
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_random) = other.as_any().downcast_ref::<RandomBox>() {
            BoolBox::new(self.id == other_random.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}

impl Display for RandomBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RandomBox()")
    }
}