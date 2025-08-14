/*! 📬 HTTPRequestBox & HTTPResponseBox - HTTP メッセージ処理
 * 
 * ## 📝 概要
 * HTTP/1.1 プロトコルのリクエスト・レスポンス処理を提供するBox群
 * SocketBox と連携して完全なHTTPサーバー・クライアント機能を実現
 * 
 * ## 🛠️ HTTPRequestBox - リクエスト処理
 * ### HTTP Method & URL
 * - `getMethod()` - HTTP メソッド取得 (GET, POST, etc.)
 * - `getPath()` - URL パス取得
 * - `getQueryString()` - クエリ文字列取得
 * 
 * ### Headers
 * - `getHeader(name)` - 特定ヘッダー取得
 * - `getAllHeaders()` - 全ヘッダー取得（MapBox）
 * - `hasHeader(name)` - ヘッダー存在確認
 * 
 * ### Body & Content
 * - `getBody()` - リクエストボディ取得
 * - `getContentType()` - Content-Type取得
 * - `getContentLength()` - Content-Length取得
 * 
 * ## 🛠️ HTTPResponseBox - レスポンス生成
 * ### Status & Headers
 * - `setStatus(code, message)` - ステータス設定
 * - `setHeader(name, value)` - ヘッダー設定
 * - `setContentType(type)` - Content-Type設定
 * 
 * ### Body & Output
 * - `setBody(content)` - レスポンスボディ設定
 * - `appendBody(content)` - ボディ追加
 * - `toHttpString()` - HTTP形式文字列生成
 * 
 * ## 💡 使用例
 * ```nyash
 * // Request parsing
 * local rawRequest = socket.readHttpRequest()
 * local request = HTTPRequestBox.parse(rawRequest)
 * print("Method: " + request.getMethod())
 * print("Path: " + request.getPath())
 * 
 * // Response generation
 * local response = new HTTPResponseBox()
 * response.setStatus(200, "OK")
 * response.setContentType("application/json")
 * response.setBody("{\"message\": \"Hello World\"}")
 * socket.write(response.toHttpString())
 * ```
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, BoxCore, BoxBase};
use crate::boxes::MapBox;
use std::any::Any;
use std::collections::HashMap;

/// HTTP リクエストを解析・操作するBox
#[derive(Debug, Clone)]
pub struct HTTPRequestBox {
    base: BoxBase,
    method: String,
    path: String,
    query_string: String,
    headers: HashMap<String, String>,
    body: String,
    http_version: String,
}

impl HTTPRequestBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            method: "GET".to_string(),
            path: "/".to_string(),
            query_string: "".to_string(),
            headers: HashMap::new(),
            body: "".to_string(),
            http_version: "HTTP/1.1".to_string(),
        }
    }
    
    /// 生のHTTPリクエスト文字列を解析
    pub fn parse(raw_request: Box<dyn NyashBox>) -> Self {
        let request_str = raw_request.to_string_box().value;
        let mut request = HTTPRequestBox::new();
        
        let lines: Vec<&str> = request_str.lines().collect();
        if lines.is_empty() {
            return request;
        }
        
        // Parse request line: "GET /path HTTP/1.1"
        let request_line_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if request_line_parts.len() >= 3 {
            request.method = request_line_parts[0].to_string();
            
            // Split path and query string
            let url_parts: Vec<&str> = request_line_parts[1].splitn(2, '?').collect();
            request.path = url_parts[0].to_string();
            if url_parts.len() > 1 {
                request.query_string = url_parts[1].to_string();
            }
            
            request.http_version = request_line_parts[2].to_string();
        }
        
        // Parse headers
        let mut header_end = 1;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim().is_empty() {
                header_end = i + 1;
                break;
            }
            
            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();
                request.headers.insert(name, value);
            }
        }
        
        // Parse body (everything after headers)
        if header_end < lines.len() {
            request.body = lines[header_end..].join("\n");
        }
        
        request
    }
    
    /// HTTP メソッド取得
    pub fn get_method(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(self.method.clone()))
    }
    
    /// URL パス取得
    pub fn get_path(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(self.path.clone()))
    }
    
    /// クエリ文字列取得
    pub fn get_query_string(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(self.query_string.clone()))
    }
    
    /// 特定ヘッダー取得
    pub fn get_header(&self, name: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let header_name = name.to_string_box().value.to_lowercase();
        match self.headers.get(&header_name) {
            Some(value) => Box::new(StringBox::new(value.clone())),
            None => Box::new(StringBox::new("".to_string())),
        }
    }
    
    /// 全ヘッダー取得（MapBox形式）
    pub fn get_all_headers(&self) -> Box<dyn NyashBox> {
        let headers_map = MapBox::new();
        for (name, value) in &self.headers {
            let name_box = Box::new(StringBox::new(name.clone()));
            let value_box = Box::new(StringBox::new(value.clone()));
            headers_map.set(name_box, value_box);
        }
        Box::new(headers_map)
    }
    
    /// ヘッダー存在確認
    pub fn has_header(&self, name: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let header_name = name.to_string_box().value.to_lowercase();
        Box::new(BoolBox::new(self.headers.contains_key(&header_name)))
    }
    
    /// リクエストボディ取得
    pub fn get_body(&self) -> Box<dyn NyashBox> {
        Box::new(StringBox::new(self.body.clone()))
    }
    
    /// Content-Type取得
    pub fn get_content_type(&self) -> Box<dyn NyashBox> {
        self.get_header(Box::new(StringBox::new("content-type".to_string())))
    }
    
    /// Content-Length取得
    pub fn get_content_length(&self) -> Box<dyn NyashBox> {
        match self.headers.get("content-length") {
            Some(length_str) => {
                match length_str.parse::<i64>() {
                    Ok(length) => Box::new(IntegerBox::new(length)),
                    Err(_) => Box::new(IntegerBox::new(0)),
                }
            },
            None => Box::new(IntegerBox::new(0)),
        }
    }
}

impl NyashBox for HTTPRequestBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("HTTPRequest({} {} - {} headers)", 
                              self.method, self.path, self.headers.len()))
    }

    fn type_name(&self) -> &'static str {
        "HTTPRequestBox"
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_req) = other.as_any().downcast_ref::<HTTPRequestBox>() {
            BoolBox::new(self.base.id == other_req.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl BoxCore for HTTPRequestBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTPRequest({} {} - {} headers)", 
               self.method, self.path, self.headers.len())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for HTTPRequestBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

/// HTTP レスポンスを生成・操作するBox
#[derive(Debug, Clone)]
pub struct HTTPResponseBox {
    base: BoxBase,
    status_code: i32,
    status_message: String,
    headers: HashMap<String, String>,
    body: String,
    http_version: String,
}

impl HTTPResponseBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            status_code: 200,
            status_message: "OK".to_string(),
            headers: HashMap::new(),
            body: "".to_string(),
            http_version: "HTTP/1.1".to_string(),
        }
    }
    
    /// ステータスコード・メッセージ設定
    pub fn set_status(&self, code: Box<dyn NyashBox>, message: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        // Note: This would need interior mutability for actual mutation
        // For now, this is a placeholder for the API structure
        let _code_val = code.to_string_box().value.parse::<i32>().unwrap_or(200);
        let _message_val = message.to_string_box().value;
        
        // TODO: Use RefCell or similar for interior mutability
        Box::new(BoolBox::new(true))
    }
    
    /// ヘッダー設定
    pub fn set_header(&self, name: Box<dyn NyashBox>, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let _name_str = name.to_string_box().value;
        let _value_str = value.to_string_box().value;
        
        // TODO: Use RefCell for interior mutability
        Box::new(BoolBox::new(true))
    }
    
    /// Content-Type設定
    pub fn set_content_type(&self, content_type: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let content_type_str = content_type.to_string_box().value;
        self.set_header(
            Box::new(StringBox::new("Content-Type".to_string())),
            Box::new(StringBox::new(content_type_str))
        )
    }
    
    /// レスポンスボディ設定
    pub fn set_body(&self, content: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let _content_str = content.to_string_box().value;
        
        // TODO: Use RefCell for interior mutability
        Box::new(BoolBox::new(true))
    }
    
    /// ボディ追加
    pub fn append_body(&self, content: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let _content_str = content.to_string_box().value;
        
        // TODO: Use RefCell for interior mutability
        Box::new(BoolBox::new(true))
    }
    
    /// HTTP形式文字列生成
    pub fn to_http_string(&self) -> Box<dyn NyashBox> {
        let mut response = String::new();
        
        // Status line
        response.push_str(&format!("{} {} {}\r\n", 
                                  self.http_version, self.status_code, self.status_message));
        
        // Headers
        for (name, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", name, value));
        }
        
        // Content-Length if not already set
        if !self.headers.contains_key("content-length") && !self.body.is_empty() {
            response.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
        }
        
        // Empty line before body
        response.push_str("\r\n");
        
        // Body
        response.push_str(&self.body);
        
        Box::new(StringBox::new(response))
    }
    
    /// Quick HTML response creation
    pub fn create_html_response(content: Box<dyn NyashBox>) -> Self {
        let mut response = HTTPResponseBox::new();
        response.status_code = 200;
        response.status_message = "OK".to_string();
        response.headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        response.body = content.to_string_box().value;
        response
    }
    
    /// Quick JSON response creation
    pub fn create_json_response(content: Box<dyn NyashBox>) -> Self {
        let mut response = HTTPResponseBox::new();
        response.status_code = 200;
        response.status_message = "OK".to_string();
        response.headers.insert("Content-Type".to_string(), "application/json".to_string());
        response.body = content.to_string_box().value;
        response
    }
    
    /// Quick 404 response creation
    pub fn create_404_response() -> Self {
        let mut response = HTTPResponseBox::new();
        response.status_code = 404;
        response.status_message = "Not Found".to_string();
        response.headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        response.body = "<html><body><h1>404 - Not Found</h1></body></html>".to_string();
        response
    }
}

impl NyashBox for HTTPResponseBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("HTTPResponse({} {} - {} bytes)", 
                              self.status_code, self.status_message, self.body.len()))
    }

    fn type_name(&self) -> &'static str {
        "HTTPResponseBox"
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_resp) = other.as_any().downcast_ref::<HTTPResponseBox>() {
            BoolBox::new(self.base.id == other_resp.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl BoxCore for HTTPResponseBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTPResponse({} {} - {} bytes)", 
               self.status_code, self.status_message, self.body.len())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for HTTPResponseBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}