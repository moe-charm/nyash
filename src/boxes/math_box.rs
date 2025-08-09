/*!
 * Nyash Math Box - Mathematical operations
 * 
 * 数学演算を提供するBox型
 * Everything is Box哲学に基づく数学ライブラリ
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox};
use std::fmt::{Debug, Display};
use std::any::Any;

/// 数学演算を提供するBox
#[derive(Debug, Clone)]
pub struct MathBox {
    id: u64,
}

impl MathBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self { id }
    }
    
    /// 絶対値を計算
    pub fn abs(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            Box::new(IntegerBox::new(int_box.value.abs()))
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            Box::new(FloatBox::new(float_box.value.abs()))
        } else {
            Box::new(StringBox::new("Error: abs() requires numeric input"))
        }
    }
    
    /// 最大値を返す
    pub fn max(&self, a: Box<dyn NyashBox>, b: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let (Some(a_int), Some(b_int)) = (
            a.as_any().downcast_ref::<IntegerBox>(),
            b.as_any().downcast_ref::<IntegerBox>()
        ) {
            Box::new(IntegerBox::new(a_int.value.max(b_int.value)))
        } else if let (Some(a_float), Some(b_float)) = (
            a.as_any().downcast_ref::<FloatBox>(),
            b.as_any().downcast_ref::<FloatBox>()
        ) {
            Box::new(FloatBox::new(a_float.value.max(b_float.value)))
        } else {
            Box::new(StringBox::new("Error: max() requires numeric inputs"))
        }
    }
    
    /// 最小値を返す
    pub fn min(&self, a: Box<dyn NyashBox>, b: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let (Some(a_int), Some(b_int)) = (
            a.as_any().downcast_ref::<IntegerBox>(),
            b.as_any().downcast_ref::<IntegerBox>()
        ) {
            Box::new(IntegerBox::new(a_int.value.min(b_int.value)))
        } else if let (Some(a_float), Some(b_float)) = (
            a.as_any().downcast_ref::<FloatBox>(),
            b.as_any().downcast_ref::<FloatBox>()
        ) {
            Box::new(FloatBox::new(a_float.value.min(b_float.value)))
        } else {
            Box::new(StringBox::new("Error: min() requires numeric inputs"))
        }
    }
    
    /// 累乗を計算
    pub fn pow(&self, base: Box<dyn NyashBox>, exp: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let (Some(base_int), Some(exp_int)) = (
            base.as_any().downcast_ref::<IntegerBox>(),
            exp.as_any().downcast_ref::<IntegerBox>()
        ) {
            if exp_int.value >= 0 {
                let result = (base_int.value as f64).powi(exp_int.value as i32);
                Box::new(FloatBox::new(result))
            } else {
                Box::new(StringBox::new("Error: negative exponent"))
            }
        } else {
            Box::new(StringBox::new("Error: pow() requires numeric inputs"))
        }
    }
    
    /// 平方根を計算
    pub fn sqrt(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            if int_box.value >= 0 {
                Box::new(FloatBox::new((int_box.value as f64).sqrt()))
            } else {
                Box::new(StringBox::new("Error: sqrt() of negative number"))
            }
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            if float_box.value >= 0.0 {
                Box::new(FloatBox::new(float_box.value.sqrt()))
            } else {
                Box::new(StringBox::new("Error: sqrt() of negative number"))
            }
        } else {
            Box::new(StringBox::new("Error: sqrt() requires numeric input"))
        }
    }
    
    /// 円周率πを返す
    #[allow(non_snake_case)]
    pub fn getPi(&self) -> Box<dyn NyashBox> {
        Box::new(FloatBox::new(std::f64::consts::PI))
    }
    
    /// 自然対数の底eを返す
    #[allow(non_snake_case)]
    pub fn getE(&self) -> Box<dyn NyashBox> {
        Box::new(FloatBox::new(std::f64::consts::E))
    }
    
    /// サイン（正弦）
    pub fn sin(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            Box::new(FloatBox::new((int_box.value as f64).sin()))
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            Box::new(FloatBox::new(float_box.value.sin()))
        } else {
            Box::new(StringBox::new("Error: sin() requires numeric input"))
        }
    }
    
    /// コサイン（余弦）
    pub fn cos(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            Box::new(FloatBox::new((int_box.value as f64).cos()))
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            Box::new(FloatBox::new(float_box.value.cos()))
        } else {
            Box::new(StringBox::new("Error: cos() requires numeric input"))
        }
    }
    
    /// タンジェント（正接）
    pub fn tan(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            Box::new(FloatBox::new((int_box.value as f64).tan()))
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            Box::new(FloatBox::new(float_box.value.tan()))
        } else {
            Box::new(StringBox::new("Error: tan() requires numeric input"))
        }
    }
    
    /// 自然対数
    pub fn log(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            if int_box.value > 0 {
                Box::new(FloatBox::new((int_box.value as f64).ln()))
            } else {
                Box::new(StringBox::new("Error: log() of non-positive number"))
            }
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            if float_box.value > 0.0 {
                Box::new(FloatBox::new(float_box.value.ln()))
            } else {
                Box::new(StringBox::new("Error: log() of non-positive number"))
            }
        } else {
            Box::new(StringBox::new("Error: log() requires numeric input"))
        }
    }
    
    /// 常用対数（底10）
    pub fn log10(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            if int_box.value > 0 {
                Box::new(FloatBox::new((int_box.value as f64).log10()))
            } else {
                Box::new(StringBox::new("Error: log10() of non-positive number"))
            }
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            if float_box.value > 0.0 {
                Box::new(FloatBox::new(float_box.value.log10()))
            } else {
                Box::new(StringBox::new("Error: log10() of non-positive number"))
            }
        } else {
            Box::new(StringBox::new("Error: log10() requires numeric input"))
        }
    }
    
    /// 指数関数（e^x）
    pub fn exp(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            Box::new(FloatBox::new((int_box.value as f64).exp()))
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            Box::new(FloatBox::new(float_box.value.exp()))
        } else {
            Box::new(StringBox::new("Error: exp() requires numeric input"))
        }
    }
    
    /// 床関数（切り下げ）
    pub fn floor(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            Box::new(IntegerBox::new(int_box.value))  // 整数はそのまま
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            Box::new(IntegerBox::new(float_box.value.floor() as i64))
        } else {
            Box::new(StringBox::new("Error: floor() requires numeric input"))
        }
    }
    
    /// 天井関数（切り上げ）
    pub fn ceil(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            Box::new(IntegerBox::new(int_box.value))  // 整数はそのまま
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            Box::new(IntegerBox::new(float_box.value.ceil() as i64))
        } else {
            Box::new(StringBox::new("Error: ceil() requires numeric input"))
        }
    }
    
    /// 四捨五入
    pub fn round(&self, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(int_box) = value.as_any().downcast_ref::<IntegerBox>() {
            Box::new(IntegerBox::new(int_box.value))  // 整数はそのまま
        } else if let Some(float_box) = value.as_any().downcast_ref::<FloatBox>() {
            Box::new(IntegerBox::new(float_box.value.round() as i64))
        } else {
            Box::new(StringBox::new("Error: round() requires numeric input"))
        }
    }
}

impl NyashBox for MathBox {
    fn type_name(&self) -> &'static str {
        "MathBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new("MathBox()")
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_math) = other.as_any().downcast_ref::<MathBox>() {
            BoolBox::new(self.id == other_math.id)
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

impl Display for MathBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MathBox()")
    }
}

/// 浮動小数点数Box
#[derive(Debug, Clone)]
pub struct FloatBox {
    pub value: f64,
    id: u64,
}

impl FloatBox {
    pub fn new(value: f64) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self { value, id }
    }
}

impl NyashBox for FloatBox {
    fn type_name(&self) -> &'static str {
        "FloatBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new(&self.value.to_string())
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_float) = other.as_any().downcast_ref::<FloatBox>() {
            BoolBox::new((self.value - other_float.value).abs() < f64::EPSILON)
        } else if let Some(other_int) = other.as_any().downcast_ref::<IntegerBox>() {
            BoolBox::new((self.value - other_int.value as f64).abs() < f64::EPSILON)
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

impl Display for FloatBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// 範囲を表すBox
#[derive(Debug, Clone)]
pub struct RangeBox {
    pub start: i64,
    pub end: i64,
    pub step: i64,
    id: u64,
}

impl RangeBox {
    pub fn new(start: i64, end: i64, step: i64) -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self { start, end, step, id }
    }
    
    /// イテレータとして値を生成
    pub fn iter(&self) -> Vec<i64> {
        let mut result = Vec::new();
        let mut current = self.start;
        
        if self.step > 0 {
            while current < self.end {
                result.push(current);
                current += self.step;
            }
        } else if self.step < 0 {
            while current > self.end {
                result.push(current);
                current += self.step;
            }
        }
        
        result
    }
}

impl NyashBox for RangeBox {
    fn type_name(&self) -> &'static str {
        "RangeBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new(&format!("Range({}, {}, {})", self.start, self.end, self.step))
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_range) = other.as_any().downcast_ref::<RangeBox>() {
            BoolBox::new(
                self.start == other_range.start &&
                self.end == other_range.end &&
                self.step == other_range.step
            )
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

impl Display for RangeBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Range({}, {}, {})", self.start, self.end, self.step)
    }
}