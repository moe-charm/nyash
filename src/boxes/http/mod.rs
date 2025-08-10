//! HttpClientBox 🌐 - HTTP通信
// Nyashの箱システムによるHTTP通信を提供します。
// 参考: 既存Boxの設計思想

use reqwest::blocking::Client;
use reqwest::Result;

pub struct HttpClientBox {
    pub client: Client,
}

impl HttpClientBox {
    pub fn new() -> Self {
        HttpClientBox {
            client: Client::new(),
        }
    }
    pub fn get(&self, url: &str) -> Result<String> {
        let res = self.client.get(url).send()?.text()?;
        Ok(res)
    }
}
