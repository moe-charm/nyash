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
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 2,
                        actual: arguments.len(),
                    });
                }
                
                let address = self.execute_expression(&arguments[0])?;
                let port = self.execute_expression(&arguments[1])?;
                Ok(socket_box.bind(address, port))
            }
            "listen" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                
                let backlog = self.execute_expression(&arguments[0])?;
                Ok(socket_box.listen(backlog))
            }
            "accept" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(socket_box.accept())
            }
            "connect" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 2,
                        actual: arguments.len(),
                    });
                }
                
                let address = self.execute_expression(&arguments[0])?;
                let port = self.execute_expression(&arguments[1])?;
                Ok(socket_box.connect(address, port))
            }
            "read" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(socket_box.read())
            }
            "readHttpRequest" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(socket_box.read_http_request())
            }
            "write" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                
                let data = self.execute_expression(&arguments[0])?;
                Ok(socket_box.write(data))
            }
            "close" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(socket_box.close())
            }
            "isConnected" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(socket_box.is_connected())
            }
            "isServer" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(socket_box.is_server())
            }
            _ => Err(RuntimeError::UndefinedMethod {
                method: method.to_string(),
                object_type: "SocketBox".to_string(),
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
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 2,
                        actual: arguments.len(),
                    });
                }
                
                let address = self.execute_expression(&arguments[0])?;
                let port = self.execute_expression(&arguments[1])?;
                Ok(server_box.bind(address, port))
            }
            "listen" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                
                let backlog = self.execute_expression(&arguments[0])?;
                Ok(server_box.listen(backlog))
            }
            "start" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(server_box.start())
            }
            "stop" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(server_box.stop())
            }
            "get" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 2,
                        actual: arguments.len(),
                    });
                }
                
                let path = self.execute_expression(&arguments[0])?;
                let handler = self.execute_expression(&arguments[1])?;
                Ok(server_box.get(path, handler))
            }
            "post" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 2,
                        actual: arguments.len(),
                    });
                }
                
                let path = self.execute_expression(&arguments[0])?;
                let handler = self.execute_expression(&arguments[1])?;
                Ok(server_box.post(path, handler))
            }
            "put" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 2,
                        actual: arguments.len(),
                    });
                }
                
                let path = self.execute_expression(&arguments[0])?;
                let handler = self.execute_expression(&arguments[1])?;
                Ok(server_box.put(path, handler))
            }
            "delete" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 2,
                        actual: arguments.len(),
                    });
                }
                
                let path = self.execute_expression(&arguments[0])?;
                let handler = self.execute_expression(&arguments[1])?;
                Ok(server_box.delete(path, handler))
            }
            "route" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 2,
                        actual: arguments.len(),
                    });
                }
                
                let path = self.execute_expression(&arguments[0])?;
                let handler = self.execute_expression(&arguments[1])?;
                Ok(server_box.route(path, handler))
            }
            "setStaticPath" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                
                let path = self.execute_expression(&arguments[0])?;
                Ok(server_box.set_static_path(path))
            }
            "setTimeout" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                
                let timeout = self.execute_expression(&arguments[0])?;
                Ok(server_box.set_timeout(timeout))
            }
            "getActiveConnections" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(server_box.get_active_connections())
            }
            "isRunning" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(server_box.is_running())
            }
            _ => Err(RuntimeError::UndefinedMethod {
                method: method.to_string(),
                object_type: "HTTPServerBox".to_string(),
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
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(request_box.get_method())
            }
            "getPath" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(request_box.get_path())
            }
            "getQueryString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(request_box.get_query_string())
            }
            "getHeader" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                
                let header_name = self.execute_expression(&arguments[0])?;
                Ok(request_box.get_header(header_name))
            }
            "getAllHeaders" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(request_box.get_all_headers())
            }
            "hasHeader" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                
                let header_name = self.execute_expression(&arguments[0])?;
                Ok(request_box.has_header(header_name))
            }
            "getBody" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(request_box.get_body())
            }
            "getContentType" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(request_box.get_content_type())
            }
            "getContentLength" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(request_box.get_content_length())
            }
            _ => Err(RuntimeError::UndefinedMethod {
                method: method.to_string(),
                object_type: "HTTPRequestBox".to_string(),
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
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 2,
                        actual: arguments.len(),
                    });
                }
                
                let code = self.execute_expression(&arguments[0])?;
                let message = self.execute_expression(&arguments[1])?;
                Ok(response_box.set_status(code, message))
            }
            "setHeader" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 2,
                        actual: arguments.len(),
                    });
                }
                
                let name = self.execute_expression(&arguments[0])?;
                let value = self.execute_expression(&arguments[1])?;
                Ok(response_box.set_header(name, value))
            }
            "setContentType" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                
                let content_type = self.execute_expression(&arguments[0])?;
                Ok(response_box.set_content_type(content_type))
            }
            "setBody" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                
                let content = self.execute_expression(&arguments[0])?;
                Ok(response_box.set_body(content))
            }
            "appendBody" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                
                let content = self.execute_expression(&arguments[0])?;
                Ok(response_box.append_body(content))
            }
            "toHttpString" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::WrongNumberOfArguments {
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                
                Ok(response_box.to_http_string())
            }
            _ => Err(RuntimeError::UndefinedMethod {
                method: method.to_string(),
                object_type: "HTTPResponseBox".to_string(),
            }),
        }
    }
}