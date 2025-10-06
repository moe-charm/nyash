//! nykernel_stub.rs — Dev-only kernel stub (malloc/load/store)
//!
//! Centralizes the tiny integer heap used by nykernel.* extern calls
//! so both VM extern adapter and plugin loader can share the same logic.
//!
//! Behavior (dev-only):
//! - Enabled when `NYASH_ENABLE_NYKERNEL_STUB=1`
//! - 8-byte aligned addresses
//! - `store_i64` auto-resizes heap to fit (development convenience)
//! - Returns/accepts i64 values only

use std::sync::{Mutex, OnceLock};

use crate::backend::vm_types::{VMError, VMValue};

fn heap_state() -> &'static (Mutex<Vec<i64>>, Mutex<i64>) {
    static HEAP: OnceLock<(Mutex<Vec<i64>>, Mutex<i64>)> = OnceLock::new();
    HEAP.get_or_init(|| (Mutex::new(Vec::with_capacity(1024)), Mutex::new(1)))
}

fn enabled() -> bool {
    std::env::var("NYASH_ENABLE_NYKERNEL_STUB").ok().as_deref() == Some("1")
}

pub fn malloc_bytes(size: i64) -> Result<VMValue, VMError> {
    if !enabled() { return Err(VMError::InvalidInstruction("nykernel.malloc disabled".into())); }
    let cells = ((size + 7) / 8).max(0) as i64;
    let (heap_lock, ptr_lock) = heap_state();
    let mut heap = heap_lock.lock().unwrap();
    let mut ptr = ptr_lock.lock().unwrap();
    let base_cells = *ptr;
    let need = base_cells + cells;
    if need as usize > heap.len() { heap.resize(need as usize, 0); }
    *ptr = need;
    let addr_bytes = base_cells * 8;
    Ok(VMValue::Integer(addr_bytes))
}

pub fn load_i64(addr: i64) -> Result<VMValue, VMError> {
    if !enabled() { return Err(VMError::InvalidInstruction("nykernel.load_i64 disabled".into())); }
    if addr % 8 != 0 || addr < 0 { return Err(VMError::InvalidInstruction("unaligned/neg addr".into())); }
    let index = (addr / 8) as usize;
    let (heap_lock, _) = heap_state();
    let heap = heap_lock.lock().unwrap();
    let v = *heap.get(index).ok_or(VMError::InvalidInstruction("oob".into()))?;
    Ok(VMValue::Integer(v))
}

pub fn store_i64(addr: i64, val: i64) -> Result<VMValue, VMError> {
    if !enabled() { return Err(VMError::InvalidInstruction("nykernel.store_i64 disabled".into())); }
    if addr % 8 != 0 || addr < 0 { return Err(VMError::InvalidInstruction("unaligned/neg addr".into())); }
    let index = (addr / 8) as usize;
    let (heap_lock, _) = heap_state();
    let mut heap = heap_lock.lock().unwrap();
    if index >= heap.len() { heap.resize(index + 1, 0); }
    heap[index] = val;
    Ok(VMValue::Void)
}

pub fn vmvalue_to_i64(v: &VMValue) -> i64 {
    match v {
        VMValue::Integer(i) => *i,
        VMValue::Float(f) => *f as i64,
        VMValue::String(s) => s.parse::<i64>().unwrap_or(0),
        VMValue::Bool(b) => if *b { 1 } else { 0 },
        _ => 0,
    }
}
