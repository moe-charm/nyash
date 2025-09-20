//! Nyash Net Plugin (HTTP minimal) — TypeBox v2
//! Provides ServerBox/RequestBox/ResponseBox/ClientBox and socket variants.
//! Pure in-process HTTP over localhost for E2E of BoxRef args/returns.

use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::io::Write as IoWrite;
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use crate::state::{ClientState, RequestState, ResponseState, ServerState, SockConnState};

// ===== Simple logger (enabled when NYASH_NET_LOG=1) =====
static LOG_ON: Lazy<bool> = Lazy::new(|| std::env::var("NYASH_NET_LOG").unwrap_or_default() == "1");
static LOG_PATH: Lazy<String> = Lazy::new(|| {
    std::env::var("NYASH_NET_LOG_FILE").unwrap_or_else(|_| "net_plugin.log".to_string())
});
static LOG_MTX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn net_log(msg: &str) {
    if !*LOG_ON {
        return;
    }
    // Always mirror to stderr for visibility
    eprintln!("[net] {}", msg);
    let _g = LOG_MTX.lock().unwrap();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&*LOG_PATH)
    {
        let _ = writeln!(f, "[{:?}] {}", std::time::SystemTime::now(), msg);
    }
}

macro_rules! netlog {
    ($($arg:tt)*) => {{ let s = format!($($arg)*); net_log(&s); }}
}

// Constants moved to a dedicated module for readability
mod consts;
use consts::*;

// Global State
// moved to state.rs

// State structs moved to state.rs

// legacy v1 abi/init removed

/* legacy v1 entry removed
#[no_mangle]
pub extern "C" fn nyash_plugin_invoke(
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match type_id {
            T_SERVER => server_invoke(method_id, instance_id, args, args_len, result, result_len),
            T_REQUEST => request_invoke(method_id, instance_id, args, args_len, result, result_len),
            T_RESPONSE => {
                response_invoke(method_id, instance_id, args, args_len, result, result_len)
            }
            T_CLIENT => client_invoke(method_id, instance_id, args, args_len, result, result_len),
            T_SOCK_SERVER => {
                sock_server_invoke(method_id, instance_id, args, args_len, result, result_len)
            }
            T_SOCK_CLIENT => {
                sock_client_invoke(method_id, instance_id, args, args_len, result, result_len)
            }
            T_SOCK_CONN => {
                sock_conn_invoke(method_id, instance_id, args, args_len, result, result_len)
            }
            _ => E_INV_TYPE,
        }
    }
}
*/

// ===== TypeBox ABI v2 (per-Box resolve/invoke_id) =====
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,     // 'TYBX'
    pub version: u16,     // 1
    pub struct_size: u16, // sizeof(NyashTypeBoxFfi)
    pub name: *const std::os::raw::c_char,
    pub resolve: Option<extern "C" fn(*const std::os::raw::c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}
unsafe impl Sync for NyashTypeBoxFfi {}

mod ffi;
extern "C" fn responsebox_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = ffi::cstr_to_string(name);
    match s.as_ref() {
        "setStatus" => M_RESP_SET_STATUS,
        "setHeader" => M_RESP_SET_HEADER,
        "write" => M_RESP_WRITE,
        "readBody" => M_RESP_READ_BODY,
        "getStatus" => M_RESP_GET_STATUS,
        "getHeader" => M_RESP_GET_HEADER,
        "birth" => M_BIRTH,
        "fini" => u32::MAX,
        _ => 0,
    }
}
extern "C" fn clientbox_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = ffi::cstr_to_string(name);
    match s.as_ref() {
        "get" => M_CLIENT_GET,
        "post" => M_CLIENT_POST,
        "birth" => M_BIRTH,
        "fini" => u32::MAX,
        _ => 0,
    }
}

extern "C" fn responsebox_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe { response_invoke(method_id, instance_id, args, args_len, result, result_len) }
}

extern "C" fn clientbox_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe { client_invoke(method_id, instance_id, args, args_len, result, result_len) }
}

#[no_mangle]
pub static nyash_typebox_ResponseBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"ResponseBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(responsebox_resolve),
    invoke_id: Some(responsebox_invoke_id),
    capabilities: 0,
};

#[no_mangle]
pub static nyash_typebox_ClientBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"ClientBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(clientbox_resolve),
    invoke_id: Some(clientbox_invoke_id),
    capabilities: 0,
};

// --- ServerBox ---
extern "C" fn serverbox_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = ffi::cstr_to_string(name);
    match s.as_ref() {
        "start" => M_SERVER_START,
        "stop" => M_SERVER_STOP,
        "accept" => M_SERVER_ACCEPT,
        "birth" => M_BIRTH,
        "fini" => u32::MAX,
        _ => 0,
    }
}
extern "C" fn serverbox_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe { server_invoke(method_id, instance_id, args, args_len, result, result_len) }
}
#[no_mangle]
pub static nyash_typebox_ServerBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"ServerBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(serverbox_resolve),
    invoke_id: Some(serverbox_invoke_id),
    capabilities: 0,
};

// --- SockServerBox ---
extern "C" fn sockserver_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = ffi::cstr_to_string(name);
    match s.as_ref() {
        "start" => M_SRV_START,
        "stop" => M_SRV_STOP,
        "accept" => M_SRV_ACCEPT,
        "acceptTimeout" => M_SRV_ACCEPT_TIMEOUT,
        "birth" => M_SRV_BIRTH,
        "fini" => u32::MAX,
        _ => 0,
    }
}
extern "C" fn sockserver_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe { sockets::sock_server_invoke(method_id, instance_id, args, args_len, result, result_len) }
}
#[no_mangle]
pub static nyash_typebox_SockServerBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"SockServerBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(sockserver_resolve),
    invoke_id: Some(sockserver_invoke_id),
    capabilities: 0,
};

// --- SockClientBox ---
extern "C" fn sockclient_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = ffi::cstr_to_string(name);
    match s.as_ref() {
        "connect" => M_SC_CONNECT,
        "birth" => M_SC_BIRTH,
        "fini" => u32::MAX,
        _ => 0,
    }
}
extern "C" fn sockclient_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe { sockets::sock_client_invoke(method_id, instance_id, args, args_len, result, result_len) }
}
#[no_mangle]
pub static nyash_typebox_SockClientBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"SockClientBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(sockclient_resolve),
    invoke_id: Some(sockclient_invoke_id),
    capabilities: 0,
};

// --- SockConnBox ---
extern "C" fn sockconn_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = ffi::cstr_to_string(name);
    match s.as_ref() {
        "send" => M_CONN_SEND,
        "recv" => M_CONN_RECV,
        "close" => M_CONN_CLOSE,
        "recvTimeout" => M_CONN_RECV_TIMEOUT,
        "birth" => M_CONN_BIRTH,
        "fini" => u32::MAX,
        _ => 0,
    }
}
extern "C" fn sockconn_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe { sockets::sock_conn_invoke(method_id, instance_id, args, args_len, result, result_len) }
}
#[no_mangle]
pub static nyash_typebox_SockConnBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"SockConnBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(sockconn_resolve),
    invoke_id: Some(sockconn_invoke_id),
    capabilities: 0,
};
extern "C" fn requestbox_resolve(name: *const std::os::raw::c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    let s = ffi::cstr_to_string(name);
    match s.as_ref() {
        "path" => M_REQ_PATH,
        "readBody" => M_REQ_READ_BODY,
        "respond" => M_REQ_RESPOND,
        "birth" => M_BIRTH,
        "fini" => u32::MAX,
        _ => 0,
    }
}
extern "C" fn requestbox_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe { request_invoke(method_id, instance_id, args, args_len, result, result_len) }
}

#[no_mangle]
pub static nyash_typebox_RequestBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"RequestBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(requestbox_resolve),
    invoke_id: Some(requestbox_invoke_id),
    capabilities: 0,
};

unsafe fn server_invoke(
    m: u32,
    id: u32,
    args: *const u8,
    args_len: usize,
    res: *mut u8,
    res_len: *mut usize,
) -> i32 {
    match m {
        M_BIRTH => {
            let id = state::next_server_id();
            state::SERVER_INSTANCES.lock().unwrap().insert(
                id,
                ServerState {
                    running: Arc::new(AtomicBool::new(false)),
                    port: 0,
                    pending: Arc::new(Mutex::new(VecDeque::new())),
                    handle: Mutex::new(None),
                    start_seq: 0,
                },
            );
            tlv::write_u32(id, res, res_len)
        }
        M_SERVER_START => {
            // args: TLV string/int (port)
            let port = tlv::tlv_parse_i32(slice(args, args_len)).unwrap_or(0);
            if let Some(s) = state::SERVER_INSTANCES.lock().unwrap().get_mut(&id) {
                s.port = port;
                s.start_seq = state::next_server_start_seq();
                let running = s.running.clone();
                let pending = s.pending.clone();
                running.store(true, Ordering::SeqCst);
                // Bind listener synchronously to avoid race with client connect
                let addr = format!("127.0.0.1:{}", port);
                let listener = match TcpListener::bind(&addr) {
                    Ok(l) => {
                        netlog!("http:listener bound {}", addr);
                        l
                    }
                    Err(e) => {
                        netlog!("http:bind error {} err={:?}", addr, e);
                        running.store(false, Ordering::SeqCst);
                        return tlv::write_tlv_void(res, res_len);
                    }
                };
                // Spawn HTTP listener thread (real TCP)
                let handle = std::thread::spawn(move || {
                    let _ = listener.set_nonblocking(true);
                    loop {
                        if !running.load(Ordering::SeqCst) {
                            break;
                        }
                        match listener.accept() {
                            Ok((mut stream, _)) => {
                                // Parse minimal HTTP request (GET/POST)
                                let _ = stream.set_read_timeout(Some(Duration::from_millis(2000)));
                                if let Some((path, body, resp_hint)) =
                                    http_helpers::read_http_request(&mut stream)
                                {
                                    // Store stream for later respond()
                                    let conn_id = state::next_sock_conn_id();
                                    state::SOCK_CONNS.lock().unwrap().insert(
                                        conn_id,
                                        SockConnState {
                                            stream: Mutex::new(stream),
                                        },
                                    );

                                    let req_id = state::next_request_id();
                                    state::REQUESTS.lock().unwrap().insert(
                                        req_id,
                                        RequestState {
                                            path,
                                            body,
                                            response_id: resp_hint,
                                            server_conn_id: Some(conn_id),
                                            responded: false,
                                        },
                                    );
                                    if let Some(h) = resp_hint {
                                        netlog!("http:accept linked resp_id hint={} for req_id={} conn_id={}", h, req_id, conn_id);
                                    }
                                    pending.lock().unwrap().push_back(req_id);
                                } else {
                                    // Malformed; drop connection
                                }
                            }
                            Err(_) => {
                                std::thread::sleep(Duration::from_millis(10));
                            }
                        }
                    }
                });
                *s.handle.lock().unwrap() = Some(handle);
            }
            // mark active server
            *state::ACTIVE_SERVER_ID.lock().unwrap() = Some(id);
            tlv::write_tlv_void(res, res_len)
        }
        M_SERVER_STOP => {
            if let Some(s) = state::SERVER_INSTANCES.lock().unwrap().get_mut(&id) {
                s.running.store(false, Ordering::SeqCst);
                if let Some(h) = s.handle.lock().unwrap().take() {
                    let _ = h.join();
                }
            }
            // clear active if this server was active
            let mut active = state::ACTIVE_SERVER_ID.lock().unwrap();
            if active.map(|v| v == id).unwrap_or(false) {
                *active = None;
            }
            tlv::write_tlv_void(res, res_len)
        }
        M_SERVER_ACCEPT => {
            // wait up to ~5000ms for a request to arrive
            for _ in 0..1000 {
                // Prefer TCP-backed requests (server_conn_id=Some) over stub ones
                if let Some(req_id) = {
                    let mut map = state::SERVER_INSTANCES.lock().unwrap();
                    if let Some(s) = map.get_mut(&id) {
                        let mut q = s.pending.lock().unwrap();
                        // Find first index with TCP backing
                        let mut chosen: Option<usize> = None;
                        for i in 0..q.len() {
                            if let Some(rid) = q.get(i).copied() {
                                if let Some(rq) = state::REQUESTS.lock().unwrap().get(&rid) {
                                    if rq.server_conn_id.is_some() {
                                        chosen = Some(i);
                                        break;
                                    }
                                }
                            }
                        }
                        if let Some(idx) = chosen {
                            q.remove(idx)
                        } else {
                            q.pop_front()
                        }
                    } else {
                        None
                    }
                } {
                    netlog!("server.accept: return req_id={} srv_id={}", req_id, id);
                    *state::LAST_ACCEPTED_REQ.lock().unwrap() = Some(req_id);
                    return tlv::write_tlv_handle(T_REQUEST, req_id, res, res_len);
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            tlv::write_tlv_void(res, res_len)
        }
        _ => E_INV_METHOD,
    }
}

unsafe fn request_invoke(
    m: u32,
    id: u32,
    _args: *const u8,
    _args_len: usize,
    res: *mut u8,
    res_len: *mut usize,
) -> i32 {
    match m {
        M_BIRTH => {
            let id = state::next_request_id();
            state::REQUESTS.lock().unwrap().insert(
                id,
                RequestState {
                    path: String::new(),
                    body: vec![],
                    response_id: None,
                    server_conn_id: None,
                    responded: false,
                },
            );
            tlv::write_u32(id, res, res_len)
        }
        M_REQ_PATH => {
            if let Some(rq) = state::REQUESTS.lock().unwrap().get(&id) {
                tlv::write_tlv_string(&rq.path, res, res_len)
            } else {
                E_INV_HANDLE
            }
        }
        M_REQ_READ_BODY => {
            if let Some(rq) = state::REQUESTS.lock().unwrap().get(&id) {
                tlv::write_tlv_bytes(&rq.body, res, res_len)
            } else {
                E_INV_HANDLE
            }
        }
        M_REQ_RESPOND => {
            // args: TLV Handle(Response)
            let (t, provided_resp_id) = tlv::tlv_parse_handle(slice(_args, _args_len))
                .map_err(|_| ())
                .or(Err(()))
                .unwrap_or((0, 0));
            if t != T_RESPONSE {
                return E_INV_ARGS;
            }
            // Acquire request
            let mut rq_map = state::REQUESTS.lock().unwrap();
            if let Some(rq) = rq_map.get_mut(&id) {
                netlog!(
                    "Request.respond: req_id={} provided_resp_id={} server_conn_id={:?} response_id_hint={:?}",
                    id, provided_resp_id, rq.server_conn_id, rq.response_id
                );
                // If request is backed by a real socket, write HTTP over that socket
                if let Some(conn_id) = rq.server_conn_id {
                    drop(rq_map);
                    // Read response content from provided response handle
                    let (status, headers, body) = {
                        let resp_map = state::RESPONSES.lock().unwrap();
                        if let Some(src) = resp_map.get(&provided_resp_id) {
                            netlog!(
                                "Request.respond: Reading response id={}, status={}, body_len={}",
                                provided_resp_id,
                                src.status,
                                src.body.len()
                            );
                            (src.status, src.headers.clone(), src.body.clone())
                        } else {
                            netlog!(
                                "Request.respond: Response id={} not found!",
                                provided_resp_id
                            );
                            return E_INV_HANDLE;
                        }
                    };
                    // Build minimal HTTP/1.1 response
                    let reason = match status {
                        200 => "OK",
                        201 => "Created",
                        204 => "No Content",
                        400 => "Bad Request",
                        404 => "Not Found",
                        500 => "Internal Server Error",
                        _ => "OK",
                    };
                    let mut buf = Vec::new();
                    buf.extend_from_slice(format!("HTTP/1.1 {} {}\r\n", status, reason).as_bytes());
                    let mut has_len = false;
                    for (k, v) in &headers {
                        if k.eq_ignore_ascii_case("Content-Length") {
                            has_len = true;
                        }
                        buf.extend_from_slice(format!("{}: {}\r\n", k, v).as_bytes());
                    }
                    if !has_len {
                        buf.extend_from_slice(
                            format!("Content-Length: {}\r\n", body.len()).as_bytes(),
                        );
                    }
                    buf.extend_from_slice(b"Connection: close\r\n");
                    buf.extend_from_slice(b"\r\n");
                    buf.extend_from_slice(&body);
                    // Write and close
                    netlog!(
                        "Request.respond: Sending HTTP response, buf_len={}",
                        buf.len()
                    );
                    if let Some(conn) = state::SOCK_CONNS.lock().unwrap().remove(&conn_id) {
                        if let Ok(mut s) = conn.stream.lock() {
                            let _ = s.write_all(&buf);
                            let _ = s.flush();
                            netlog!(
                                "Request.respond: HTTP response sent to socket conn_id={}",
                                conn_id
                            );
                        }
                    } else {
                        netlog!("Request.respond: Socket conn_id={} not found!", conn_id);
                    }
                    // Also mirror to paired client Response handle to avoid race on immediate read
                    if let Some(target_id) = {
                        let rq_map2 = state::REQUESTS.lock().unwrap();
                        rq_map2.get(&id).and_then(|rq2| rq2.response_id)
                    } {
                        let mut resp_map = state::RESPONSES.lock().unwrap();
                        let dst = resp_map.entry(target_id).or_insert(ResponseState {
                            status: 200,
                            headers: HashMap::new(),
                            body: vec![],
                            client_conn_id: None,
                            parsed: true,
                        });
                        dst.status = status;
                        dst.headers = headers.clone();
                        dst.body = body.clone();
                        netlog!("Request.respond: mirrored client handle id={} body_len={} headers={} status={}", target_id, dst.body.len(), dst.headers.len(), dst.status);
                    }
                    // mark responded
                    {
                        let mut rq_map3 = state::REQUESTS.lock().unwrap();
                        if let Some(rq3) = rq_map3.get_mut(&id) {
                            rq3.responded = true;
                        }
                    }
                    return tlv::write_tlv_void(res, res_len);
                }

                // Not backed by a socket: attempt reroute to last accepted or latest TCP-backed unresponded request
                drop(rq_map);
                let candidate_req = {
                    if let Some(last_id) = *state::LAST_ACCEPTED_REQ.lock().unwrap() {
                        if let Some(r) = state::REQUESTS.lock().unwrap().get(&last_id) {
                            if r.server_conn_id.is_some() && !r.responded {
                                Some(last_id)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                .or_else(|| {
                    state::REQUESTS
                        .lock()
                        .unwrap()
                        .iter()
                        .filter_map(|(rid, rqs)| {
                            if rqs.server_conn_id.is_some() && !rqs.responded {
                                Some(*rid)
                            } else {
                                None
                            }
                        })
                        .max()
                });
                if let Some(target_req_id) = candidate_req {
                    let (conn_id_alt, resp_hint_alt) = {
                        let map = state::REQUESTS.lock().unwrap();
                        let r = map.get(&target_req_id).unwrap();
                        (r.server_conn_id.unwrap(), r.response_id)
                    };
                    let (status, headers, body) = {
                        let resp_map = state::RESPONSES.lock().unwrap();
                        if let Some(src) = resp_map.get(&provided_resp_id) {
                            (src.status, src.headers.clone(), src.body.clone())
                        } else {
                            return E_INV_HANDLE;
                        }
                    };
                    let reason = match status {
                        200 => "OK",
                        201 => "Created",
                        204 => "No Content",
                        400 => "Bad Request",
                        404 => "Not Found",
                        500 => "Internal Server Error",
                        _ => "OK",
                    };
                    let mut buf = Vec::new();
                    buf.extend_from_slice(format!("HTTP/1.1 {} {}\r\n", status, reason).as_bytes());
                    let mut has_len = false;
                    for (k, v) in &headers {
                        if k.eq_ignore_ascii_case("Content-Length") {
                            has_len = true;
                        }
                        buf.extend_from_slice(format!("{}: {}\r\n", k, v).as_bytes());
                    }
                    if !has_len {
                        buf.extend_from_slice(
                            format!("Content-Length: {}\r\n", body.len()).as_bytes(),
                        );
                    }
                    buf.extend_from_slice(b"Connection: close\r\n\r\n");
                    buf.extend_from_slice(&body);
                    netlog!(
                        "Request.respond: reroute TCP send via req_id={} conn_id={}",
                        target_req_id,
                        conn_id_alt
                    );
                    if let Some(conn) = state::SOCK_CONNS.lock().unwrap().remove(&conn_id_alt) {
                        if let Ok(mut s) = conn.stream.lock() {
                            let _ = s.write_all(&buf);
                            let _ = s.flush();
                        }
                    }
                    if let Some(target_id) = resp_hint_alt {
                        let mut resp_map = state::RESPONSES.lock().unwrap();
                        let dst = resp_map.entry(target_id).or_insert(ResponseState {
                            status: 200,
                            headers: HashMap::new(),
                            body: vec![],
                            client_conn_id: None,
                            parsed: true,
                        });
                        dst.status = status;
                        dst.headers = headers.clone();
                        dst.body = body.clone();
                        netlog!("Request.respond: mirrored client handle id={} body_len={} headers={} status={}", target_id, dst.body.len(), dst.headers.len(), dst.status);
                    }
                    if let Some(rq4) = state::REQUESTS.lock().unwrap().get_mut(&target_req_id) {
                        rq4.responded = true;
                    }
                    return tlv::write_tlv_void(res, res_len);
                }
                netlog!("Request.respond: no suitable TCP-backed request found for reroute; invalid handle");
                return E_INV_HANDLE;
            }
            E_INV_HANDLE
        }
        _ => E_INV_METHOD,
    }
}

unsafe fn response_invoke(
    m: u32,
    id: u32,
    args: *const u8,
    args_len: usize,
    res: *mut u8,
    res_len: *mut usize,
) -> i32 {
    match m {
        M_BIRTH => {
            let id = state::next_response_id();
            state::RESPONSES.lock().unwrap().insert(
                id,
                ResponseState {
                    status: 200,
                    headers: HashMap::new(),
                    body: vec![],
                    client_conn_id: None,
                    parsed: false,
                },
            );
            netlog!("Response.birth: new id={}", id);
            tlv::write_u32(id, res, res_len)
        }
        M_RESP_SET_STATUS => {
            let code = tlv::tlv_parse_i32(slice(args, args_len)).unwrap_or(200);
            if let Some(rp) = state::RESPONSES.lock().unwrap().get_mut(&id) {
                rp.status = code;
            }
            tlv::write_tlv_void(res, res_len)
        }
        M_RESP_SET_HEADER => {
            if let Ok((name, value)) = tlv::tlv_parse_two_strings(slice(args, args_len)) {
                if let Some(rp) = state::RESPONSES.lock().unwrap().get_mut(&id) {
                    rp.headers.insert(name, value);
                }
                return tlv::write_tlv_void(res, res_len);
            }
            E_INV_ARGS
        }
        M_RESP_WRITE => {
            // Accept String or Bytes
            let bytes = tlv::tlv_parse_bytes(slice(args, args_len)).unwrap_or_default();
            netlog!("HttpResponse.write: id={} bytes_len={}", id, bytes.len());
            if let Some(rp) = state::RESPONSES.lock().unwrap().get_mut(&id) {
                rp.body.extend_from_slice(&bytes);
                netlog!("HttpResponse.write: body now has {} bytes", rp.body.len());
            }
            tlv::write_tlv_void(res, res_len)
        }
        M_RESP_READ_BODY => {
            netlog!("HttpResponse.readBody: enter id={}", id);
            // If bound to a client connection, lazily read and parse (with short retries)
            for _ in 0..50 {
                let need_parse = {
                    if let Some(rp) = state::RESPONSES.lock().unwrap().get(&id) {
                        rp.client_conn_id
                    } else {
                        return E_INV_HANDLE;
                    }
                };
                if let Some(conn_id) = need_parse {
                    http_helpers::parse_client_response_into(id, conn_id);
                    std::thread::sleep(Duration::from_millis(5));
                } else {
                    break;
                }
            }
            if let Some(rp) = state::RESPONSES.lock().unwrap().get(&id) {
                netlog!(
                    "HttpResponse.readBody: id={} body_len={}",
                    id,
                    rp.body.len()
                );
                tlv::write_tlv_bytes(&rp.body, res, res_len)
            } else {
                E_INV_HANDLE
            }
        }
        M_RESP_GET_STATUS => {
            for _ in 0..50 {
                let need_parse = {
                    if let Some(rp) = state::RESPONSES.lock().unwrap().get(&id) {
                        rp.client_conn_id
                    } else {
                        return E_INV_HANDLE;
                    }
                };
                if let Some(conn_id) = need_parse {
                    http_helpers::parse_client_response_into(id, conn_id);
                    std::thread::sleep(Duration::from_millis(5));
                } else {
                    break;
                }
            }
            if let Some(rp) = state::RESPONSES.lock().unwrap().get(&id) {
                tlv::write_tlv_i32(rp.status, res, res_len)
            } else {
                E_INV_HANDLE
            }
        }
        M_RESP_GET_HEADER => {
            if let Ok(name) = tlv::tlv_parse_string(slice(args, args_len)) {
                for _ in 0..50 {
                    let need_parse = {
                        if let Some(rp) = state::RESPONSES.lock().unwrap().get(&id) {
                            rp.client_conn_id
                        } else {
                            return E_INV_HANDLE;
                        }
                    };
                    if let Some(conn_id) = need_parse {
                        http_helpers::parse_client_response_into(id, conn_id);
                        std::thread::sleep(Duration::from_millis(5));
                    } else {
                        break;
                    }
                }
                if let Some(rp) = state::RESPONSES.lock().unwrap().get(&id) {
                    let v = rp.headers.get(&name).cloned().unwrap_or_default();
                    return tlv::write_tlv_string(&v, res, res_len);
                } else {
                    return E_INV_HANDLE;
                }
            }
            E_INV_ARGS
        }
        _ => E_INV_METHOD,
    }
}

unsafe fn client_invoke(
    m: u32,
    _id: u32,
    args: *const u8,
    args_len: usize,
    res: *mut u8,
    res_len: *mut usize,
) -> i32 {
    match m {
        M_BIRTH => {
            let id = state::next_client_id();
            state::CLIENTS.lock().unwrap().insert(id, ClientState);
            tlv::write_u32(id, res, res_len)
        }
        M_CLIENT_GET => {
            // args: TLV String(url)
            let url = tlv::tlv_parse_string(slice(args, args_len)).unwrap_or_default();
            let port = http_helpers::parse_port(&url).unwrap_or(80);
            let host = http_helpers::parse_host(&url).unwrap_or_else(|| "127.0.0.1".to_string());
            let path = http_helpers::parse_path(&url);
            // Create client response handle first, so we can include it in header
            let resp_id = state::next_response_id();
            let (_h, _p, req_bytes) = http_helpers::build_http_request("GET", &url, None, resp_id);
            // Try TCP connect (best effort)
            let mut tcp_ok = false;
            if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", host, port)) {
                let _ = stream.write_all(&req_bytes);
                let _ = stream.flush();
                let conn_id = state::next_sock_conn_id();
                state::SOCK_CONNS.lock().unwrap().insert(
                    conn_id,
                    SockConnState {
                        stream: Mutex::new(stream),
                    },
                );
                // Map to server_id by port if available (not used; reserved)
                state::RESPONSES.lock().unwrap().insert(
                    resp_id,
                    ResponseState {
                        status: 0,
                        headers: HashMap::new(),
                        body: vec![],
                        client_conn_id: Some(conn_id),
                        parsed: false,
                    },
                );
                tcp_ok = true;
                netlog!(
                    "client.get: url={} resp_id={} tcp_ok=true conn_id={}",
                    url,
                    resp_id,
                    conn_id
                );
            } else {
                // Map to server_id by port if available (not used; reserved)
                state::RESPONSES.lock().unwrap().insert(
                    resp_id,
                    ResponseState {
                        status: 0,
                        headers: HashMap::new(),
                        body: vec![],
                        client_conn_id: None,
                        parsed: false,
                    },
                );
                netlog!("client.get: url={} resp_id={} tcp_ok=false", url, resp_id);
            }
            // No stub enqueue in TCP-only design
            if tcp_ok {
                tlv::write_tlv_handle(T_RESPONSE, resp_id, res, res_len)
            } else {
                // Encode error string; loader interprets returns_result=true methods' string payload as Err
                let msg = format!(
                    "connect failed for {}:{}{}",
                    host,
                    port,
                    if path.is_empty() { "" } else { &path }
                );
                tlv::write_tlv_string(&msg, res, res_len)
            }
        }
        M_CLIENT_POST => {
            // args: TLV String(url), Bytes body
            let data = slice(args, args_len);
            let (_, argc, mut pos) = tlv::tlv_parse_header(data)
                .map_err(|_| ())
                .or(Err(()))
                .unwrap_or((1, 0, 4));
            if argc < 2 {
                return E_INV_ARGS;
            }
            let (_t1, s1, p1) = tlv::tlv_parse_entry_hdr(data, pos)
                .map_err(|_| ())
                .or(Err(()))
                .unwrap_or((0, 0, 0));
            if data[pos] != 6 {
                return E_INV_ARGS;
            }
            let url = std::str::from_utf8(&data[p1..p1 + s1])
                .map_err(|_| ())
                .or(Err(()))
                .unwrap_or("")
                .to_string();
            pos = p1 + s1;
            let (t2, s2, p2) = tlv::tlv_parse_entry_hdr(data, pos)
                .map_err(|_| ())
                .or(Err(()))
                .unwrap_or((0, 0, 0));
            if t2 != 6 && t2 != 7 {
                return E_INV_ARGS;
            }
            let body = data[p2..p2 + s2].to_vec();
            let port = http_helpers::parse_port(&url).unwrap_or(80);
            let host = http_helpers::parse_host(&url).unwrap_or_else(|| "127.0.0.1".to_string());
            let path = http_helpers::parse_path(&url);
            let body_len = body.len();
            // Create client response handle
            let resp_id = state::next_response_id();
            let (_h, _p, req_bytes) = http_helpers::build_http_request("POST", &url, Some(&body), resp_id);
            let mut tcp_ok = false;
            if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", host, port)) {
                let _ = stream.write_all(&req_bytes);
                let _ = stream.flush();
                let conn_id = state::next_sock_conn_id();
                state::SOCK_CONNS.lock().unwrap().insert(
                    conn_id,
                    SockConnState {
                        stream: Mutex::new(stream),
                    },
                );
                // Map to server_id by port if available (not used; reserved)
                state::RESPONSES.lock().unwrap().insert(
                    resp_id,
                    ResponseState {
                        status: 0,
                        headers: HashMap::new(),
                        body: vec![],
                        client_conn_id: Some(conn_id),
                        parsed: false,
                    },
                );
                tcp_ok = true;
                netlog!(
                    "client.post: url={} resp_id={} tcp_ok=true conn_id={} body_len={}",
                    url,
                    resp_id,
                    conn_id,
                    body.len()
                );
            } else {
                // Map to server_id by port if available (not used; reserved)
                state::RESPONSES.lock().unwrap().insert(
                    resp_id,
                    ResponseState {
                        status: 0,
                        headers: HashMap::new(),
                        body: vec![],
                        client_conn_id: None,
                        parsed: false,
                    },
                );
                netlog!(
                    "client.post: url={} resp_id={} tcp_ok=false body_len={}",
                    url,
                    resp_id,
                    body.len()
                );
            }
            // No stub enqueue in TCP-only design
            if tcp_ok {
                tlv::write_tlv_handle(T_RESPONSE, resp_id, res, res_len)
            } else {
                let msg = format!(
                    "connect failed for {}:{}{} (body_len={})",
                    host,
                    port,
                    if path.is_empty() { "" } else { &path },
                    body_len
                );
                tlv::write_tlv_string(&msg, res, res_len)
            }
        }
        _ => E_INV_METHOD,
    }
}

// helpers moved to http_helpers.rs

// moved

// ===== Helpers =====
use ffi::slice;
mod tlv;
mod http_helpers;
mod sockets;

// ===== HTTP helpers =====
// moved

// moved

// moved

// moved

// moved

// ===== Socket implementation =====
// moved to sockets.rs

mod state;
