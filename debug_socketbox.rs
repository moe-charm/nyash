// 🔍 デバッグ版SocketBox - 全操作を詳細ログ出力

use std::sync::{Arc, Mutex};
use std::fs::OpenOptions;
use std::io::Write;

// デバッグログをファイルに出力
fn debug_log(message: &str) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let log_message = format!("[{}] {}\n", timestamp, message);
    
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug_socketbox.log") 
    {
        let _ = file.write_all(log_message.as_bytes());
        let _ = file.flush();
    }
    
    // コンソールにも出力
    print!("{}", log_message);
}

#[derive(Debug)]
pub struct DebugSocketBox {
    id: u64,
    listener: Arc<Mutex<Option<std::net::TcpListener>>>,
    is_server: Arc<Mutex<bool>>,
}

impl DebugSocketBox {
    pub fn new() -> Self {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        let instance = Self {
            id,
            listener: Arc::new(Mutex::new(None)),
            is_server: Arc::new(Mutex::new(false)),
        };
        
        debug_log(&format!("🆕 NEW SocketBox created: id={}, is_server_ptr={:p}", 
                          instance.id, &*instance.is_server));
        
        instance
    }
    
    pub fn bind(&self, addr: &str, port: &str) -> bool {
        debug_log(&format!("🔗 BIND called on id={}, is_server_ptr={:p}", 
                          self.id, &*self.is_server));
        
        let socket_addr = format!("{}:{}", addr, port);
        debug_log(&format!("🔗 BIND address: {}", socket_addr));
        
        match std::net::TcpListener::bind(&socket_addr) {
            Ok(listener) => {
                // listener設定
                match self.listener.lock() {
                    Ok(mut listener_guard) => {
                        *listener_guard = Some(listener);
                        debug_log(&format!("✅ BIND listener set successfully on id={}", self.id));
                    },
                    Err(e) => {
                        debug_log(&format!("❌ BIND listener lock failed: {:?}", e));
                        return false;
                    }
                }
                
                // is_server=true設定
                debug_log(&format!("🔧 BIND setting is_server=true on id={}, ptr={:p}", 
                                  self.id, &*self.is_server));
                
                match self.is_server.lock() {
                    Ok(mut is_server_guard) => {
                        let old_value = *is_server_guard;
                        *is_server_guard = true;
                        debug_log(&format!("✅ BIND is_server changed: {} -> true on id={}", 
                                          old_value, self.id));
                    },
                    Err(e) => {
                        debug_log(&format!("❌ BIND is_server lock failed: {:?}", e));
                    }
                }
                
                debug_log(&format!("🎉 BIND completed successfully on id={}", self.id));
                true
            },
            Err(e) => {
                debug_log(&format!("❌ BIND failed: {:?}", e));
                false
            }
        }
    }
    
    pub fn is_server(&self) -> bool {
        debug_log(&format!("❓ IS_SERVER called on id={}, ptr={:p}", 
                          self.id, &*self.is_server));
        
        match self.is_server.lock() {
            Ok(is_server_guard) => {
                let value = *is_server_guard;
                debug_log(&format!("📖 IS_SERVER result: {} on id={}", value, self.id));
                value
            },
            Err(e) => {
                debug_log(&format!("❌ IS_SERVER lock failed: {:?}", e));
                false
            }
        }
    }
}

impl Clone for DebugSocketBox {
    fn clone(&self) -> Self {
        debug_log(&format!("🔄 CLONE called on id={}", self.id));
        debug_log(&format!("🔄 CLONE original is_server_ptr={:p}", &*self.is_server));
        
        let new_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
            
        let cloned = Self {
            id: new_id, // 新しいID
            listener: Arc::clone(&self.listener),
            is_server: Arc::clone(&self.is_server), // ✅ Arc共有
        };
        
        debug_log(&format!("🔄 CLONE created: old_id={} -> new_id={}", self.id, cloned.id));
        debug_log(&format!("🔄 CLONE new is_server_ptr={:p}", &*cloned.is_server));
        debug_log(&format!("🔄 CLONE Arc共有確認: {} == {}", 
                          Arc::as_ptr(&self.is_server) as *const _ as usize,
                          Arc::as_ptr(&cloned.is_server) as *const _ as usize));
        
        cloned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debug_socketbox() {
        // ログファイルをクリア
        std::fs::write("debug_socketbox.log", "").unwrap();
        
        debug_log("🚀 === DEBUG TEST START ===");
        
        // Step 1: 作成
        let socket = DebugSocketBox::new();
        debug_log(&format!("Step 1 completed: id={}", socket.id));
        
        // Step 2: bind実行
        debug_log("🔥 Step 2: BIND execution");
        let bind_result = socket.bind("127.0.0.1", "18080");
        debug_log(&format!("Step 2 completed: bind_result={}", bind_result));
        
        // Step 3: 状態確認
        debug_log("🔥 Step 3: Check state after bind");
        let is_server1 = socket.is_server();
        debug_log(&format!("Step 3 completed: is_server={}", is_server1));
        
        // Step 4: clone実行
        debug_log("🔥 Step 4: CLONE execution");
        let socket_cloned = socket.clone();
        
        // Step 5: clone後の状態確認
        debug_log("🔥 Step 5: Check state after clone");
        let is_server2 = socket_cloned.is_server();
        debug_log(&format!("Step 5 completed: cloned is_server={}", is_server2));
        
        // Step 6: 元の状態確認
        debug_log("🔥 Step 6: Check original after clone");
        let is_server3 = socket.is_server();
        debug_log(&format!("Step 6 completed: original is_server={}", is_server3));
        
        debug_log("🎉 === DEBUG TEST COMPLETED ===");
        
        assert!(bind_result, "bind should succeed");
        assert!(is_server1, "is_server should be true after bind");
        assert!(is_server2, "cloned is_server should be true (shared Arc)");
        assert!(is_server3, "original is_server should still be true");
    }
}