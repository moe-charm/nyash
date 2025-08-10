//! HttpClientBox 🌐 - HTTP通信
// Nyashの箱システムによるHTTP通信を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use std::any::Any;
use reqwest::blocking::Client;
use reqwest::Result;

#[derive(Debug)]
pub struct HttpClientBox {
    pub client: Client,
    id: u64,
}

impl HttpClientBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        HttpClientBox {
            client: Client::new(),
            id,
        }
    }
    
    pub fn get(&self, url: &str) -> Result<String> {
        let res = self.client.get(url).send()?.text()?;
        Ok(res)
    }
}

impl NyashBox for HttpClientBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // Create a new client instance since Client doesn't implement Clone in a straightforward way
        Box::new(HttpClientBox::new())
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("HttpClientBox(id: {})", self.id))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        "HttpClientBox"
    }

    fn box_id(&self) -> u64 {
        self.id
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_http) = other.as_any().downcast_ref::<HttpClientBox>() {
            BoolBox::new(self.id == other_http.id)
        } else {
            BoolBox::new(false)
        }
    }
}
