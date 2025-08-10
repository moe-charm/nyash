//! ResultBox ⚠️ - エラー処理（ResultBox推奨）
// Nyashの箱システムによるエラー処理を提供します。
// 参考: 既存Boxの設計思想

pub enum ResultBox<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> ResultBox<T, E> {
    pub fn is_ok(&self) -> bool {
        matches!(self, ResultBox::Ok(_))
    }
    pub fn is_err(&self) -> bool {
        matches!(self, ResultBox::Err(_))
    }
    pub fn unwrap(self) -> T {
        match self {
            ResultBox::Ok(val) => val,
            ResultBox::Err(_) => panic!("called `unwrap()` on an `Err` value"),
        }
    }
}
