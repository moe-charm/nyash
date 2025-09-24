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
