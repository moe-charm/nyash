//! FutureBox 🔄 - 非同期処理基盤
// Nyashの箱システムによる非同期処理の基盤を提供します。
// 参考: 既存Boxの設計思想

use std::future::Future;
use std::pin::Pin;

pub struct FutureBox<T> {
    pub future: Pin<Box<dyn Future<Output = T> + Send>>,
}

impl<T> FutureBox<T> {
    pub fn new<F>(fut: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        FutureBox {
            future: Box::pin(fut),
        }
    }
}
