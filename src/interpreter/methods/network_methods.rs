/*!
 * Network and Communication Box Methods Module
 * 
 * Contains method implementations for network-related Box types:
 * - HttpClientBox (execute_http_method) - HTTP client operations
 * - StreamBox (execute_stream_method) - Stream processing operations
 */

use super::super::*;
use crate::box_trait::{NyashBox, StringBox};
use crate::boxes::{HttpClientBox, StreamBox};

impl NyashInterpreter {
    /// HttpClientBoxのメソッド呼び出しを実行
    pub(in crate::interpreter) fn execute_http_method(&mut self, http_box: &HttpClientBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "get" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("get() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let url = self.execute_expression(&arguments[0])?;
                Ok(http_box.http_get(url))
            }
            "post" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("post() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let url = self.execute_expression(&arguments[0])?;
                let body = self.execute_expression(&arguments[1])?;
                Ok(http_box.post(url, body))
            }
            "put" => {
                if arguments.len() != 2 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("put() expects 2 arguments, got {}", arguments.len()),
                    });
                }
                let url = self.execute_expression(&arguments[0])?;
                let body = self.execute_expression(&arguments[1])?;
                Ok(http_box.put(url, body))
            }
            "delete" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("delete() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let url = self.execute_expression(&arguments[0])?;
                Ok(http_box.delete(url))
            }
            "request" => {
                if arguments.len() != 3 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("request() expects 3 arguments, got {}", arguments.len()),
                    });
                }
                let method_arg = self.execute_expression(&arguments[0])?;
                let url = self.execute_expression(&arguments[1])?;
                let options = self.execute_expression(&arguments[2])?;
                Ok(http_box.request(method_arg, url, options))
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for HttpClientBox", method),
            })
        }
    }

    /// StreamBoxのメソッド呼び出しを実行
    pub(in crate::interpreter) fn execute_stream_method(&mut self, stream_box: &StreamBox, method: &str, arguments: &[ASTNode]) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match method {
            "write" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("write() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let data = self.execute_expression(&arguments[0])?;
                Ok(stream_box.stream_write(data))
            }
            "read" => {
                if arguments.len() != 1 {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("read() expects 1 argument, got {}", arguments.len()),
                    });
                }
                let count = self.execute_expression(&arguments[0])?;
                Ok(stream_box.stream_read(count))
            }
            "position" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("position() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(stream_box.get_position())
            }
            "length" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("length() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(stream_box.get_length())
            }
            "reset" => {
                if !arguments.is_empty() {
                    return Err(RuntimeError::InvalidOperation {
                        message: format!("reset() expects 0 arguments, got {}", arguments.len()),
                    });
                }
                Ok(stream_box.stream_reset())
            }
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown method '{}' for StreamBox", method),
            })
        }
    }
}