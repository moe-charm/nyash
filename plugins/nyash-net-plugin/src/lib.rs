//! Nyash Net Plugin (HTTP stub) - BID-FFI v1
//! Provides HttpServerBox (singleton), HttpRequestBox, HttpResponseBox, HttpClientBox
//! This is a pure in-process stub (no real sockets), suitable for E2E of BoxRef args/returns.

use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, Arc, atomic::{AtomicBool, AtomicU32, Ordering}};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::time::Duration;

// Error codes
const OK: i32 = 0;
const E_SHORT: i32 = -1;
const E_INV_TYPE: i32 = -2;
const E_INV_METHOD: i32 = -3;
const E_INV_ARGS: i32 = -4;
const E_ERR: i32 = -5;
const E_INV_HANDLE: i32 = -8;

// Type IDs
const T_SERVER: u32 = 20;
const T_REQUEST: u32 = 21;
const T_RESPONSE: u32 = 22;
const T_CLIENT: u32 = 23;
// Socket
const T_SOCK_SERVER: u32 = 30;
const T_SOCK_CONN: u32 = 31;
const T_SOCK_CLIENT: u32 = 32;

// Methods
const M_BIRTH: u32 = 0;

// Server
const M_SERVER_START: u32 = 1;
const M_SERVER_STOP: u32 = 2;
const M_SERVER_ACCEPT: u32 = 3; // -> Handle(Request)

// Request
const M_REQ_PATH: u32 = 1;      // -> String
const M_REQ_READ_BODY: u32 = 2; // -> Bytes (optional)
const M_REQ_RESPOND: u32 = 3;   // arg: Handle(Response)

// Response
const M_RESP_SET_STATUS: u32 = 1; // arg: i32
const M_RESP_SET_HEADER: u32 = 2; // args: name, value (string)
const M_RESP_WRITE: u32 = 3;      // arg: bytes/string
const M_RESP_READ_BODY: u32 = 4;  // -> Bytes
const M_RESP_GET_STATUS: u32 = 5; // -> i32
const M_RESP_GET_HEADER: u32 = 6; // arg: name -> string (or empty)

// Client
const M_CLIENT_GET: u32 = 1;   // arg: url -> Handle(Response)
const M_CLIENT_POST: u32 = 2;  // args: url, body(bytes/string) -> Handle(Response)

// Socket Server
const M_SRV_BIRTH: u32 = 0;
const M_SRV_START: u32 = 1; // port
const M_SRV_STOP: u32 = 2;
const M_SRV_ACCEPT: u32 = 3; // -> Handle(T_SOCK_CONN)
const M_SRV_ACCEPT_TIMEOUT: u32 = 4; // ms -> Handle(T_SOCK_CONN) or void

// Socket Client
const M_SC_BIRTH: u32 = 0;
const M_SC_CONNECT: u32 = 1; // host, port -> Handle(T_SOCK_CONN)

// Socket Conn
const M_CONN_BIRTH: u32 = 0;
const M_CONN_SEND: u32 = 1; // bytes/string -> void
const M_CONN_RECV: u32 = 2; // -> bytes
const M_CONN_CLOSE: u32 = 3; // -> void
const M_CONN_RECV_TIMEOUT: u32 = 4; // ms -> bytes (empty if timeout)

// Global State
static SERVER_INSTANCES: Lazy<Mutex<HashMap<u32, ServerState>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static SERVER_START_SEQ: AtomicU32 = AtomicU32::new(1);
static ACTIVE_SERVER_ID: Lazy<Mutex<Option<u32>>> = Lazy::new(|| Mutex::new(None));
static REQUESTS: Lazy<Mutex<HashMap<u32, RequestState>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static RESPONSES: Lazy<Mutex<HashMap<u32, ResponseState>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static CLIENTS: Lazy<Mutex<HashMap<u32, ClientState>>> = Lazy::new(|| Mutex::new(HashMap::new()));

static SERVER_ID: AtomicU32 = AtomicU32::new(1);
static REQUEST_ID: AtomicU32 = AtomicU32::new(1);
static RESPONSE_ID: AtomicU32 = AtomicU32::new(1);
static CLIENT_ID: AtomicU32 = AtomicU32::new(1);
static SOCK_SERVER_ID: AtomicU32 = AtomicU32::new(1);
static SOCK_CONN_ID: AtomicU32 = AtomicU32::new(1);
static SOCK_CLIENT_ID: AtomicU32 = AtomicU32::new(1);

struct ServerState {
    running: bool,
    port: i32,
    pending: VecDeque<u32>, // queue of request ids
    start_seq: u32,
}

struct RequestState {
    path: String,
    body: Vec<u8>,
    response_id: Option<u32>,
}

struct ResponseState {
    status: i32,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

struct ClientState;

// Socket types
struct SockServerState {
    running: Arc<AtomicBool>,
    pending: Arc<Mutex<VecDeque<u32>>>,
    handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

struct SockConnState {
    stream: Mutex<TcpStream>,
}

struct SockClientState;

#[no_mangle]
pub extern "C" fn nyash_plugin_abi() -> u32 { 1 }

#[no_mangle]
pub extern "C" fn nyash_plugin_init() -> i32 { OK }

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
            T_RESPONSE => response_invoke(method_id, instance_id, args, args_len, result, result_len),
            T_CLIENT => client_invoke(method_id, instance_id, args, args_len, result, result_len),
            T_SOCK_SERVER => sock_server_invoke(method_id, instance_id, args, args_len, result, result_len),
            T_SOCK_CLIENT => sock_client_invoke(method_id, instance_id, args, args_len, result, result_len),
            T_SOCK_CONN => sock_conn_invoke(method_id, instance_id, args, args_len, result, result_len),
            _ => E_INV_TYPE,
        }
    }
}

unsafe fn server_invoke(m: u32, id: u32, args: *const u8, args_len: usize, res: *mut u8, res_len: *mut usize) -> i32 {
    match m {
        M_BIRTH => {
            let id = SERVER_ID.fetch_add(1, Ordering::Relaxed);
            SERVER_INSTANCES.lock().unwrap().insert(id, ServerState { running: false, port: 0, pending: VecDeque::new(), start_seq: 0 });
            write_u32(id, res, res_len)
        }
        M_SERVER_START => {
            // args: TLV string/int (port)
            let port = tlv_parse_i32(slice(args, args_len)).unwrap_or(0);
            if let Some(s) = SERVER_INSTANCES.lock().unwrap().get_mut(&id) {
                s.running = true; s.port = port; s.start_seq = SERVER_START_SEQ.fetch_add(1, Ordering::Relaxed);
            }
            // mark active server
            *ACTIVE_SERVER_ID.lock().unwrap() = Some(id);
            write_tlv_void(res, res_len)
        }
        M_SERVER_STOP => {
            if let Some(s) = SERVER_INSTANCES.lock().unwrap().get_mut(&id) {
                s.running = false;
            }
            // clear active if this server was active
            let mut active = ACTIVE_SERVER_ID.lock().unwrap();
            if active.map(|v| v == id).unwrap_or(false) { *active = None; }
            write_tlv_void(res, res_len)
        }
        M_SERVER_ACCEPT => {
            // wait up to ~5000ms for a request to arrive
            for _ in 0..1000 {
                if let Some(req_id) = {
                    let mut map = SERVER_INSTANCES.lock().unwrap();
                    if let Some(s) = map.get_mut(&id) { s.pending.pop_front() } else { None }
                } {
                    return write_tlv_handle(T_REQUEST, req_id, res, res_len);
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            write_tlv_void(res, res_len)
        }
        _ => E_INV_METHOD,
    }
}

unsafe fn request_invoke(m: u32, id: u32, _args: *const u8, _args_len: usize, res: *mut u8, res_len: *mut usize) -> i32 {
    match m {
        M_BIRTH => {
            let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
            REQUESTS.lock().unwrap().insert(id, RequestState { path: String::new(), body: vec![], response_id: None });
            write_u32(id, res, res_len)
        }
        M_REQ_PATH => {
            if let Some(rq) = REQUESTS.lock().unwrap().get(&id) {
                write_tlv_string(&rq.path, res, res_len)
            } else { E_INV_HANDLE }
        }
        M_REQ_READ_BODY => {
            if let Some(rq) = REQUESTS.lock().unwrap().get(&id) {
                write_tlv_bytes(&rq.body, res, res_len)
            } else { E_INV_HANDLE }
        }
        M_REQ_RESPOND => {
            // args: TLV Handle(Response)
            let (t, provided_resp_id) = tlv_parse_handle(slice(_args, _args_len)).map_err(|_| ()).or(Err(())).unwrap_or((0,0));
            if t != T_RESPONSE { return E_INV_ARGS; }
            // Acquire request
            let mut rq_map = REQUESTS.lock().unwrap();
            if let Some(rq) = rq_map.get_mut(&id) {
                // Determine target response id: prefer existing client response id if present
                let target_id = if let Some(existing) = rq.response_id { existing } else { provided_resp_id };
                rq.response_id = Some(target_id);
                drop(rq_map); // release before locking responses

                // Copy response content from provided_resp_id to target_id
                let mut resp_map = RESPONSES.lock().unwrap();
                let (src_status, src_headers, src_body) = if let Some(src) = resp_map.get(&provided_resp_id) {
                    (src.status, src.headers.clone(), src.body.clone())
                } else { return E_INV_HANDLE };
                let dst = resp_map.entry(target_id).or_insert(ResponseState { status: 200, headers: HashMap::new(), body: vec![] });
                dst.status = src_status;
                dst.headers = src_headers;
                dst.body = src_body;
                return write_tlv_void(res, res_len);
            }
            E_INV_HANDLE
        }
        _ => E_INV_METHOD,
    }
}

unsafe fn response_invoke(m: u32, id: u32, args: *const u8, args_len: usize, res: *mut u8, res_len: *mut usize) -> i32 {
    match m {
        M_BIRTH => {
            let id = RESPONSE_ID.fetch_add(1, Ordering::Relaxed);
            RESPONSES.lock().unwrap().insert(id, ResponseState { status: 200, headers: HashMap::new(), body: vec![] });
            write_u32(id, res, res_len)
        }
        M_RESP_SET_STATUS => {
            let code = tlv_parse_i32(slice(args, args_len)).unwrap_or(200);
            if let Some(rp) = RESPONSES.lock().unwrap().get_mut(&id) { rp.status = code; }
            write_tlv_void(res, res_len)
        }
        M_RESP_SET_HEADER => {
            if let Ok((name, value)) = tlv_parse_two_strings(slice(args, args_len)) {
                if let Some(rp) = RESPONSES.lock().unwrap().get_mut(&id) { rp.headers.insert(name, value); }
                return write_tlv_void(res, res_len);
            }
            E_INV_ARGS
        }
        M_RESP_WRITE => {
            // Accept String or Bytes
            let bytes = tlv_parse_bytes(slice(args, args_len)).unwrap_or_default();
            if let Some(rp) = RESPONSES.lock().unwrap().get_mut(&id) { rp.body.extend_from_slice(&bytes); }
            write_tlv_void(res, res_len)
        }
        M_RESP_READ_BODY => {
            if let Some(rp) = RESPONSES.lock().unwrap().get(&id) { write_tlv_bytes(&rp.body, res, res_len) } else { E_INV_HANDLE }
        }
        M_RESP_GET_STATUS => {
            if let Some(rp) = RESPONSES.lock().unwrap().get(&id) { write_tlv_i32(rp.status, res, res_len) } else { E_INV_HANDLE }
        }
        M_RESP_GET_HEADER => {
            if let Ok(name) = tlv_parse_string(slice(args, args_len)) {
                if let Some(rp) = RESPONSES.lock().unwrap().get(&id) {
                    let v = rp.headers.get(&name).cloned().unwrap_or_default();
                    return write_tlv_string(&v, res, res_len);
                } else { return E_INV_HANDLE; }
            }
            E_INV_ARGS
        }
        _ => E_INV_METHOD,
    }
}

unsafe fn client_invoke(m: u32, id: u32, args: *const u8, args_len: usize, res: *mut u8, res_len: *mut usize) -> i32 {
    match m {
        M_BIRTH => {
            let id = CLIENT_ID.fetch_add(1, Ordering::Relaxed);
            CLIENTS.lock().unwrap().insert(id, ClientState);
            write_u32(id, res, res_len)
        }
        M_CLIENT_GET => {
            // args: TLV String(url)
            let url = tlv_parse_string(slice(args, args_len)).unwrap_or_default();
            let path = parse_path(&url);
            let port_hint = parse_port(&url);
            // Create Request
            let req_id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
            REQUESTS.lock().unwrap().insert(req_id, RequestState { path, body: vec![], response_id: None });
            // Enqueue to server: prefer port match, else newest running
            {
                let mut servers = SERVER_INSTANCES.lock().unwrap();
                if let Some(ph) = port_hint {
                    if let Some((_, s)) = servers.iter_mut().find(|(_, s)| s.running && s.port == ph) {
                        s.pending.push_back(req_id);
                    } else {
                        if let Some((_, s)) = servers.iter_mut().filter(|(_, s)| s.running).max_by_key(|(_, s)| s.start_seq) {
                            s.pending.push_back(req_id);
                        }
                    }
                } else if let Some((_, s)) = servers.iter_mut().filter(|(_, s)| s.running).max_by_key(|(_, s)| s.start_seq) {
                    s.pending.push_back(req_id);
                }
            }
            // Create Response handle for client side to read later
            let resp_id = RESPONSE_ID.fetch_add(1, Ordering::Relaxed);
            RESPONSES.lock().unwrap().insert(resp_id, ResponseState { status: 200, headers: HashMap::new(), body: vec![] });
            // Link
            if let Some(rq) = REQUESTS.lock().unwrap().get_mut(&req_id) { rq.response_id = Some(resp_id); }
            // Return Handle(Response)
            write_tlv_handle(T_RESPONSE, resp_id, res, res_len)
        }
        M_CLIENT_POST => {
            // args: TLV String(url), Bytes body
            let data = slice(args, args_len);
            let (_, argc, mut pos) = tlv_parse_header(data).map_err(|_| ()).or(Err(())).unwrap_or((1,0,4));
            if argc < 2 { return E_INV_ARGS; }
            let (_t1, s1, p1) = tlv_parse_entry_hdr(data, pos).map_err(|_| ()).or(Err(())).unwrap_or((0,0,0));
            if data[pos] != 6 { return E_INV_ARGS; }
            let url = std::str::from_utf8(&data[p1..p1+s1]).map_err(|_| ()).or(Err(())) .unwrap_or("").to_string();
            pos = p1 + s1;
            let (t2, s2, p2) = tlv_parse_entry_hdr(data, pos).map_err(|_| ()).or(Err(())).unwrap_or((0,0,0));
            if t2 != 6 && t2 != 7 { return E_INV_ARGS; }
            let body = data[p2..p2+s2].to_vec();

            let path = parse_path(&url);
            let port_hint = parse_port(&url);
            // Create Request
            let req_id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
            REQUESTS.lock().unwrap().insert(req_id, RequestState { path, body, response_id: None });
            // Enqueue to server: prefer port match, else newest running
            {
                let mut servers = SERVER_INSTANCES.lock().unwrap();
                if let Some(ph) = port_hint {
                    if let Some((_, s)) = servers.iter_mut().find(|(_, s)| s.running && s.port == ph) {
                        s.pending.push_back(req_id);
                    } else {
                        if let Some((_, s)) = servers.iter_mut().filter(|(_, s)| s.running).max_by_key(|(_, s)| s.start_seq) {
                            s.pending.push_back(req_id);
                        }
                    }
                } else if let Some((_, s)) = servers.iter_mut().filter(|(_, s)| s.running).max_by_key(|(_, s)| s.start_seq) {
                    s.pending.push_back(req_id);
                }
            }
            // Create paired client Response
            let resp_id = RESPONSE_ID.fetch_add(1, Ordering::Relaxed);
            RESPONSES.lock().unwrap().insert(resp_id, ResponseState { status: 200, headers: HashMap::new(), body: vec![] });
            if let Some(rq) = REQUESTS.lock().unwrap().get_mut(&req_id) { rq.response_id = Some(resp_id); }
            write_tlv_handle(T_RESPONSE, resp_id, res, res_len)
        }
        _ => E_INV_METHOD,
    }
}

fn parse_path(url: &str) -> String {
    // Robust-ish path extraction:
    // - http://host:port/path -> "/path"
    // - https://host/path -> "/path"
    // - /relative -> as-is
    // - otherwise -> "/"
    if url.starts_with('/') { return url.to_string(); }
    if let Some(scheme_pos) = url.find("//") {
        let after_scheme = &url[scheme_pos+2..];
        if let Some(slash) = after_scheme.find('/') {
            return after_scheme[slash..].to_string();
        } else {
            return "/".to_string();
        }
    }
    "/".to_string()
}

fn parse_port(url: &str) -> Option<i32> {
    // match patterns like http://host:PORT/ or :PORT/
    if let Some(pat) = url.split("//").nth(1) {
        if let Some(after_host) = pat.split('/').next() {
            if let Some(colon) = after_host.rfind(':') {
                return after_host[colon+1..].parse::<i32>().ok();
            }
        }
    }
    None
}

// ===== Helpers =====
unsafe fn slice<'a>(p: *const u8, len: usize) -> &'a [u8] { std::slice::from_raw_parts(p, len) }

fn write_u32(v: u32, res: *mut u8, res_len: *mut usize) -> i32 {
    unsafe {
        if res_len.is_null() { return E_INV_ARGS; }
        if res.is_null() || *res_len < 4 { *res_len = 4; return E_SHORT; }
        let b = v.to_le_bytes();
        std::ptr::copy_nonoverlapping(b.as_ptr(), res, 4);
        *res_len = 4;
    }
    OK
}

fn write_tlv_result(payloads: &[(u8, &[u8])], res: *mut u8, res_len: *mut usize) -> i32 {
    if res_len.is_null() { return E_INV_ARGS; }
    let mut buf = Vec::with_capacity(4 + payloads.iter().map(|(_,p)| 4 + p.len()).sum::<usize>());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes());
    for (tag, p) in payloads {
        buf.push(*tag); buf.push(0); buf.extend_from_slice(&(p.len() as u16).to_le_bytes()); buf.extend_from_slice(p);
    }
    unsafe {
        let need = buf.len();
        if res.is_null() || *res_len < need { *res_len = need; return E_SHORT; }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), res, need);
        *res_len = need;
    }
    OK
}

fn write_tlv_void(res: *mut u8, res_len: *mut usize) -> i32 { write_tlv_result(&[(9u8, &[])], res, res_len) }
fn write_tlv_string(s: &str, res: *mut u8, res_len: *mut usize) -> i32 { write_tlv_result(&[(6u8, s.as_bytes())], res, res_len) }
fn write_tlv_bytes(b: &[u8], res: *mut u8, res_len: *mut usize) -> i32 { write_tlv_result(&[(7u8, b)], res, res_len) }
fn write_tlv_i32(v: i32, res: *mut u8, res_len: *mut usize) -> i32 { write_tlv_result(&[(2u8, &v.to_le_bytes())], res, res_len) }
fn write_tlv_handle(t: u32, id: u32, res: *mut u8, res_len: *mut usize) -> i32 {
    let mut payload = [0u8;8]; payload[0..4].copy_from_slice(&t.to_le_bytes()); payload[4..8].copy_from_slice(&id.to_le_bytes());
    write_tlv_result(&[(8u8, &payload)], res, res_len)
}

fn tlv_parse_header(data: &[u8]) -> Result<(u16,u16,usize), ()> {
    if data.len() < 4 { return Err(()); }
    let ver = u16::from_le_bytes([data[0], data[1]]); let argc = u16::from_le_bytes([data[2], data[3]]);
    if ver != 1 { return Err(()); }
    Ok((ver, argc, 4))
}
fn tlv_parse_string(data: &[u8]) -> Result<String, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?; if argc < 1 { return Err(()); }
    let (tag, size, p) = tlv_parse_entry_hdr(data, pos)?; if tag != 6 { return Err(()); }
    Ok(std::str::from_utf8(&data[p..p+size]).map_err(|_| ())?.to_string())
}
fn tlv_parse_two_strings(data: &[u8]) -> Result<(String,String), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?; if argc < 2 { return Err(()); }
    let (tag1, size1, p1) = tlv_parse_entry_hdr(data, pos)?; if tag1 != 6 { return Err(()); }
    let s1 = std::str::from_utf8(&data[p1..p1+size1]).map_err(|_| ())?.to_string(); pos = p1+size1;
    let (tag2, size2, p2) = tlv_parse_entry_hdr(data, pos)?; if tag2 != 6 { return Err(()); }
    let s2 = std::str::from_utf8(&data[p2..p2+size2]).map_err(|_| ())?.to_string();
    Ok((s1,s2))
}
fn tlv_parse_bytes(data: &[u8]) -> Result<Vec<u8>, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?; if argc < 1 { return Err(()); }
    let (tag, size, p) = tlv_parse_entry_hdr(data, pos)?; if tag != 6 && tag != 7 { return Err(()); }
    Ok(data[p..p+size].to_vec())
}
fn tlv_parse_i32(data: &[u8]) -> Result<i32, ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?; if argc < 1 { return Err(()); }
    let (tag, size, p) = tlv_parse_entry_hdr(data, pos)?;
    match (tag, size) {
        (2, 4) => { let mut b=[0u8;4]; b.copy_from_slice(&data[p..p+4]); Ok(i32::from_le_bytes(b)) }
        (5, 8) => { // accept i64
            let mut b=[0u8;8]; b.copy_from_slice(&data[p..p+8]); Ok(i64::from_le_bytes(b) as i32)
        }
        _ => Err(())
    }
}
fn tlv_parse_handle(data: &[u8]) -> Result<(u32,u32), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?; if argc < 1 { return Err(()); }
    let (tag, size, p) = tlv_parse_entry_hdr(data, pos)?; if tag != 8 || size != 8 { return Err(()); }
    let mut t = [0u8;4]; let mut i = [0u8;4]; t.copy_from_slice(&data[p..p+4]); i.copy_from_slice(&data[p+4..p+8]);
    Ok((u32::from_le_bytes(t), u32::from_le_bytes(i)))
}
fn tlv_parse_entry_hdr(data: &[u8], pos: usize) -> Result<(u8,usize,usize), ()> {
    if pos+4 > data.len() { return Err(()); }
    let tag = data[pos]; let _rsv = data[pos+1]; let size = u16::from_le_bytes([data[pos+2], data[pos+3]]) as usize; let p = pos+4;
    if p+size > data.len() { return Err(()); }
    Ok((tag,size,p))
}

// ===== Simple logger (enabled when NYASH_NET_LOG=1) =====
static LOG_ON: Lazy<bool> = Lazy::new(|| std::env::var("NYASH_NET_LOG").unwrap_or_default() == "1");
static LOG_PATH: Lazy<String> = Lazy::new(|| std::env::var("NYASH_NET_LOG_FILE").unwrap_or_else(|_| "net_plugin.log".to_string()));
static LOG_MTX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn net_log(msg: &str) {
    if !*LOG_ON { return; }
    // Always mirror to stderr for visibility
    eprintln!("[net] {}", msg);
    let _g = LOG_MTX.lock().unwrap();
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&*LOG_PATH) {
        let _ = writeln!(f, "[{:?}] {}", std::time::SystemTime::now(), msg);
    }
}

macro_rules! netlog {
    ($($arg:tt)*) => {{ let s = format!($($arg)*); net_log(&s); }}
}

// ===== Socket implementation =====
static SOCK_SERVERS: Lazy<Mutex<HashMap<u32, SockServerState>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static SOCK_CONNS: Lazy<Mutex<HashMap<u32, SockConnState>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static SOCK_CLIENTS: Lazy<Mutex<HashMap<u32, SockClientState>>> = Lazy::new(|| Mutex::new(HashMap::new()));

unsafe fn sock_server_invoke(m: u32, id: u32, args: *const u8, args_len: usize, res: *mut u8, res_len: *mut usize) -> i32 {
    match m {
        M_SRV_BIRTH => {
            netlog!("sock:birth server");
            let id = SOCK_SERVER_ID.fetch_add(1, Ordering::Relaxed);
            SOCK_SERVERS.lock().unwrap().insert(id, SockServerState { running: Arc::new(AtomicBool::new(false)), pending: Arc::new(Mutex::new(VecDeque::new())), handle: Mutex::new(None) });
            write_u32(id, res, res_len)
        }
        M_SRV_START => {
            let port = tlv_parse_i32(slice(args, args_len)).unwrap_or(0);
            netlog!("sock:start server id={} port={}", id, port);
            if let Some(ss) = SOCK_SERVERS.lock().unwrap().get(&id) {
                let running = ss.running.clone();
                let pending = ss.pending.clone();
                running.store(true, Ordering::SeqCst);
                let handle = std::thread::spawn(move || {
                    let addr = format!("127.0.0.1:{}", port);
                    let listener = TcpListener::bind(addr);
                    if let Ok(listener) = listener {
                        listener.set_nonblocking(true).ok();
                        while running.load(Ordering::SeqCst) {
                            match listener.accept() {
                                Ok((stream, _)) => {
                                    stream.set_nonblocking(false).ok();
                                    let conn_id = SOCK_CONN_ID.fetch_add(1, Ordering::Relaxed);
                                    SOCK_CONNS.lock().unwrap().insert(conn_id, SockConnState { stream: Mutex::new(stream) });
                                    netlog!("sock:accept conn_id={}", conn_id);
                                    pending.lock().unwrap().push_back(conn_id);
                                }
                                Err(_) => {
                                    std::thread::sleep(std::time::Duration::from_millis(10));
                                }
                            }
                        }
                        netlog!("sock:listener exit port={}", port);
                    }
                });
                *ss.handle.lock().unwrap() = Some(handle);
            }
            write_tlv_void(res, res_len)
        }
        M_SRV_STOP => {
            netlog!("sock:stop server id={}", id);
            if let Some(ss) = SOCK_SERVERS.lock().unwrap().get(&id) {
                ss.running.store(false, Ordering::SeqCst);
                if let Some(h) = ss.handle.lock().unwrap().take() { let _ = h.join(); }
            }
            write_tlv_void(res, res_len)
        }
        M_SRV_ACCEPT => {
            if let Some(ss) = SOCK_SERVERS.lock().unwrap().get(&id) {
                // wait up to ~5000ms
                for _ in 0..1000 {
                    if let Some(cid) = ss.pending.lock().unwrap().pop_front() {
                        netlog!("sock:accept returned conn_id={}", cid);
                        return write_tlv_handle(T_SOCK_CONN, cid, res, res_len);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
            netlog!("sock:accept timeout id={}", id);
            write_tlv_void(res, res_len)
        }
        M_SRV_ACCEPT_TIMEOUT => {
            let timeout_ms = tlv_parse_i32(slice(args, args_len)).unwrap_or(0).max(0) as u64;
            if let Some(ss) = SOCK_SERVERS.lock().unwrap().get(&id) {
                let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
                loop {
                    if let Some(cid) = ss.pending.lock().unwrap().pop_front() {
                        netlog!("sock:acceptTimeout returned conn_id={}", cid);
                        return write_tlv_handle(T_SOCK_CONN, cid, res, res_len);
                    }
                    if std::time::Instant::now() >= deadline { break; }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
            netlog!("sock:acceptTimeout timeout id={} ms={}", id, timeout_ms);
            write_tlv_void(res, res_len)
        }
        _ => E_INV_METHOD,
    }
}

unsafe fn sock_client_invoke(m: u32, id: u32, args: *const u8, args_len: usize, res: *mut u8, res_len: *mut usize) -> i32 {
    match m {
        M_SC_BIRTH => {
            let id = SOCK_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
            SOCK_CLIENTS.lock().unwrap().insert(id, SockClientState);
            write_u32(id, res, res_len)
        }
        M_SC_CONNECT => {
            // args: host(string), port(i32)
            let data = slice(args, args_len);
            let (_, argc, mut pos) = tlv_parse_header(data).map_err(|_| ()).or(Err(())).unwrap_or((1,0,4));
            if argc < 2 { return E_INV_ARGS; }
            let (_t1, s1, p1) = tlv_parse_entry_hdr(data, pos).map_err(|_| ()).or(Err(())).unwrap_or((0,0,0));
            if data[pos] != 6 { return E_INV_ARGS; }
            let host = std::str::from_utf8(&data[p1..p1+s1]).map_err(|_| ()).or(Err(())) .unwrap_or("").to_string();
            pos = p1 + s1;
            let (_t2, _s2, p2) = tlv_parse_entry_hdr(data, pos).map_err(|_| ()).or(Err(())).unwrap_or((0,0,0));
            let port = if data[pos] == 2 { // i32
                let mut b=[0u8;4]; b.copy_from_slice(&data[p2..p2+4]); i32::from_le_bytes(b)
            } else { return E_INV_ARGS };
            let addr = format!("{}:{}", host, port);
            match TcpStream::connect(addr) {
                Ok(mut stream) => {
                    stream.set_nonblocking(false).ok();
                    let conn_id = SOCK_CONN_ID.fetch_add(1, Ordering::Relaxed);
                    SOCK_CONNS.lock().unwrap().insert(conn_id, SockConnState { stream: Mutex::new(stream) });
                    netlog!("sock:connect ok conn_id={}", conn_id);
                    write_tlv_handle(T_SOCK_CONN, conn_id, res, res_len)
                }
                Err(e) => { netlog!("sock:connect error: {:?}", e); E_ERR }
            }
        }
        _ => E_INV_METHOD,
    }
}

unsafe fn sock_conn_invoke(m: u32, id: u32, args: *const u8, args_len: usize, res: *mut u8, res_len: *mut usize) -> i32 {
    match m {
        M_CONN_BIRTH => {
            // not used directly
            write_u32(0, res, res_len)
        }
        M_CONN_SEND => {
            let bytes = tlv_parse_bytes(slice(args, args_len)).unwrap_or_default();
            if let Some(conn) = SOCK_CONNS.lock().unwrap().get(&id) {
                if let Ok(mut s) = conn.stream.lock() { let _ = s.write_all(&bytes); }
                netlog!("sock:send id={} n={}", id, bytes.len());
                return write_tlv_void(res, res_len);
            }
            E_INV_HANDLE
        }
        M_CONN_RECV => {
            if let Some(conn) = SOCK_CONNS.lock().unwrap().get(&id) {
                if let Ok(mut s) = conn.stream.lock() {
                    let mut buf = vec![0u8; 4096];
                    match s.read(&mut buf) {
                        Ok(n) => { buf.truncate(n); netlog!("sock:recv id={} n={}", id, n); return write_tlv_bytes(&buf, res, res_len); }
                        Err(_) => return write_tlv_bytes(&[], res, res_len),
                    }
                }
            }
            E_INV_HANDLE
        }
        M_CONN_RECV_TIMEOUT => {
            let timeout_ms = tlv_parse_i32(slice(args, args_len)).unwrap_or(0).max(0) as u64;
            if let Some(conn) = SOCK_CONNS.lock().unwrap().get(&id) {
                if let Ok(mut s) = conn.stream.lock() {
                    let _ = s.set_read_timeout(Some(Duration::from_millis(timeout_ms)));
                    let mut buf = vec![0u8; 4096];
                    let resv = s.read(&mut buf);
                    let _ = s.set_read_timeout(None);
                    match resv {
                        Ok(n) => { buf.truncate(n); netlog!("sock:recvTimeout id={} n={} ms={}", id, n, timeout_ms); return write_tlv_bytes(&buf, res, res_len); }
                        Err(_) => return write_tlv_bytes(&[], res, res_len),
                    }
                }
            }
            E_INV_HANDLE
        }
        M_CONN_CLOSE => {
            // Drop the stream by removing entry
            SOCK_CONNS.lock().unwrap().remove(&id);
            write_tlv_void(res, res_len)
        }
        _ => E_INV_METHOD,
    }
}
