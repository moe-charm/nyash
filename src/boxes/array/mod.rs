//! ArrayBox 📦 - 配列・リスト操作（両者一致！）
// Nyashの箱システムによる配列・リスト操作を提供します。
// 参考: 既存Boxの設計思想

pub struct ArrayBox {
    pub items: Vec<Box<dyn std::any::Any>>,
}

impl ArrayBox {
    /// 新しいArrayBoxを作成
    pub fn new() -> Self {
        ArrayBox { items: Vec::new() }
    }
    /// 要素を追加
    pub fn push(&mut self, item: Box<dyn std::any::Any>) {
        self.items.push(item);
    }
    /// 要素数を取得
    pub fn len(&self) -> usize {
        self.items.len()
    }
    /// 要素を取得
    pub fn get(&self, index: usize) -> Option<&Box<dyn std::any::Any>> {
        self.items.get(index)
    }
    /// 要素を削除
    pub fn remove(&mut self, index: usize) -> Option<Box<dyn std::any::Any>> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }
}
