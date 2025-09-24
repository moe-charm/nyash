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
