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
