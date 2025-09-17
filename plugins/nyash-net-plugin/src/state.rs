use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Mutex,
};

use super::{
    ClientState, RequestState, ResponseState, ServerState, SockClientState, SockConnState,
    SockServerState,
};

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
pub(crate) static SOCK_CLIENT_ID: AtomicU32 = AtomicU32::new(1);

pub(crate) static SOCK_SERVERS: Lazy<Mutex<HashMap<u32, SockServerState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub(crate) static SOCK_CONNS: Lazy<Mutex<HashMap<u32, SockConnState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
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
pub(crate) fn next_sock_client_id() -> u32 {
    SOCK_CLIENT_ID.fetch_add(1, Ordering::Relaxed)
}
