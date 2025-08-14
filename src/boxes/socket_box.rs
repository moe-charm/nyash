/*! 🔌 SocketBox - TCP/UDP Socket networking
 * 
 * ## 📝 概要
 * Rustの std::net を基盤とした高性能ネットワーキング Box
 * TCP サーバー・クライアント両対応、HTTPサーバー基盤として利用
 * 
 * ## 🛠️ 利用可能メソッド
 * ### TCP Server
 * - `bind(address, port)` - TCP ソケット bind
 * - `listen(backlog)` - 接続待機開始
 * - `accept()` - クライアント接続受諾
 * 
 * ### TCP Client  
 * - `connect(address, port)` - サーバーへ接続
 * 
 * ### IO Operations
 * - `read()` - データ読み取り
 * - `write(data)` - データ送信
 * - `close()` - ソケット閉鎖
 * 
 * ## 💡 使用例
 * ```nyash
 * // TCP Server
 * server = new SocketBox()
 * server.bind("0.0.0.0", 8080)
 * server.listen(128)
 * client = server.accept()
 * 
 * // TCP Client
 * client = new SocketBox()
 * client.connect("127.0.0.1", 8080)
 * client.write("Hello Server!")
 * response = client.read()
 * ```
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;
use std::net::{TcpListener, TcpStream, SocketAddr, ToSocketAddrs};
use std::io::{Read, Write, BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// TCP/UDP ソケット操作を提供するBox
#[derive(Debug)]
pub struct SocketBox {
    base: BoxBase,
    // TCP Server
    listener: Arc<Mutex<Option<TcpListener>>>,
    // TCP Client/Connected Socket
    stream: Arc<Mutex<Option<TcpStream>>>,
    // Connection state
    is_server: Arc<Mutex<bool>>,
    is_connected: Arc<Mutex<bool>>,
}

impl Clone for SocketBox {
    fn clone(&self) -> Self {
        // Read the current state values atomically
        let current_is_server = *self.is_server.lock().unwrap();
        let current_is_connected = *self.is_connected.lock().unwrap();
        
        // For listener and stream, we can't clone them, so we'll share them
        // but create new Arc instances with the current state
        let cloned = Self {
            base: BoxBase::new(), // New unique ID for clone
            listener: Arc::clone(&self.listener),  // Share the same listener
            stream: Arc::clone(&self.stream),      // Share the same stream  
            is_server: Arc::new(Mutex::new(current_is_server)),     // New Arc with current value
            is_connected: Arc::new(Mutex::new(current_is_connected)), // New Arc with current value
        };
        
        let original_arc_ptr = Arc::as_ptr(&self.is_server) as usize;
        let cloned_arc_ptr = Arc::as_ptr(&cloned.is_server) as usize;
        println!("🔄 SocketBox::clone() - original Box ID: {}, cloned Box ID: {}, Arc ptr: {:x} -> {:x}, is_server: {}", 
                self.base.id, cloned.base.id, original_arc_ptr, cloned_arc_ptr, current_is_server);
        cloned
    }
}

impl SocketBox {
    pub fn new() -> Self {
        let instance = Self {
            base: BoxBase::new(),
            listener: Arc::new(Mutex::new(None)),
            stream: Arc::new(Mutex::new(None)),
            is_server: Arc::new(Mutex::new(false)),
            is_connected: Arc::new(Mutex::new(false)),
        };
        let arc_ptr = Arc::as_ptr(&instance.is_server) as usize;
        println!("🔧 SocketBox::new() - created (Box ID: {}, Arc ptr: {:x})", instance.base.id, arc_ptr);
        instance
    }
    
    /// TCP ソケットをアドレス・ポートにバインド
    pub fn bind(&self, address: Box<dyn NyashBox>, port: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let addr_str = address.to_string_box().value;
        let port_str = port.to_string_box().value;
        
        let socket_addr = format!("{}:{}", addr_str, port_str);
        println!("🔍 SocketBox::bind() called with address: {} (Box ID: {})", socket_addr, self.base.id);
        
        match TcpListener::bind(&socket_addr) {
            Ok(listener) => {
                println!("✅ SocketBox::bind() - TcpListener created successfully (Box ID: {})", self.base.id);
                match self.listener.lock() {
                    Ok(mut listener_guard) => {
                        *listener_guard = Some(listener);
                        let arc_ptr = Arc::as_ptr(&self.listener) as usize;
                        println!("✅ SocketBox::bind() - Listener stored successfully (Box ID: {}, Arc ptr: {:x})", self.base.id, arc_ptr);
                    },
                    Err(_) => {
                        println!("🚨 SocketBox::bind() - Failed to acquire listener lock (Box ID: {})", self.base.id);
                        return Box::new(BoolBox::new(false));
                    }
                }
                match self.is_server.lock() {
                    Ok(mut is_server_guard) => {
                        *is_server_guard = true;
                        let arc_ptr = Arc::as_ptr(&self.is_server) as usize;
                        // Verify the value was actually set
                        let verify_value = *is_server_guard;
                        println!("✅ SocketBox::bind() - is_server set to true (Box ID: {}, Arc ptr: {:x}, verify: {})", self.base.id, arc_ptr, verify_value);
                        // Also verify it's readable immediately
                        drop(is_server_guard);
                        let reread_value = *self.is_server.lock().unwrap();
                        println!("🔍 SocketBox::bind() - reread value: {}", reread_value);
                    },
                    Err(_) => {
                        println!("🚨 SocketBox::bind() - Failed to acquire is_server lock (Box ID: {})", self.base.id);
                        // Non-critical error, continue
                    }
                }
                Box::new(BoolBox::new(true))
            },
            Err(_e) => {
                // Port might be in use, return false
                Box::new(BoolBox::new(false))
            }
        }
    }
    
    /// 指定した backlog で接続待機開始
    pub fn listen(&self, backlog: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let _backlog_num = backlog.to_string_box().value.parse::<i32>().unwrap_or(128);
        println!("🔍 SocketBox::listen() called (Box ID: {})", self.base.id);
        
        // Check if listener exists and is properly bound
        let listener_guard = match self.listener.lock() {
            Ok(guard) => guard,
            Err(_) => {
                println!("🚨 SocketBox::listen() - Failed to acquire listener lock (Box ID: {})", self.base.id);
                return Box::new(BoolBox::new(false));
            },
        };
        
        println!("🔍 SocketBox::listen() - Listener guard acquired (Box ID: {}), is_some: {}", 
                self.base.id, listener_guard.is_some());
        
        if let Some(ref listener) = *listener_guard {
            println!("✅ SocketBox::listen() - Listener found (Box ID: {})", self.base.id);
            let arc_ptr = Arc::as_ptr(&self.listener) as usize;
            println!("🔍 SocketBox::listen() - Listener Arc ptr: {:x}", arc_ptr);
            // Try to get the local address to confirm the listener is working
            match listener.local_addr() {
                Ok(_addr) => {
                    println!("✅ SocketBox::listen() - Listener is valid (Box ID: {})", self.base.id);
                    // Listener is properly set up and can accept connections
                    Box::new(BoolBox::new(true))
                },
                Err(_) => {
                    println!("🚨 SocketBox::listen() - Listener exists but has issues (Box ID: {})", self.base.id);
                    // Listener exists but has issues
                    Box::new(BoolBox::new(false))
                }
            }
        } else {
            println!("🚨 SocketBox::listen() - No listener bound (Box ID: {})", self.base.id);
            let arc_ptr = Arc::as_ptr(&self.listener) as usize;
            println!("🔍 SocketBox::listen() - Listener Arc ptr: {:x}", arc_ptr);
            // No listener bound - this is expected behavior for now
            // HTTPServerBox will handle binding separately
            Box::new(BoolBox::new(false))
        }
    }
    
    /// クライアント接続を受諾（ブロッキング）
    pub fn accept(&self) -> Box<dyn NyashBox> {
        let listener_guard = self.listener.lock().unwrap();
        if let Some(ref listener) = *listener_guard {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    drop(listener_guard);
                    
                    // Create new SocketBox for the client connection
                    let client_socket = SocketBox::new();
                    *client_socket.stream.lock().unwrap() = Some(stream);
                    *client_socket.is_connected.lock().unwrap() = true;
                    
                    Box::new(client_socket)
                },
                Err(e) => {
                    eprintln!("🚨 SocketBox accept error: {}", e);
                    Box::new(BoolBox::new(false))
                }
            }
        } else {
            Box::new(BoolBox::new(false))
        }
    }
    
    /// サーバーに接続（クライアントモード）
    pub fn connect(&self, address: Box<dyn NyashBox>, port: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let addr_str = address.to_string_box().value;
        let port_str = port.to_string_box().value;
        
        let socket_addr = format!("{}:{}", addr_str, port_str);
        
        match TcpStream::connect(&socket_addr) {
            Ok(stream) => {
                // Set timeout for read/write operations
                let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
                
                *self.stream.lock().unwrap() = Some(stream);
                *self.is_connected.lock().unwrap() = true;
                *self.is_server.lock().unwrap() = false;
                Box::new(BoolBox::new(true))
            },
            Err(e) => {
                eprintln!("🚨 SocketBox connect error: {}", e);
                Box::new(BoolBox::new(false))
            }
        }
    }
    
    /// データを読み取り（改行まで or EOF）
    pub fn read(&self) -> Box<dyn NyashBox> {
        let stream_guard = self.stream.lock().unwrap();
        if let Some(ref stream) = *stream_guard {
            // Clone the stream to avoid borrowing issues
            match stream.try_clone() {
                Ok(stream_clone) => {
                    drop(stream_guard);
                    
                    let mut reader = BufReader::new(stream_clone);
                    let mut buffer = String::new();
                    
                    match reader.read_line(&mut buffer) {
                        Ok(_) => {
                            // Remove trailing newline
                            if buffer.ends_with('\n') {
                                buffer.pop();
                                if buffer.ends_with('\r') {
                                    buffer.pop();
                                }
                            }
                            Box::new(StringBox::new(buffer))
                        },
                        Err(e) => {
                            eprintln!("🚨 SocketBox read error: {}", e);
                            Box::new(StringBox::new("".to_string()))
                        }
                    }
                },
                Err(e) => {
                    eprintln!("🚨 SocketBox stream clone error: {}", e);
                    Box::new(StringBox::new("".to_string()))
                }
            }
        } else {
            Box::new(StringBox::new("".to_string()))
        }
    }
    
    /// HTTP request を読み取り（ヘッダーまで含む）
    pub fn read_http_request(&self) -> Box<dyn NyashBox> {
        let stream_guard = self.stream.lock().unwrap();
        if let Some(ref stream) = *stream_guard {
            match stream.try_clone() {
                Ok(stream_clone) => {
                    drop(stream_guard);
                    
                    let mut reader = BufReader::new(stream_clone);
                    let mut request = String::new();
                    let mut line = String::new();
                    
                    // Read HTTP request line by line until empty line
                    loop {
                        line.clear();
                        match reader.read_line(&mut line) {
                            Ok(0) => break, // EOF
                            Ok(_) => {
                                request.push_str(&line);
                                // Empty line indicates end of headers
                                if line.trim().is_empty() {
                                    break;
                                }
                            },
                            Err(e) => {
                                eprintln!("🚨 SocketBox HTTP read error: {}", e);
                                break;
                            }
                        }
                    }
                    
                    Box::new(StringBox::new(request))
                },
                Err(e) => {
                    eprintln!("🚨 SocketBox stream clone error: {}", e);
                    Box::new(StringBox::new("".to_string()))
                }
            }
        } else {
            Box::new(StringBox::new("".to_string()))
        }
    }
    
    /// データを送信
    pub fn write(&self, data: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let data_str = data.to_string_box().value;
        
        let mut stream_guard = self.stream.lock().unwrap();
        if let Some(ref mut stream) = *stream_guard {
            match stream.write_all(data_str.as_bytes()) {
                Ok(_) => {
                    match stream.flush() {
                        Ok(_) => Box::new(BoolBox::new(true)),
                        Err(e) => {
                            eprintln!("🚨 SocketBox flush error: {}", e);
                            Box::new(BoolBox::new(false))
                        }
                    }
                },
                Err(e) => {
                    eprintln!("🚨 SocketBox write error: {}", e);
                    Box::new(BoolBox::new(false))
                }
            }
        } else {
            Box::new(BoolBox::new(false))
        }
    }
    
    /// ソケット閉鎖
    pub fn close(&self) -> Box<dyn NyashBox> {
        *self.stream.lock().unwrap() = None;
        *self.listener.lock().unwrap() = None;
        *self.is_connected.lock().unwrap() = false;
        *self.is_server.lock().unwrap() = false;
        Box::new(BoolBox::new(true))
    }
    
    /// 接続状態確認
    pub fn is_connected(&self) -> Box<dyn NyashBox> {
        Box::new(BoolBox::new(*self.is_connected.lock().unwrap()))
    }
    
    /// サーバーモード確認
    pub fn is_server(&self) -> Box<dyn NyashBox> {
        let is_server_value = *self.is_server.lock().unwrap();
        let arc_ptr = Arc::as_ptr(&self.is_server) as usize;
        println!("🔍 SocketBox::is_server() called - returning {} (Box ID: {}, Arc ptr: {:x})", 
                is_server_value, self.base.id, arc_ptr);
        // Double-check by re-reading
        let double_check = *self.is_server.lock().unwrap();
        if is_server_value != double_check {
            println!("🚨 WARNING: is_server value changed between reads! {} -> {}", is_server_value, double_check);
        }
        Box::new(BoolBox::new(is_server_value))
    }
}

impl NyashBox for SocketBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    fn to_string_box(&self) -> StringBox {
        let is_server = *self.is_server.lock().unwrap();
        let is_connected = *self.is_connected.lock().unwrap();
        
        let status = if is_server {
            "Server"
        } else if is_connected {
            "Connected"
        } else {
            "Disconnected"
        };
        
        StringBox::new(format!("SocketBox(id: {}, status: {})", self.base.id, status))
    }

    fn type_name(&self) -> &'static str {
        "SocketBox"
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_socket) = other.as_any().downcast_ref::<SocketBox>() {
            BoolBox::new(self.base.id == other_socket.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl BoxCore for SocketBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let is_server = *self.is_server.lock().unwrap();
        let is_connected = *self.is_connected.lock().unwrap();
        
        let status = if is_server {
            "Server"
        } else if is_connected {
            "Connected"
        } else {
            "Disconnected"
        };
        
        write!(f, "SocketBox(id: {}, status: {})", self.base.id, status)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for SocketBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// Auto-cleanup implementation for proper resource management
impl Drop for SocketBox {
    fn drop(&mut self) {
        // Ensure sockets are properly closed
        let _ = self.close();
    }
}