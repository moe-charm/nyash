use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr;
use chrono::{DateTime, Utc, Datelike, Timelike};
use rand::Rng;

// MathBox構造体
pub struct MathBox {
    // MathBoxは状態を持たない
}

// RandomBox構造体
pub struct RandomBox {
    rng: rand::rngs::ThreadRng,
}

// TimeBox構造体
pub struct TimeBox {
    // TimeBoxは状態を持たない
}

// DateTimeBox構造体
pub struct DateTimeBox {
    datetime: DateTime<Utc>,
}

// ================== MathBox C API ==================

#[no_mangle]
pub extern "C" fn nyash_math_create() -> *mut c_void {
    let math_box = Box::new(MathBox {});
    Box::into_raw(math_box) as *mut c_void
}

#[no_mangle]
pub extern "C" fn nyash_math_sqrt(value: f64) -> f64 {
    value.sqrt()
}

#[no_mangle]
pub extern "C" fn nyash_math_pow(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}

#[no_mangle]
pub extern "C" fn nyash_math_sin(value: f64) -> f64 {
    value.sin()
}

#[no_mangle]
pub extern "C" fn nyash_math_cos(value: f64) -> f64 {
    value.cos()
}

#[no_mangle]
pub extern "C" fn nyash_math_tan(value: f64) -> f64 {
    value.tan()
}

#[no_mangle]
pub extern "C" fn nyash_math_abs(value: f64) -> f64 {
    value.abs()
}

#[no_mangle]
pub extern "C" fn nyash_math_floor(value: f64) -> f64 {
    value.floor()
}

#[no_mangle]
pub extern "C" fn nyash_math_ceil(value: f64) -> f64 {
    value.ceil()
}

#[no_mangle]
pub extern "C" fn nyash_math_round(value: f64) -> f64 {
    value.round()
}

#[no_mangle]
pub extern "C" fn nyash_math_log(value: f64) -> f64 {
    value.ln()
}

#[no_mangle]
pub extern "C" fn nyash_math_log10(value: f64) -> f64 {
    value.log10()
}

#[no_mangle]
pub extern "C" fn nyash_math_exp(value: f64) -> f64 {
    value.exp()
}

#[no_mangle]
pub extern "C" fn nyash_math_min(a: f64, b: f64) -> f64 {
    a.min(b)
}

#[no_mangle]
pub extern "C" fn nyash_math_max(a: f64, b: f64) -> f64 {
    a.max(b)
}

#[no_mangle]
pub extern "C" fn nyash_math_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr as *mut MathBox);
    }
}

// ================== RandomBox C API ==================

#[no_mangle]
pub extern "C" fn nyash_random_create() -> *mut c_void {
    let random_box = Box::new(RandomBox {
        rng: rand::thread_rng(),
    });
    Box::into_raw(random_box) as *mut c_void
}

#[no_mangle]
pub extern "C" fn nyash_random_next(ptr: *mut c_void) -> f64 {
    if ptr.is_null() {
        return 0.0;
    }
    unsafe {
        let random_box = &mut *(ptr as *mut RandomBox);
        random_box.rng.gen()
    }
}

#[no_mangle]
pub extern "C" fn nyash_random_range(ptr: *mut c_void, min: f64, max: f64) -> f64 {
    if ptr.is_null() {
        return min;
    }
    unsafe {
        let random_box = &mut *(ptr as *mut RandomBox);
        random_box.rng.gen_range(min..=max)
    }
}

#[no_mangle]
pub extern "C" fn nyash_random_int(ptr: *mut c_void, min: i64, max: i64) -> i64 {
    if ptr.is_null() {
        return min;
    }
    unsafe {
        let random_box = &mut *(ptr as *mut RandomBox);
        random_box.rng.gen_range(min..=max)
    }
}

#[no_mangle]
pub extern "C" fn nyash_random_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr as *mut RandomBox);
    }
}

// ================== TimeBox C API ==================

#[no_mangle]
pub extern "C" fn nyash_time_create() -> *mut c_void {
    let time_box = Box::new(TimeBox {});
    Box::into_raw(time_box) as *mut c_void
}

#[no_mangle]
pub extern "C" fn nyash_time_now() -> *mut c_void {
    let datetime = Box::new(DateTimeBox {
        datetime: Utc::now(),
    });
    Box::into_raw(datetime) as *mut c_void
}

#[no_mangle]
pub extern "C" fn nyash_time_parse(time_str: *const c_char) -> *mut c_void {
    if time_str.is_null() {
        return ptr::null_mut();
    }
    
    unsafe {
        let c_str = CStr::from_ptr(time_str);
        if let Ok(rust_str) = c_str.to_str() {
            // ISO 8601形式をパース
            if let Ok(datetime) = rust_str.parse::<DateTime<Utc>>() {
                let datetime_box = Box::new(DateTimeBox { datetime });
                return Box::into_raw(datetime_box) as *mut c_void;
            }
        }
    }
    
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn nyash_time_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr as *mut TimeBox);
    }
}

// ================== DateTimeBox C API ==================

#[no_mangle]
pub extern "C" fn nyash_datetime_to_string(ptr: *mut c_void) -> *mut c_char {
    if ptr.is_null() {
        return ptr::null_mut();
    }
    
    unsafe {
        let datetime_box = &*(ptr as *mut DateTimeBox);
        let datetime_str = datetime_box.datetime.to_rfc3339();
        
        if let Ok(c_string) = CString::new(datetime_str) {
            c_string.into_raw()
        } else {
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn nyash_datetime_year(ptr: *mut c_void) -> i32 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let datetime_box = &*(ptr as *mut DateTimeBox);
        datetime_box.datetime.year()
    }
}

#[no_mangle]
pub extern "C" fn nyash_datetime_month(ptr: *mut c_void) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let datetime_box = &*(ptr as *mut DateTimeBox);
        datetime_box.datetime.month()
    }
}

#[no_mangle]
pub extern "C" fn nyash_datetime_day(ptr: *mut c_void) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let datetime_box = &*(ptr as *mut DateTimeBox);
        datetime_box.datetime.day()
    }
}

#[no_mangle]
pub extern "C" fn nyash_datetime_hour(ptr: *mut c_void) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let datetime_box = &*(ptr as *mut DateTimeBox);
        datetime_box.datetime.hour()
    }
}

#[no_mangle]
pub extern "C" fn nyash_datetime_minute(ptr: *mut c_void) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let datetime_box = &*(ptr as *mut DateTimeBox);
        datetime_box.datetime.minute()
    }
}

#[no_mangle]
pub extern "C" fn nyash_datetime_second(ptr: *mut c_void) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let datetime_box = &*(ptr as *mut DateTimeBox);
        datetime_box.datetime.second()
    }
}

#[no_mangle]
pub extern "C" fn nyash_datetime_timestamp(ptr: *mut c_void) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let datetime_box = &*(ptr as *mut DateTimeBox);
        datetime_box.datetime.timestamp()
    }
}

#[no_mangle]
pub extern "C" fn nyash_datetime_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr as *mut DateTimeBox);
    }
}

#[no_mangle]
pub extern "C" fn nyash_string_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}