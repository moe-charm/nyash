/*! 📊 BufferBox - バイナリデータ処理Box
 * 
 * ## 📝 概要
 * バイナリデータの読み書きを扱うBox。
 * ファイル操作、ネットワーク通信、画像処理などで使用。
 * 
 * ## 🛠️ 利用可能メソッド
 * - `write(data)` - バイトデータ書き込み
 * - `read(count)` - 指定バイト数読み取り
 * - `readAll()` - 全データ読み取り
 * - `clear()` - バッファクリア
 * - `length()` - データサイズ取得
 * - `append(buffer)` - 他のBufferを追加
 * - `slice(start, end)` - 部分データ取得
 * 
 * ## 💡 使用例
 * ```nyash
 * local buffer
 * buffer = new BufferBox()
 * 
 * // データ書き込み
 * buffer.write([72, 101, 108, 108, 111])  // "Hello"
 * print("Size: " + buffer.length())
 * 
 * // データ読み取り
 * local data
 * data = buffer.readAll()
 * ```
 */

use crate::box_trait::{NyashBox, StringBox, BoolBox, IntegerBox, BoxCore, BoxBase};
use crate::boxes::array::ArrayBox;
use std::any::Any;
use std::sync::RwLock;
use std::fmt::{Debug, Display};

pub struct BufferBox {
    data: RwLock<Vec<u8>>,
    base: BoxBase,
}

impl BufferBox {
    pub fn new() -> Self {
        BufferBox { 
            data: RwLock::new(Vec::new()),
            base: BoxBase::new(),
        }
    }

    /// Rust向けヘルパー: バッファ長をusizeで取得（テスト用）
    pub fn len(&self) -> usize {
        self.data.read().unwrap().len()
    }
    
    pub fn from_vec(data: Vec<u8>) -> Self {
        BufferBox { 
            data: RwLock::new(data),
            base: BoxBase::new(),
        }
    }
    
    /// データを書き込む
    pub fn write(&self, data: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        // ArrayBoxから変換 - use crate::boxes::array::ArrayBox directly
        if let Some(array_box) = data.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            let mut buffer = self.data.write().unwrap();
            let items = array_box.items.read().unwrap();
            for item in items.iter() {
                if let Some(int_box) = item.as_any().downcast_ref::<IntegerBox>() {
                    if int_box.value >= 0 && int_box.value <= 255 {
                        buffer.push(int_box.value as u8);
                    }
                }
            }
            Box::new(IntegerBox::new(buffer.len() as i64))
        } else {
            let type_name = data.type_name();
            Box::new(StringBox::new(&format!("Error: write() requires ArrayBox of integers, got {}", type_name)))
        }
    }
    
    /// すべてのデータを読み取る
    pub fn readAll(&self) -> Box<dyn NyashBox> {
        let buffer = self.data.read().unwrap();
        let array = ArrayBox::new();
        for &byte in buffer.iter() {
            array.push(Box::new(IntegerBox::new(byte as i64)));
        }
        Box::new(array)
    }
    
    /// 指定バイト数読み取る
    pub fn read(&self, count: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(count_int) = count.as_any().downcast_ref::<IntegerBox>() {
            let mut buffer = self.data.write().unwrap();
            let count = count_int.value.min(buffer.len() as i64) as usize;
            let array = ArrayBox::new();
            
            // 先頭からcount個取り出す
            let bytes: Vec<u8> = buffer.drain(0..count).collect();
            for byte in bytes {
                array.push(Box::new(IntegerBox::new(byte as i64)));
            }
            Box::new(array)
        } else {
            Box::new(StringBox::new("Error: read() requires integer count"))
        }
    }
    
    /// バッファをクリア
    pub fn clear(&self) -> Box<dyn NyashBox> {
        self.data.write().unwrap().clear();
        Box::new(StringBox::new("ok"))
    }
    
    /// データサイズを取得
    pub fn length(&self) -> Box<dyn NyashBox> {
        Box::new(IntegerBox::new(self.data.read().unwrap().len() as i64))
    }
    
    /// 他のBufferBoxを追加
    pub fn append(&self, other: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(other_buffer) = other.as_any().downcast_ref::<BufferBox>() {
            let mut self_data = self.data.write().unwrap();
            let other_data = other_buffer.data.read().unwrap();
            self_data.extend_from_slice(&other_data);
            Box::new(IntegerBox::new(self_data.len() as i64))
        } else {
            Box::new(StringBox::new("Error: append() requires BufferBox"))
        }
    }
    
    /// 部分データ取得
    pub fn slice(&self, start: Box<dyn NyashBox>, end: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let (Some(start_int), Some(end_int)) = (
            start.as_any().downcast_ref::<IntegerBox>(),
            end.as_any().downcast_ref::<IntegerBox>()
        ) {
            let data = self.data.read().unwrap();
            let start = (start_int.value as usize).min(data.len());
            let end = (end_int.value as usize).min(data.len());
            
            if start <= end {
                let slice_data = data[start..end].to_vec();
                Box::new(BufferBox::from_vec(slice_data))
            } else {
                Box::new(StringBox::new("Error: invalid slice range"))
            }
        } else {
            Box::new(StringBox::new("Error: slice() requires integer indices"))
        }
    }
}

// Clone implementation for BufferBox (needed since RwLock doesn't auto-derive Clone)
impl Clone for BufferBox {
    fn clone(&self) -> Self {
        let data = self.data.read().unwrap();
        BufferBox::from_vec(data.clone())
    }
}

impl BoxCore for BufferBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let data = self.data.read().unwrap();
        write!(f, "BufferBox({} bytes)", data.len())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Display for BufferBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

impl NyashBox for BufferBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        let data = self.data.read().unwrap();
        StringBox::new(format!("BufferBox({} bytes)", data.len()))
    }


    fn type_name(&self) -> &'static str {
        "BufferBox"
    }


    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_buffer) = other.as_any().downcast_ref::<BufferBox>() {
            // RwLock内容を比較
            let self_data = self.data.read().unwrap();
            let other_data = other_buffer.data.read().unwrap();
            BoolBox::new(*self_data == *other_data)
        } else {
            BoolBox::new(false)
        }
    }
}
