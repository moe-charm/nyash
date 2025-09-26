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
    unsafe {
        sockets::sock_client_invoke(method_id, instance_id, args, args_len, result, result_len)
    }
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
