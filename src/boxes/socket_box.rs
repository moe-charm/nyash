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

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock}; // Arc追加
use std::time::Duration;

/// TCP/UDP ソケット操作を提供するBox
#[derive(Debug)]
pub struct SocketBox {
    base: BoxBase,
    // TCP Server
    listener: Arc<RwLock<Option<TcpListener>>>, // Arc追加
    // TCP Client/Connected Socket
    stream: Arc<RwLock<Option<TcpStream>>>, // Arc追加
    // Connection state
    is_server: Arc<RwLock<bool>>,    // Arc追加
    is_connected: Arc<RwLock<bool>>, // Arc追加
}

impl Clone for SocketBox {
    fn clone(&self) -> Self {
        // ディープコピー（独立インスタンス）
        let is_server_val = *self.is_server.read().unwrap();
        let is_connected_val = *self.is_connected.read().unwrap();

        Self {
            base: BoxBase::new(),                                  // New unique ID for clone
            listener: Arc::new(RwLock::new(None)),                 // 新しいArc
            stream: Arc::new(RwLock::new(None)),                   // 新しいArc
            is_server: Arc::new(RwLock::new(is_server_val)),       // 状態のみコピー
            is_connected: Arc::new(RwLock::new(is_connected_val)), // 状態のみコピー
        }
    }
}

impl SocketBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            listener: Arc::new(RwLock::new(None)), // Arc::new追加
            stream: Arc::new(RwLock::new(None)),   // Arc::new追加
            is_server: Arc::new(RwLock::new(false)), // Arc::new追加
            is_connected: Arc::new(RwLock::new(false)), // Arc::new追加
        }
    }

    /// TCP ソケットをアドレス・ポートにバインド
    pub fn bind(&self, address: Box<dyn NyashBox>, port: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let addr_str = address.to_string_box().value;
        let port_str = port.to_string_box().value;

        let socket_addr = format!("{}:{}", addr_str, port_str);

        eprintln!("🔥 SOCKETBOX DEBUG: bind() called");
        eprintln!("🔥   Socket ID = {}", self.base.id);
        eprintln!("🔥   Address = {}", socket_addr);
        eprintln!("🔥   Arc pointer = {:p}", &self.is_server);

        match TcpListener::bind(&socket_addr) {
            Ok(listener) => {
                eprintln!("✅ TCP bind successful");

                // listener設定
                match self.listener.write() {
                    Ok(mut listener_guard) => {
                        *listener_guard = Some(listener);
                        eprintln!("✅ Listener stored successfully");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to lock listener mutex: {}", e);
                        return Box::new(BoolBox::new(false));
                    }
                }

                // is_server状態設定 - 徹底デバッグ
                match self.is_server.write() {
                    Ok(mut is_server_guard) => {
                        eprintln!("🔥 BEFORE MUTATION:");
                        eprintln!("🔥   is_server value = {}", *is_server_guard);
                        eprintln!("🔥   RwLock pointer = {:p}", &self.is_server);
                        eprintln!("🔥   Guard pointer = {:p}", &*is_server_guard);

                        // 状態変更
                        *is_server_guard = true;

                        eprintln!("🔥 AFTER MUTATION:");
                        eprintln!("🔥   is_server value = {}", *is_server_guard);
                        eprintln!("🔥   Value confirmed = {}", *is_server_guard == true);

                        // 明示的にドロップしてロック解除
                        drop(is_server_guard);
                        eprintln!("✅ is_server guard dropped");

                        // 再確認テスト
                        match self.is_server.read() {
                            Ok(check_guard) => {
                                eprintln!("🔥 RECHECK AFTER DROP:");
                                eprintln!("🔥   is_server value = {}", *check_guard);
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to recheck: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ SOCKETBOX: Failed to lock is_server mutex: {}", e);
                        return Box::new(BoolBox::new(false));
                    }
                }

                eprintln!("✅ bind() completed successfully");
                Box::new(BoolBox::new(true))
            }
            Err(e) => {
                eprintln!("❌ TCP bind failed: {}", e);
                Box::new(BoolBox::new(false))
            }
        }
    }

    /// 指定した backlog で接続待機開始
    pub fn listen(&self, backlog: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let _backlog_num = backlog.to_string_box().value.parse::<i32>().unwrap_or(128);

        // Check if listener exists and is properly bound
        let listener_guard = match self.listener.read() {
            Ok(guard) => guard,
            Err(_) => return Box::new(BoolBox::new(false)),
        };

        if let Some(ref listener) = *listener_guard {
            // Try to get the local address to confirm the listener is working
            match listener.local_addr() {
                Ok(_addr) => {
                    // Listener is properly set up and can accept connections
                    Box::new(BoolBox::new(true))
                }
                Err(_) => {
                    // Listener exists but has issues
                    Box::new(BoolBox::new(false))
                }
            }
        } else {
            // No listener bound - this is expected behavior for now
            // HTTPServerBox will handle binding separately
            Box::new(BoolBox::new(false))
        }
    }

    /// クライアント接続を受諾（ブロッキング）
    pub fn accept(&self) -> Box<dyn NyashBox> {
        let listener_guard = self.listener.write().unwrap();
        if let Some(ref listener) = *listener_guard {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    drop(listener_guard);

                    // Create new SocketBox for the client connection
                    let client_socket = SocketBox::new();
                    *client_socket.stream.write().unwrap() = Some(stream);
                    *client_socket.is_connected.write().unwrap() = true;

                    Box::new(client_socket)
                }
                Err(e) => {
                    eprintln!("🚨 SocketBox accept error: {}", e);
                    Box::new(BoolBox::new(false))
                }
            }
        } else {
            Box::new(BoolBox::new(false))
        }
    }

    /// クライアント接続を受諾（タイムアウトms、タイムアウト時はvoid）
    pub fn accept_timeout(&self, timeout_ms: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let ms = timeout_ms.to_string_box().value.parse::<u64>().unwrap_or(0);
        if ms == 0 {
            return self.accept();
        }

        let start = std::time::Instant::now();
        if let Ok(guard) = self.listener.write() {
            if let Some(ref listener) = *guard {
                let _ = listener.set_nonblocking(true);
                loop {
                    match listener.accept() {
                        Ok((stream, _addr)) => {
                            let _ = listener.set_nonblocking(false);
                            drop(guard);
                            let client_socket = SocketBox::new();
                            *client_socket.stream.write().unwrap() = Some(stream);
                            *client_socket.is_connected.write().unwrap() = true;
                            return Box::new(client_socket);
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::WouldBlock {
                                if start.elapsed() >= Duration::from_millis(ms) {
                                    let _ = listener.set_nonblocking(false);
                                    return Box::new(crate::box_trait::VoidBox::new());
                                }
                                std::thread::sleep(Duration::from_millis(5));
                                continue;
                            } else {
                                eprintln!("🚨 SocketBox accept_timeout error: {}", e);
                                let _ = listener.set_nonblocking(false);
                                return Box::new(crate::box_trait::VoidBox::new());
                            }
                        }
                    }
                }
            }
        }
        Box::new(crate::box_trait::VoidBox::new())
    }

    /// サーバーに接続（クライアントモード）
    pub fn connect(
        &self,
        address: Box<dyn NyashBox>,
        port: Box<dyn NyashBox>,
    ) -> Box<dyn NyashBox> {
        let addr_str = address.to_string_box().value;
        let port_str = port.to_string_box().value;

        let socket_addr = format!("{}:{}", addr_str, port_str);

        match TcpStream::connect(&socket_addr) {
            Ok(stream) => {
                // Set timeout for read/write operations
                let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));

                *self.stream.write().unwrap() = Some(stream);
                *self.is_connected.write().unwrap() = true;
                *self.is_server.write().unwrap() = false;
                Box::new(BoolBox::new(true))
            }
            Err(e) => {
                eprintln!("🚨 SocketBox connect error: {}", e);
                Box::new(BoolBox::new(false))
            }
        }
    }

    /// データを読み取り（改行まで or EOF）
    pub fn read(&self) -> Box<dyn NyashBox> {
        let stream_guard = self.stream.write().unwrap();
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
                        }
                        Err(e) => {
                            eprintln!("🚨 SocketBox read error: {}", e);
                            Box::new(StringBox::new("".to_string()))
                        }
                    }
                }
                Err(e) => {
                    eprintln!("🚨 SocketBox stream clone error: {}", e);
                    Box::new(StringBox::new("".to_string()))
                }
            }
        } else {
            Box::new(StringBox::new("".to_string()))
        }
    }

    /// タイムアウト付き読み取り（ms）。タイムアウト時は空文字。
    pub fn recv_timeout(&self, timeout_ms: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        let ms = timeout_ms.to_string_box().value.parse::<u64>().unwrap_or(0);
        let stream_guard = self.stream.write().unwrap();
        if let Some(ref stream) = *stream_guard {
            match stream.try_clone() {
                Ok(stream_clone) => {
                    drop(stream_guard);
                    let _ = stream_clone.set_read_timeout(Some(Duration::from_millis(ms)));
                    let mut reader = BufReader::new(stream_clone);
                    let mut buffer = String::new();
                    match reader.read_line(&mut buffer) {
                        Ok(_) => {
                            if buffer.ends_with('\n') {
                                buffer.pop();
                                if buffer.ends_with('\r') {
                                    buffer.pop();
                                }
                            }
                            Box::new(StringBox::new(&buffer))
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::WouldBlock
                                || e.kind() == std::io::ErrorKind::TimedOut
                            {
                                return Box::new(StringBox::new(""));
                            }
                            eprintln!("🚨 SocketBox recv_timeout error: {}", e);
                            Box::new(StringBox::new(""))
                        }
                    }
                }
                Err(e) => {
                    eprintln!("🚨 SocketBox stream clone error: {}", e);
                    Box::new(StringBox::new(""))
                }
            }
        } else {
            Box::new(StringBox::new(""))
        }
    }

    /// HTTP request を読み取り（ヘッダーまで含む）
    pub fn read_http_request(&self) -> Box<dyn NyashBox> {
        let stream_guard = self.stream.write().unwrap();
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
                            }
                            Err(e) => {
                                eprintln!("🚨 SocketBox HTTP read error: {}", e);
                                break;
                            }
                        }
                    }

                    Box::new(StringBox::new(request))
                }
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

        let mut stream_guard = self.stream.write().unwrap();
        if let Some(ref mut stream) = *stream_guard {
            match stream.write_all(data_str.as_bytes()) {
                Ok(_) => match stream.flush() {
                    Ok(_) => Box::new(BoolBox::new(true)),
                    Err(e) => {
                        eprintln!("🚨 SocketBox flush error: {}", e);
                        Box::new(BoolBox::new(false))
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
        *self.stream.write().unwrap() = None;
        *self.listener.write().unwrap() = None;
        *self.is_connected.write().unwrap() = false;
        *self.is_server.write().unwrap() = false;
        Box::new(BoolBox::new(true))
    }

    /// 接続状態確認
    pub fn is_connected(&self) -> Box<dyn NyashBox> {
        Box::new(BoolBox::new(*self.is_connected.write().unwrap()))
    }

    /// サーバーモード確認
    pub fn is_server(&self) -> Box<dyn NyashBox> {
        eprintln!("🔥 SOCKETBOX DEBUG: is_server() called");
        eprintln!("🔥   Socket ID = {}", self.base.id);
        eprintln!("🔥   RwLock pointer = {:p}", &self.is_server);

        match self.is_server.read() {
            Ok(is_server_guard) => {
                let is_server_value = *is_server_guard;
                eprintln!("🔥 IS_SERVER READ:");
                eprintln!("🔥   is_server value = {}", is_server_value);
                eprintln!("🔥   Guard pointer = {:p}", &*is_server_guard);
                eprintln!("🔥   Returning BoolBox with value = {}", is_server_value);

                Box::new(BoolBox::new(is_server_value))
            }
            Err(e) => {
                eprintln!(
                    "❌ SOCKETBOX: Failed to lock is_server mutex in is_server(): {}",
                    e
                );
                Box::new(BoolBox::new(false))
            }
        }
    }
}

impl NyashBox for SocketBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 🎯 状態共有の核心実装 - SocketBox状態保持問題の根本解決
    fn share_box(&self) -> Box<dyn NyashBox> {
        let new_instance = SocketBox {
            base: BoxBase::new(),                         // 新しいID
            listener: Arc::clone(&self.listener),         // 状態共有
            stream: Arc::clone(&self.stream),             // 状態共有
            is_server: Arc::clone(&self.is_server),       // 状態共有
            is_connected: Arc::clone(&self.is_connected), // 状態共有
        };
        Box::new(new_instance)
    }

    fn to_string_box(&self) -> StringBox {
        eprintln!(
            "🔥 SOCKETBOX to_string_box() called - Socket ID = {}",
            self.base.id
        );
        eprintln!("🔥   RwLock pointer = {:p}", &self.is_server);

        let is_server = match self.is_server.read() {
            Ok(guard) => {
                eprintln!("✅ is_server.read() successful");
                *guard
            }
            Err(e) => {
                eprintln!("❌ is_server.read() failed: {}", e);
                false // デフォルト値
            }
        };

        let is_connected = match self.is_connected.read() {
            Ok(guard) => {
                eprintln!("✅ is_connected.read() successful");
                *guard
            }
            Err(e) => {
                eprintln!("❌ is_connected.read() failed: {}", e);
                false // デフォルト値
            }
        };

        let status = if is_server {
            "Server"
        } else if is_connected {
            "Connected"
        } else {
            "Disconnected"
        };

        StringBox::new(format!(
            "SocketBox(id: {}, status: {})",
            self.base.id, status
        ))
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
        eprintln!(
            "🔥 SOCKETBOX fmt_box() called - Socket ID = {}",
            self.base.id
        );

        let is_server = match self.is_server.read() {
            Ok(guard) => *guard,
            Err(e) => {
                eprintln!("❌ fmt_box: is_server.read() failed: {}", e);
                false
            }
        };

        let is_connected = match self.is_connected.read() {
            Ok(guard) => *guard,
            Err(e) => {
                eprintln!("❌ fmt_box: is_connected.read() failed: {}", e);
                false
            }
        };

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
