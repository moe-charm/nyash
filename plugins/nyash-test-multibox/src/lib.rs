//! Test Multi-Box Plugin for Nyash
//! 
//! Provides TestBoxA and TestBoxB to demonstrate multi-box plugin support

use std::collections::HashMap;
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

// ============ FFI Types ============

#[repr(C)]
pub struct NyashHostVtable {
    pub alloc: unsafe extern "C" fn(size: usize) -> *mut u8,
    pub free: unsafe extern "C" fn(ptr: *mut u8),
    pub wake: unsafe extern "C" fn(handle: u64),
    pub log: unsafe extern "C" fn(level: i32, msg: *const c_char),
}

#[repr(C)]
pub struct NyashMethodInfo {
    pub method_id: u32,
    pub name: *const c_char,
    pub signature: u32,
}

unsafe impl Sync for NyashMethodInfo {}

#[repr(C)]
pub struct NyashPluginInfo {
    pub type_id: u32,
    pub type_name: *const c_char,
    pub method_count: usize,
    pub methods: *const NyashMethodInfo,
}

unsafe impl Sync for NyashPluginInfo {}

// Error codes
const NYB_SUCCESS: i32 = 0;
const NYB_E_INVALID_ARGS: i32 = -4;
const NYB_E_INVALID_HANDLE: i32 = -8;

// Method IDs
const METHOD_BIRTH: u32 = 0;
const METHOD_HELLO: u32 = 1;
const METHOD_GREET: u32 = 2;
const METHOD_FINI: u32 = u32::MAX;

// Type IDs
const TYPE_ID_TESTBOX_A: u32 = 200;
const TYPE_ID_TESTBOX_B: u32 = 201;

// Global state
static mut HOST_VTABLE: Option<&'static NyashHostVtable> = None;
static mut INSTANCES: Option<Mutex<HashMap<u32, TestInstance>>> = None;
static mut INSTANCE_COUNTER: u32 = 1;

enum TestInstance {
    BoxA { message: String },
    BoxB { counter: i32 },
}

// ============ Plugin Info for Each Box Type ============

static TESTBOX_A_NAME: &[u8] = b"TestBoxA\0";
static TESTBOX_B_NAME: &[u8] = b"TestBoxB\0";

static METHOD_HELLO_NAME: &[u8] = b"hello\0";
static METHOD_GREET_NAME: &[u8] = b"greet\0";
static METHOD_BIRTH_NAME: &[u8] = b"birth\0";
static METHOD_FINI_NAME: &[u8] = b"fini\0";

static TESTBOX_A_METHODS: [NyashMethodInfo; 3] = [
    NyashMethodInfo {
        method_id: METHOD_BIRTH,
        name: METHOD_BIRTH_NAME.as_ptr() as *const c_char,
        signature: 0,
    },
    NyashMethodInfo {
        method_id: METHOD_HELLO,
        name: METHOD_HELLO_NAME.as_ptr() as *const c_char,
        signature: 0,
    },
    NyashMethodInfo {
        method_id: METHOD_FINI,
        name: METHOD_FINI_NAME.as_ptr() as *const c_char,
        signature: 0,
    },
];

static TESTBOX_B_METHODS: [NyashMethodInfo; 3] = [
    NyashMethodInfo {
        method_id: METHOD_BIRTH,
        name: METHOD_BIRTH_NAME.as_ptr() as *const c_char,
        signature: 0,
    },
    NyashMethodInfo {
        method_id: METHOD_GREET,
        name: METHOD_GREET_NAME.as_ptr() as *const c_char,
        signature: 0,
    },
    NyashMethodInfo {
        method_id: METHOD_FINI,
        name: METHOD_FINI_NAME.as_ptr() as *const c_char,
        signature: 0,
    },
];

static TESTBOX_A_INFO: NyashPluginInfo = NyashPluginInfo {
    type_id: TYPE_ID_TESTBOX_A,
    type_name: TESTBOX_A_NAME.as_ptr() as *const c_char,
    method_count: 3,
    methods: TESTBOX_A_METHODS.as_ptr(),
};

static TESTBOX_B_INFO: NyashPluginInfo = NyashPluginInfo {
    type_id: TYPE_ID_TESTBOX_B,
    type_name: TESTBOX_B_NAME.as_ptr() as *const c_char,
    method_count: 3,
    methods: TESTBOX_B_METHODS.as_ptr(),
};

// ============ Plugin Entry Points ============

#[no_mangle]
pub extern "C" fn nyash_plugin_abi() -> u32 {
    1  // BID-1 ABI version
}

#[no_mangle]
pub extern "C" fn nyash_plugin_init(
    host: *const NyashHostVtable,
    _info: *mut std::ffi::c_void,  // For v2, we use get_box_info instead
) -> i32 {
    if host.is_null() {
        return NYB_E_INVALID_ARGS;
    }
    
    unsafe {
        HOST_VTABLE = Some(&*host);
        INSTANCES = Some(Mutex::new(HashMap::new()));
        
        // Log initialization
        log_info("Multi-box test plugin initialized");
    }
    
    NYB_SUCCESS
}

// ============ Multi-Box v2 Functions ============

#[no_mangle]
pub extern "C" fn nyash_plugin_get_box_count() -> u32 {
    2  // TestBoxA and TestBoxB
}

#[no_mangle]
pub extern "C" fn nyash_plugin_get_box_info(index: u32) -> *const NyashPluginInfo {
    match index {
        0 => &TESTBOX_A_INFO as *const NyashPluginInfo,
        1 => &TESTBOX_B_INFO as *const NyashPluginInfo,
        _ => ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn nyash_plugin_get_type_id(box_name: *const c_char) -> u32 {
    if box_name.is_null() {
        return 0;
    }
    
    unsafe {
        let name = std::ffi::CStr::from_ptr(box_name).to_string_lossy();
        match name.as_ref() {
            "TestBoxA" => TYPE_ID_TESTBOX_A,
            "TestBoxB" => TYPE_ID_TESTBOX_B,
            _ => 0,
        }
    }
}

// ============ Method Invocation ============

#[no_mangle]
pub extern "C" fn nyash_plugin_invoke(
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    _args: *const u8,
    _args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    unsafe {
        match (type_id, method_id) {
            // TestBoxA methods
            (TYPE_ID_TESTBOX_A, METHOD_BIRTH) => {
                create_instance_a(result, result_len)
            }
            (TYPE_ID_TESTBOX_A, METHOD_HELLO) => {
                hello_method(instance_id, result, result_len)
            }
            (TYPE_ID_TESTBOX_A, METHOD_FINI) => {
                destroy_instance(instance_id)
            }
            
            // TestBoxB methods
            (TYPE_ID_TESTBOX_B, METHOD_BIRTH) => {
                create_instance_b(result, result_len)
            }
            (TYPE_ID_TESTBOX_B, METHOD_GREET) => {
                greet_method(instance_id, result, result_len)
            }
            (TYPE_ID_TESTBOX_B, METHOD_FINI) => {
                destroy_instance(instance_id)
            }
            
            _ => NYB_E_INVALID_ARGS,
        }
    }
}

// ============ Method Implementations ============

unsafe fn create_instance_a(result: *mut u8, result_len: *mut usize) -> i32 {
    if let Some(ref mutex) = INSTANCES {
        if let Ok(mut map) = mutex.lock() {
            let id = INSTANCE_COUNTER;
            INSTANCE_COUNTER += 1;
            
            map.insert(id, TestInstance::BoxA {
                message: "Hello from TestBoxA!".to_string(),
            });
            
            // Return instance ID
            if *result_len >= 4 {
                let bytes = id.to_le_bytes();
                ptr::copy_nonoverlapping(bytes.as_ptr(), result, 4);
                *result_len = 4;
                
                log_info(&format!("Created TestBoxA instance {}", id));
                return NYB_SUCCESS;
            }
        }
    }
    NYB_E_INVALID_ARGS
}

unsafe fn create_instance_b(result: *mut u8, result_len: *mut usize) -> i32 {
    if let Some(ref mutex) = INSTANCES {
        if let Ok(mut map) = mutex.lock() {
            let id = INSTANCE_COUNTER;
            INSTANCE_COUNTER += 1;
            
            map.insert(id, TestInstance::BoxB {
                counter: 0,
            });
            
            // Return instance ID
            if *result_len >= 4 {
                let bytes = id.to_le_bytes();
                ptr::copy_nonoverlapping(bytes.as_ptr(), result, 4);
                *result_len = 4;
                
                log_info(&format!("Created TestBoxB instance {}", id));
                return NYB_SUCCESS;
            }
        }
    }
    NYB_E_INVALID_ARGS
}

unsafe fn hello_method(instance_id: u32, result: *mut u8, result_len: *mut usize) -> i32 {
    if let Some(ref mutex) = INSTANCES {
        if let Ok(map) = mutex.lock() {
            if let Some(TestInstance::BoxA { message }) = map.get(&instance_id) {
                // Return message as TLV string
                write_tlv_string(message, result, result_len)
            } else {
                NYB_E_INVALID_HANDLE
            }
        } else {
            NYB_E_INVALID_ARGS
        }
    } else {
        NYB_E_INVALID_ARGS
    }
}

unsafe fn greet_method(instance_id: u32, result: *mut u8, result_len: *mut usize) -> i32 {
    if let Some(ref mutex) = INSTANCES {
        if let Ok(mut map) = mutex.lock() {
            if let Some(TestInstance::BoxB { counter }) = map.get_mut(&instance_id) {
                *counter += 1;
                let message = format!("Greeting #{} from TestBoxB!", counter);
                
                // Return message as TLV string
                write_tlv_string(&message, result, result_len)
            } else {
                NYB_E_INVALID_HANDLE
            }
        } else {
            NYB_E_INVALID_ARGS
        }
    } else {
        NYB_E_INVALID_ARGS
    }
}

unsafe fn destroy_instance(instance_id: u32) -> i32 {
    if let Some(ref mutex) = INSTANCES {
        if let Ok(mut map) = mutex.lock() {
            if map.remove(&instance_id).is_some() {
                log_info(&format!("Destroyed instance {}", instance_id));
                NYB_SUCCESS
            } else {
                NYB_E_INVALID_HANDLE
            }
        } else {
            NYB_E_INVALID_ARGS
        }
    } else {
        NYB_E_INVALID_ARGS
    }
}

// ============ Helper Functions ============

unsafe fn write_tlv_string(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    let bytes = s.as_bytes();
    let needed = 8 + bytes.len();  // header(4) + entry(4) + string
    
    if *result_len < needed {
        return NYB_E_INVALID_ARGS;
    }
    
    // TLV header
    *result = 1;  // version low
    *result.offset(1) = 0;  // version high
    *result.offset(2) = 1;  // argc low
    *result.offset(3) = 0;  // argc high
    
    // String entry
    *result.offset(4) = 6;  // Tag::String
    *result.offset(5) = 0;  // padding
    let len_bytes = (bytes.len() as u16).to_le_bytes();
    *result.offset(6) = len_bytes[0];
    *result.offset(7) = len_bytes[1];
    
    // String data
    ptr::copy_nonoverlapping(bytes.as_ptr(), result.offset(8), bytes.len());
    
    *result_len = needed;
    NYB_SUCCESS
}

unsafe fn log_info(message: &str) {
    if let Some(vtable) = HOST_VTABLE {
        if let Ok(c_str) = std::ffi::CString::new(message) {
            (vtable.log)(1, c_str.as_ptr());
        }
    }
}

#[no_mangle]
pub extern "C" fn nyash_plugin_shutdown() {
    unsafe {
        log_info("Multi-box test plugin shutting down");
        INSTANCES = None;
        HOST_VTABLE = None;
    }
}