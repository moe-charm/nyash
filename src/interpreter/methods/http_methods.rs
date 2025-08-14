/*! 🌐 HTTP Method Implementations
 * 
 * HTTP関連Boxのメソッド実行を実装
 * SocketBox, HTTPServerBox, HTTPRequestBox, HTTPResponseBox
 */

use super::super::*;
use crate::boxes::{SocketBox, HTTPServerBox, HTTPRequestBox, HTTPResponseBox};

impl NyashInterpreter {
    /// SocketBox methods
    pub(in crate::interpreter) fn execute_socket_method(
        &mut self, 
        socket_box: &SocketBox, 
        method: &str, 
        arguments: &[ASTNode]
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "bind" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("bind() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                
                let address = self.execute_expression(&arguments[0])?;
                let port = self.execute_expression(&arguments[1])?;
                let result = socket_box.bind(address, port);
                Ok(result)
            }
            "listen" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("listen() expects 1 argument, got {}", arguments.len()),
                    });
                }
                
                let backlog = self.execute_expression(&arguments[0])?;
                Ok(socket_box.listen(backlog))
            }
            "accept" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("accept() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(socket_box.accept())
            }
            "connect" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("connect() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                
                let address = self.execute_expression(&arguments[0])?;
                let port = self.execute_expression(&arguments[1])?;
                Ok(socket_box.connect(address, port))
            }
            "read" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("read() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(socket_box.read())
            }
            "readHttpRequest" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("readHttpRequest() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(socket_box.read_http_request())
            }
            "write" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("write() expects 1 argument, got {}", arguments.len()),
                    });
                }
                
                let data = self.execute_expression(&arguments[0])?;
                Ok(socket_box.write(data))
            }
            "close" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("close() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(socket_box.close())
            }
            "isConnected" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isConnected() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(socket_box.is_connected())
            }
            "isServer" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("isServer() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(socket_box.is_server())
            }
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(Box::new(socket_box.to_string_box()))
            }
            _ => Err(RuntimeError::UndefinedVariable {
                name: format!("SocketBox method '{}' not found", method),
            }),
        }
    }

    /// HTTPServerBox methods
    pub(in crate::interpreter) fn execute_http_server_method(
        &mut self, 
        server_box: &HTTPServerBox, 
        method: &str, 
        arguments: &[ASTNode]
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "bind" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("bind() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                
                let address = self.execute_expression(&arguments[0])?;
                let port = self.execute_expression(&arguments[1])?;
                Ok(server_box.bind(address, port))
            }
            "listen" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("listen() expects 1 argument, got {}", arguments.len()),
                    });
                }
                
                let backlog = self.execute_expression(&arguments[0])?;
                Ok(server_box.listen(backlog))
            }
            "start" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("start() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(server_box.start())
            }
            "stop" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("stop() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(server_box.stop())
            }
            "get" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                
                let path = self.execute_expression(&arguments[0])?;
                let handler = self.execute_expression(&arguments[1])?;
                Ok(server_box.get(path, handler))
            }
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(Box::new(server_box.to_string_box()))
            }
            _ => Err(RuntimeError::UndefinedVariable {
                name: format!("HTTPServerBox method '{}' not found", method),
            }),
        }
    }

    /// HTTPRequestBox methods  
    pub(in crate::interpreter) fn execute_http_request_method(
        &mut self, 
        request_box: &HTTPRequestBox, 
        method: &str, 
        arguments: &[ASTNode]
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "getMethod" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getMethod() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(request_box.get_method())
            }
            "getPath" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("getPath() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(request_box.get_path())
            }
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(Box::new(request_box.to_string_box()))
            }
            _ => Err(RuntimeError::UndefinedVariable {
                name: format!("HTTPRequestBox method '{}' not found", method),
            }),
        }
    }

    /// HTTPResponseBox methods
    pub(in crate::interpreter) fn execute_http_response_method(
        &mut self, 
        response_box: &HTTPResponseBox, 
        method: &str, 
        arguments: &[ASTNode]
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "setStatus" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("setStatus() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                
                let code = self.execute_expression(&arguments[0])?;
                let message = self.execute_expression(&arguments[1])?;
                Ok(response_box.set_status(code, message))
            }
            "toHttpString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toHttpString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(response_box.to_http_string())
            }
            "toString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("toString() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                
                Ok(Box::new(response_box.to_string_box()))
            }
            _ => Err(RuntimeError::UndefinedVariable {
                name: format!("HTTPResponseBox method '{}' not found", method),
            }),
        }
    }
}