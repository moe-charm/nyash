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
