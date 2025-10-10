#![cfg_attr(target_arch = "wasm32", no_std)]

#[cfg(target_arch = "wasm32")]
mod wasm_runtime {
    use core::alloc::{GlobalAlloc, Layout};
    use core::cell::UnsafeCell;

    // Very small bump allocator (not thread-safe, dev-only)
    pub struct WasmAllocator;

    unsafe impl GlobalAlloc for WasmAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            bump_alloc(layout.size(), layout.align())
        }
        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
    }

    #[global_allocator]
    static ALLOCATOR: WasmAllocator = WasmAllocator;

    struct BumpPtr(UnsafeCell<usize>);
    unsafe impl Sync for BumpPtr {}

    // Start with 1 MiB to leave space for stacks/globals; tweak as needed
    static HEAP_PTR: BumpPtr = BumpPtr(UnsafeCell::new(1024 * 1024));

    #[inline]
    unsafe fn bump_alloc(size: usize, align: usize) -> *mut u8 {
        let p = HEAP_PTR.0.get();
        let mut cur = *p;
        // align up
        let aligned = (cur + (align - 1)) & !(align - 1);
        let out = aligned as *mut u8;
        *p = aligned + size;
        out
    }

    #[no_mangle]
    pub extern "C" fn nykernel_malloc(size: i64) -> i64 {
        unsafe {
            let layout = Layout::from_size_align_unchecked(size as usize, 8);
            ALLOCATOR.alloc(layout) as i64
        }
    }

    #[no_mangle]
    pub extern "C" fn nykernel_load_i64(addr: i64) -> i64 {
        unsafe { *(addr as *const i64) }
    }

    #[no_mangle]
    pub extern "C" fn nykernel_store_i64(addr: i64, val: i64) {
        unsafe {
            *(addr as *mut i64) = val;
        }
    }
}

// Non-wasm targets: provide dummies to allow linking tests if needed.
#[cfg(not(target_arch = "wasm32"))]
mod host_stubs {
    #[no_mangle]
    pub extern "C" fn nykernel_malloc(_size: i64) -> i64 {
        0
    }
    #[no_mangle]
    pub extern "C" fn nykernel_load_i64(_addr: i64) -> i64 {
        0
    }
    #[no_mangle]
    pub extern "C" fn nykernel_store_i64(_addr: i64, _val: i64) {}
}
