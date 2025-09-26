//! HttpClientBox 🌐 - HTTP通信
// Nyashの箱システムによるHTTP通信を提供します。
// 参考: 既存Boxの設計思想
//
// NOTE: HTTPサポートは現在開発中です。
// reqwestクレートの依存関係のため、一時的に無効化されています。

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct HttpClientBox {
    base: BoxBase,
}

impl HttpClientBox {
    pub fn new() -> Self {
        HttpClientBox {
            base: BoxBase::new(),
        }
    }

    /// HTTP GETリクエスト（スタブ）
    pub fn http_get(&self, _url: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        Box::new(StringBox::new("HTTP support is currently disabled"))
    }

    /// HTTP POSTリクエスト（スタブ）
    pub fn post(&self, _url: Box<dyn NyashBox>, _body: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        Box::new(StringBox::new("HTTP support is currently disabled"))
    }

    /// HTTP PUT リクエスト（スタブ）
    pub fn put(&self, _url: Box<dyn NyashBox>, _body: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        Box::new(StringBox::new("HTTP support is currently disabled"))
    }

    /// HTTP DELETE リクエスト（スタブ）
    pub fn delete(&self, _url: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        Box::new(StringBox::new("HTTP support is currently disabled"))
    }

    /// ヘッダー付きHTTPリクエスト（スタブ）
    pub fn request(
        &self,
        _method: Box<dyn NyashBox>,
        _url: Box<dyn NyashBox>,
        _options: Box<dyn NyashBox>,
    ) -> Box<dyn NyashBox> {
        Box::new(StringBox::new("HTTP support is currently disabled"))
    }
}

impl NyashBox for HttpClientBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("HttpClientBox(id: {})", self.base.id))
    }

    fn type_name(&self) -> &'static str {
        "HttpClientBox"
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_http) = other.as_any().downcast_ref::<HttpClientBox>() {
            BoolBox::new(self.base.id == other_http.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl BoxCore for HttpClientBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpClientBox(id: {})", self.base.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for HttpClientBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}
