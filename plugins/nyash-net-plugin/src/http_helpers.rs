use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

use crate::state;

pub fn parse_path(url: &str) -> String {
    if url.starts_with('/') {
        return url.to_string();
    }
    if let Some(scheme_pos) = url.find("//") {
        let after_scheme = &url[scheme_pos + 2..];
        if let Some(slash) = after_scheme.find('/') {
            return after_scheme[slash..].to_string();
        } else {
            return "/".to_string();
        }
    }
    "/".to_string()
}

pub fn parse_port(url: &str) -> Option<i32> {
    if let Some(pat) = url.split("//").nth(1) {
        if let Some(after_host) = pat.split('/').next() {
            if let Some(colon) = after_host.rfind(':') {
                return after_host[colon + 1..].parse::<i32>().ok();
            }
        }
    }
    None
}

pub fn parse_host(url: &str) -> Option<String> {
    if let Some(rest) = url.split("//").nth(1) {
        let host_port = rest.split('/').next().unwrap_or("");
        let host = host_port.split(':').next().unwrap_or("");
        if !host.is_empty() {
            return Some(host.to_string());
        }
    }
    None
}

pub fn build_http_request(
    method: &str,
    url: &str,
    body: Option<&[u8]>,
    resp_id: u32,
) -> (String, String, Vec<u8>) {
    let host = parse_host(url).unwrap_or_else(|| "127.0.0.1".to_string());
    let path = parse_path(url);
    let mut buf = Vec::new();
    buf.extend_from_slice(format!("{} {} HTTP/1.1\r\n", method, &path).as_bytes());
    buf.extend_from_slice(format!("Host: {}\r\n", host).as_bytes());
    buf.extend_from_slice(b"User-Agent: nyash-net-plugin/0.1\r\n");
    buf.extend_from_slice(format!("X-Nyash-Resp-Id: {}\r\n", resp_id).as_bytes());
    match body {
        Some(b) => {
            buf.extend_from_slice(format!("Content-Length: {}\r\n", b.len()).as_bytes());
            buf.extend_from_slice(b"Content-Type: application/octet-stream\r\n");
            buf.extend_from_slice(b"Connection: close\r\n\r\n");
            buf.extend_from_slice(b);
        }
        None => {
            buf.extend_from_slice(b"Connection: close\r\n\r\n");
        }
    }
    (host, path, buf)
}

pub fn read_http_request(stream: &mut TcpStream) -> Option<(String, Vec<u8>, Option<u32>)> {
    let mut buf = Vec::with_capacity(1024);
    let mut tmp = [0u8; 1024];
    let header_end;
    loop {
        match stream.read(&mut tmp) {
            Ok(0) => return None,
            Ok(n) => {
                buf.extend_from_slice(&tmp[..n]);
                if let Some(pos) = find_header_end(&buf) {
                    header_end = pos;
                    break;
                }
                if buf.len() > 64 * 1024 {
                    return None;
                }
            }
            Err(_) => return None,
        }
    }
    let header = &buf[..header_end];
    let after = &buf[header_end + 4..];
    let header_str = String::from_utf8_lossy(header);
    let mut lines = header_str.split("\r\n");
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/").to_string();
    let mut content_length: usize = 0;
    let mut resp_handle_id: Option<u32> = None;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            if k.eq_ignore_ascii_case("Content-Length") {
                content_length = v.trim().parse().unwrap_or(0);
            }
            if k.eq_ignore_ascii_case("X-Nyash-Resp-Id") {
                resp_handle_id = v.trim().parse::<u32>().ok();
            }
        }
    }
    let mut body = after.to_vec();
    while body.len() < content_length {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => body.extend_from_slice(&tmp[..n]),
            Err(_) => break,
        }
    }
    if method == "GET" || method == "POST" {
        Some((path, body, resp_handle_id))
    } else {
        None
    }
}

pub fn find_header_end(buf: &[u8]) -> Option<usize> {
    if buf.len() < 4 {
        return None;
    }
    for i in 0..=buf.len() - 4 {
        if &buf[i..i + 4] == b"\r\n\r\n" {
            return Some(i);
        }
    }
    None
}

pub fn parse_client_response_into(resp_id: u32, conn_id: u32) {
    let mut status: i32 = 200;
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut body: Vec<u8> = Vec::new();
    let mut should_remove = false;
    if let Ok(mut map) = state::SOCK_CONNS.lock() {
        if let Some(conn) = map.get(&conn_id) {
            if let Ok(mut s) = conn.stream.lock() {
                let _ = s.set_read_timeout(Some(Duration::from_millis(4000)));
                let mut buf = Vec::with_capacity(2048);
                let mut tmp = [0u8; 2048];
                loop {
                    match s.read(&mut tmp) {
                        Ok(0) => {
                            return;
                        }
                        Ok(n) => {
                            buf.extend_from_slice(&tmp[..n]);
                            if find_header_end(&buf).is_some() {
                                break;
                            }
                            if buf.len() > 256 * 1024 {
                                break;
                            }
                        }
                        Err(_) => return,
                    }
                }
                if let Some(pos) = find_header_end(&buf) {
                    let header = &buf[..pos];
                    let after = &buf[pos + 4..];
                    let header_str = String::from_utf8_lossy(header);
                    let mut lines = header_str.split("\r\n");
                    if let Some(status_line) = lines.next() {
                        let mut sp = status_line.split_whitespace();
                        let _ver = sp.next();
                        if let Some(code_str) = sp.next() {
                            status = code_str.parse::<i32>().unwrap_or(200);
                        }
                    }
                    for line in lines {
                        if let Some((k, v)) = line.split_once(':') {
                            headers.insert(k.trim().to_string(), v.trim().to_string());
                        }
                    }
                    body.extend_from_slice(after);
                    let need = headers
                        .get("Content-Length")
                        .and_then(|v| v.parse::<usize>().ok())
                        .unwrap_or(0);
                    while body.len() < need {
                        match s.read(&mut tmp) {
                            Ok(0) => break,
                            Ok(n) => body.extend_from_slice(&tmp[..n]),
                            Err(_) => break,
                        }
                    }
                    should_remove = true;
                }
            }
        }
        if should_remove {
            map.remove(&conn_id);
        }
    }
    if let Some(rp) = state::RESPONSES.lock().unwrap().get_mut(&resp_id) {
        rp.status = status;
        rp.headers = headers;
        rp.body = body;
        rp.parsed = true;
        rp.client_conn_id = None;
    }
}
