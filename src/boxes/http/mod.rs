//! HttpClientBox 🌐 - HTTP通信
// Nyashの箱システムによるHTTP通信を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{NyashBox, StringBox, BoolBox};
use crate::boxes::map_box::MapBox;
use std::any::Any;
use std::sync::{Arc, Mutex};
use reqwest::blocking::Client;
use reqwest::Result;

#[derive(Debug, Clone)]
pub struct HttpClientBox {
    client: Arc<Mutex<Client>>,
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
            client: Arc::new(Mutex::new(Client::new())),
            id,
        }
    }
    
    pub fn get(&self, url: &str) -> Result<String> {
        let client = self.client.lock().unwrap();
        let res = client.get(url).send()?.text()?;
        Ok(res)
    }
    
    /// HTTP GETリクエスト
    pub fn http_get(&self, url: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let url_str = url.to_string_box().value;
        match self.get(&url_str) {
            Ok(response) => Box::new(StringBox::new(&response)),
            Err(e) => Box::new(StringBox::new(&format!("Error in HTTP GET: {}", e))),
        }
    }
    
    /// HTTP POSTリクエスト
    pub fn post(&self, url: Box<dyn NyashBox>, body: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let url_str = url.to_string_box().value;
        let body_str = body.to_string_box().value;
        
        let client = self.client.lock().unwrap();
        match client.post(&url_str).body(body_str).send() {
            Ok(response) => {
                match response.text() {
                    Ok(text) => Box::new(StringBox::new(&text)),
                    Err(e) => Box::new(StringBox::new(&format!("Error reading response: {}", e))),
                }
            },
            Err(e) => Box::new(StringBox::new(&format!("Error in HTTP POST: {}", e))),
        }
    }
    
    /// HTTP PUT リクエスト
    pub fn put(&self, url: Box<dyn NyashBox>, body: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let url_str = url.to_string_box().value;
        let body_str = body.to_string_box().value;
        
        let client = self.client.lock().unwrap();
        match client.put(&url_str).body(body_str).send() {
            Ok(response) => {
                match response.text() {
                    Ok(text) => Box::new(StringBox::new(&text)),
                    Err(e) => Box::new(StringBox::new(&format!("Error reading response: {}", e))),
                }
            },
            Err(e) => Box::new(StringBox::new(&format!("Error in HTTP PUT: {}", e))),
        }
    }
    
    /// HTTP DELETE リクエスト
    pub fn delete(&self, url: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let url_str = url.to_string_box().value;
        
        let client = self.client.lock().unwrap();
        match client.delete(&url_str).send() {
            Ok(response) => {
                match response.text() {
                    Ok(text) => Box::new(StringBox::new(&text)),
                    Err(e) => Box::new(StringBox::new(&format!("Error reading response: {}", e))),
                }
            },
            Err(e) => Box::new(StringBox::new(&format!("Error in HTTP DELETE: {}", e))),
        }
    }
    
    /// ヘッダー付きHTTPリクエスト
    pub fn request(&self, method: Box<dyn NyashBox>, url: Box<dyn NyashBox>, options: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let method_str = method.to_string_box().value.to_uppercase();
        let url_str = url.to_string_box().value;
        
        // optionsはMapBoxと仮定
        if let Some(map_box) = options.as_any().downcast_ref::<MapBox>() {
            let client = self.client.lock().unwrap();
            let mut request = match method_str.as_str() {
                "GET" => client.get(&url_str),
                "POST" => client.post(&url_str),
                "PUT" => client.put(&url_str),
                "DELETE" => client.delete(&url_str),
                _ => return Box::new(StringBox::new(&format!("Unsupported HTTP method: {}", method_str))),
            };
            
            // ヘッダー設定
            if let Some(headers_box) = map_box.get(Box::new(StringBox::new("headers"))).as_any().downcast_ref::<MapBox>() {
                let headers_map = headers_box.map.lock().unwrap();
                for (key, value) in headers_map.iter() {
                    request = request.header(key, value.to_string_box().value);
                }
            }
            
            // ボディ設定
            if let Some(body_box) = map_box.get(Box::new(StringBox::new("body"))).as_any().downcast_ref::<StringBox>() {
                request = request.body(body_box.value.clone());
            }
            
            match request.send() {
                Ok(response) => {
                    match response.text() {
                        Ok(text) => Box::new(StringBox::new(&text)),
                        Err(e) => Box::new(StringBox::new(&format!("Error reading response: {}", e))),
                    }
                },
                Err(e) => Box::new(StringBox::new(&format!("Error in HTTP request: {}", e))),
            }
        } else {
            Box::new(StringBox::new("Error: options must be a MapBox"))
        }
    }
}

impl NyashBox for HttpClientBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
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
