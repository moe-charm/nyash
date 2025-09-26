use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::net::TcpStream;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Mutex,
};

// Local state structs formerly defined in lib.rs
pub(crate) struct ServerState {
    pub(crate) running: Arc<AtomicBool>,
    pub(crate) port: i32,
    pub(crate) pending: Arc<Mutex<VecDeque<u32>>>, // queue of request ids
    pub(crate) handle: Mutex<Option<std::thread::JoinHandle<()>>>,
    pub(crate) start_seq: u32,
}

pub(crate) struct RequestState {
    pub(crate) path: String,
    pub(crate) body: Vec<u8>,
    pub(crate) response_id: Option<u32>,
    // For HTTP-over-TCP server: map to an active accepted socket to respond on
    pub(crate) server_conn_id: Option<u32>,
    pub(crate) responded: bool,
}

pub(crate) struct ResponseState {
    pub(crate) status: i32,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Vec<u8>,
    // For HTTP-over-TCP client: associated socket connection id to read from
    pub(crate) client_conn_id: Option<u32>,
    pub(crate) parsed: bool,
}

pub(crate) struct ClientState;

// Socket types
pub(crate) struct SockServerState {
    pub(crate) running: Arc<AtomicBool>,
    pub(crate) pending: Arc<Mutex<VecDeque<u32>>>,
    pub(crate) handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

pub(crate) struct SockConnState {
    pub(crate) stream: Mutex<TcpStream>,
}

pub(crate) struct SockClientState;

pub(crate) static SERVER_INSTANCES: Lazy<Mutex<HashMap<u32, ServerState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub(crate) static SERVER_START_SEQ: AtomicU32 = AtomicU32::new(1);
pub(crate) static ACTIVE_SERVER_ID: Lazy<Mutex<Option<u32>>> = Lazy::new(|| Mutex::new(None));
pub(crate) static LAST_ACCEPTED_REQ: Lazy<Mutex<Option<u32>>> = Lazy::new(|| Mutex::new(None));
pub(crate) static REQUESTS: Lazy<Mutex<HashMap<u32, RequestState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub(crate) static RESPONSES: Lazy<Mutex<HashMap<u32, ResponseState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub(crate) static CLIENTS: Lazy<Mutex<HashMap<u32, ClientState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub(crate) static SERVER_ID: AtomicU32 = AtomicU32::new(1);
pub(crate) static REQUEST_ID: AtomicU32 = AtomicU32::new(1);
pub(crate) static RESPONSE_ID: AtomicU32 = AtomicU32::new(1);
pub(crate) static CLIENT_ID: AtomicU32 = AtomicU32::new(1);
pub(crate) static SOCK_SERVER_ID: AtomicU32 = AtomicU32::new(1);
pub(crate) static SOCK_CONN_ID: AtomicU32 = AtomicU32::new(1);
#[allow(dead_code)]
pub(crate) static SOCK_CLIENT_ID: AtomicU32 = AtomicU32::new(1);

pub(crate) static SOCK_SERVERS: Lazy<Mutex<HashMap<u32, SockServerState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub(crate) static SOCK_CONNS: Lazy<Mutex<HashMap<u32, SockConnState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
#[allow(dead_code)]
pub(crate) static SOCK_CLIENTS: Lazy<Mutex<HashMap<u32, SockClientState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[inline]
pub(crate) fn next_server_id() -> u32 {
    SERVER_ID.fetch_add(1, Ordering::Relaxed)
}

#[inline]
pub(crate) fn next_server_start_seq() -> u32 {
    SERVER_START_SEQ.fetch_add(1, Ordering::Relaxed)
}

#[inline]
pub(crate) fn next_request_id() -> u32 {
    REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

#[inline]
pub(crate) fn next_response_id() -> u32 {
    RESPONSE_ID.fetch_add(1, Ordering::Relaxed)
}

#[inline]
pub(crate) fn next_client_id() -> u32 {
    CLIENT_ID.fetch_add(1, Ordering::Relaxed)
}

#[inline]
pub(crate) fn next_sock_server_id() -> u32 {
    SOCK_SERVER_ID.fetch_add(1, Ordering::Relaxed)
}

#[inline]
pub(crate) fn next_sock_conn_id() -> u32 {
    SOCK_CONN_ID.fetch_add(1, Ordering::Relaxed)
}

#[inline]
#[allow(dead_code)]
pub(crate) fn next_sock_client_id() -> u32 {
    SOCK_CLIENT_ID.fetch_add(1, Ordering::Relaxed)
}
