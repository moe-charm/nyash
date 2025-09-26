use std::collections::VecDeque;
use std::io::{Read, Write as IoWrite};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

use crate::consts::*;
use crate::logging::net_log;
use crate::state::{self, SockConnState, SockServerState};

fn logf(s: String) {
    net_log(&s);
}

pub(crate) unsafe fn sock_server_invoke(
    m: u32,
    id: u32,
    args: *const u8,
    args_len: usize,
    res: *mut u8,
    res_len: *mut usize,
) -> i32 {
    match m {
        M_SRV_BIRTH => {
            logf(format!("sock:birth server"));
            let id = state::next_sock_server_id();
            state::SOCK_SERVERS.lock().unwrap().insert(
                id,
                SockServerState {
                    running: Arc::new(AtomicBool::new(false)),
                    pending: Arc::new(Mutex::new(VecDeque::new())),
                    handle: Mutex::new(None),
                },
            );
            crate::tlv::write_u32(id, res, res_len)
        }
        M_SRV_START => {
            let port = crate::tlv::tlv_parse_i32(super::ffi::slice(args, args_len)).unwrap_or(0);
            logf(format!("sock:start server id={} port={}", id, port));
            if let Some(ss) = state::SOCK_SERVERS.lock().unwrap().get(&id) {
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
                                    let conn_id = state::next_sock_conn_id();
                                    state::SOCK_CONNS.lock().unwrap().insert(
                                        conn_id,
                                        SockConnState {
                                            stream: Mutex::new(stream),
                                        },
                                    );
                                    logf(format!("sock:accept conn_id={}", conn_id));
                                    pending.lock().unwrap().push_back(conn_id);
                                }
                                Err(_) => {
                                    std::thread::sleep(std::time::Duration::from_millis(10));
                                }
                            }
                        }
                        logf(format!("sock:listener exit port={}", port));
                    }
                });
                *ss.handle.lock().unwrap() = Some(handle);
            }
            crate::tlv::write_tlv_void(res, res_len)
        }
        M_SRV_STOP => {
            logf(format!("sock:stop server id={}", id));
            if let Some(ss) = state::SOCK_SERVERS.lock().unwrap().get(&id) {
                ss.running.store(false, Ordering::SeqCst);
                if let Some(h) = ss.handle.lock().unwrap().take() {
                    let _ = h.join();
                }
            }
            crate::tlv::write_tlv_void(res, res_len)
        }
        M_SRV_ACCEPT => {
            if let Some(ss) = state::SOCK_SERVERS.lock().unwrap().get(&id) {
                // wait up to ~5000ms
                for _ in 0..1000 {
                    if let Some(cid) = ss.pending.lock().unwrap().pop_front() {
                        logf(format!("sock:accept returned conn_id={}", cid));
                        return crate::tlv::write_tlv_handle(T_SOCK_CONN, cid, res, res_len);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
            logf(format!("sock:accept timeout id={}", id));
            crate::tlv::write_tlv_void(res, res_len)
        }
        M_SRV_ACCEPT_TIMEOUT => {
            let timeout_ms = crate::tlv::tlv_parse_i32(super::ffi::slice(args, args_len))
                .unwrap_or(0)
                .max(0) as u64;
            if let Some(ss) = state::SOCK_SERVERS.lock().unwrap().get(&id) {
                // wait up to timeout
                let loops = (timeout_ms / 5).max(1);
                for _ in 0..loops {
                    if let Some(cid) = ss.pending.lock().unwrap().pop_front() {
                        logf(format!("sock:accept returned conn_id={}", cid));
                        return crate::tlv::write_tlv_handle(T_SOCK_CONN, cid, res, res_len);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
            crate::tlv::write_tlv_void(res, res_len)
        }
        _ => E_INV_METHOD,
    }
}

pub(crate) unsafe fn sock_client_invoke(
    m: u32,
    _id: u32,
    args: *const u8,
    args_len: usize,
    res: *mut u8,
    res_len: *mut usize,
) -> i32 {
    match m {
        M_SC_BIRTH => {
            // opaque handle box
            crate::tlv::write_u32(0, res, res_len)
        }
        M_SC_CONNECT => {
            let data = super::ffi::slice(args, args_len);
            let mut pos = 0usize;
            let (_t1, s1, p1) = crate::tlv::tlv_parse_entry_hdr(data, pos)
                .map_err(|_| ())
                .or(Err(()))
                .unwrap_or((0, 0, 0));
            if data[pos] != 6 {
                return E_INV_ARGS;
            }
            let host = std::str::from_utf8(&data[p1..p1 + s1])
                .map_err(|_| ())
                .or(Err(()))
                .unwrap_or("")
                .to_string();
            pos = p1 + s1;
            let (_t2, _s2, p2) = crate::tlv::tlv_parse_entry_hdr(data, pos)
                .map_err(|_| ())
                .or(Err(()))
                .unwrap_or((0, 0, 0));
            let port = if data[pos] == 2 {
                // i32
                let mut b = [0u8; 4];
                b.copy_from_slice(&data[p2..p2 + 4]);
                i32::from_le_bytes(b)
            } else {
                return E_INV_ARGS;
            };
            let addr = format!("{}:{}", host, port);
            match TcpStream::connect(addr) {
                Ok(stream) => {
                    stream.set_nonblocking(false).ok();
                    let conn_id = state::next_sock_conn_id();
                    state::SOCK_CONNS.lock().unwrap().insert(
                        conn_id,
                        SockConnState {
                            stream: Mutex::new(stream),
                        },
                    );
                    logf(format!("sock:connect ok conn_id={}", conn_id));
                    crate::tlv::write_tlv_handle(T_SOCK_CONN, conn_id, res, res_len)
                }
                Err(e) => {
                    logf(format!("sock:connect error: {:?}", e));
                    E_ERR
                }
            }
        }
        _ => E_INV_METHOD,
    }
}

pub(crate) unsafe fn sock_conn_invoke(
    m: u32,
    id: u32,
    args: *const u8,
    args_len: usize,
    res: *mut u8,
    res_len: *mut usize,
) -> i32 {
    match m {
        M_CONN_BIRTH => {
            // not used directly
            crate::tlv::write_u32(0, res, res_len)
        }
        M_CONN_SEND => {
            let bytes =
                crate::tlv::tlv_parse_bytes(super::ffi::slice(args, args_len)).unwrap_or_default();
            if let Some(conn) = state::SOCK_CONNS.lock().unwrap().get(&id) {
                if let Ok(mut s) = conn.stream.lock() {
                    let _ = s.write_all(&bytes);
                }
                logf(format!("sock:send id={} n={}", id, bytes.len()));
                return crate::tlv::write_tlv_void(res, res_len);
            }
            E_INV_HANDLE
        }
        M_CONN_RECV => {
            if let Some(conn) = state::SOCK_CONNS.lock().unwrap().get(&id) {
                if let Ok(mut s) = conn.stream.lock() {
                    let mut buf = vec![0u8; 4096];
                    match s.read(&mut buf) {
                        Ok(n) => {
                            buf.truncate(n);
                            logf(format!("sock:recv id={} n={}", id, n));
                            return crate::tlv::write_tlv_bytes(&buf, res, res_len);
                        }
                        Err(_) => return crate::tlv::write_tlv_bytes(&[], res, res_len),
                    }
                }
            }
            E_INV_HANDLE
        }
        M_CONN_RECV_TIMEOUT => {
            let timeout_ms = crate::tlv::tlv_parse_i32(super::ffi::slice(args, args_len))
                .unwrap_or(0)
                .max(0) as u64;
            if let Some(conn) = state::SOCK_CONNS.lock().unwrap().get(&id) {
                if let Ok(mut s) = conn.stream.lock() {
                    let _ = s.set_read_timeout(Some(Duration::from_millis(timeout_ms)));
                    let mut buf = vec![0u8; 4096];
                    let resv = s.read(&mut buf);
                    let _ = s.set_read_timeout(None);
                    match resv {
                        Ok(n) => {
                            buf.truncate(n);
                            logf(format!(
                                "sock:recvTimeout id={} n={} ms={}",
                                id, n, timeout_ms
                            ));
                            return crate::tlv::write_tlv_bytes(&buf, res, res_len);
                        }
                        Err(e) => {
                            logf(format!(
                                "sock:recvTimeout error id={} ms={} err={:?}",
                                id, timeout_ms, e
                            ));
                            return E_ERR;
                        }
                    }
                }
            }
            E_INV_HANDLE
        }
        M_CONN_CLOSE => {
            // Drop the stream by removing entry
            state::SOCK_CONNS.lock().unwrap().remove(&id);
            crate::tlv::write_tlv_void(res, res_len)
        }
        _ => E_INV_METHOD,
    }
}
