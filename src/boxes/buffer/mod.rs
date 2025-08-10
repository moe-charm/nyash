//! BufferBox 📊 - バイナリデータ処理
// Nyashの箱システムによるバイナリデータ処理を提供します。
// 参考: 既存Boxの設計思想

pub struct BufferBox {
    pub data: Vec<u8>,
}

impl BufferBox {
    pub fn new() -> Self {
        BufferBox { data: Vec::new() }
    }
    pub fn from_vec(data: Vec<u8>) -> Self {
        BufferBox { data }
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}
