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
                let host =
                    http_helpers::parse_host(&url).unwrap_or_else(|| "127.0.0.1".to_string());
                let path = http_helpers::parse_path(&url);
                // Create client response handle first, so we can include it in header
                let resp_id = state::next_response_id();
                let (_h, _p, req_bytes) =
                    http_helpers::build_http_request("GET", &url, None, resp_id);
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
                let host =
                    http_helpers::parse_host(&url).unwrap_or_else(|| "127.0.0.1".to_string());
                let path = http_helpers::parse_path(&url);
                let body_len = body.len();
                // Create client response handle
                let resp_id = state::next_response_id();
                let (_h, _p, req_bytes) =
                    http_helpers::build_http_request("POST", &url, Some(&body), resp_id);
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
pub static nyash_typebox_ClientBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: 0x54594258,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: b"ClientBox\0".as_ptr() as *const std::os::raw::c_char,
    resolve: Some(clientbox_resolve),
    invoke_id: Some(clientbox_invoke_id),
    capabilities: 0,
};
