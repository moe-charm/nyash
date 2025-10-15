//! Nyash EguiBox Plugin (TypeBox ABI skeleton)
//! - Provides a minimal window/UI placeholder via Nyash ABI
//! - Windows GUI integration (egui/eframe) can be enabled later via `with-egui` feature

use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
};

// ===== Error/Status codes (BID-FFI v1 aligned) =====
const OK: i32 = 0;
const E_SHORT: i32 = -1;
const E_TYPE: i32 = -2;
const E_METHOD: i32 = -3;
const E_ARGS: i32 = -4;
const E_FAIL: i32 = -5;

// ===== IDs =====
const TID_EGUI: u32 = 70; // match nyash.toml [box_types]

// methods
const M_BIRTH: u32 = 0;
const M_OPEN: u32 = 1; // open(width:int, height:int, title:str)
const M_UI_LABEL: u32 = 2; // uiLabel(text:str)
const M_UI_BUTTON: u32 = 3; // uiButton(text:str) -> future: events
const M_POLL_EVENT: u32 = 4; // pollEvent() -> Result.Ok(text) / Result.Err("none")
const M_RUN: u32 = 5; // run() -> enters loop or no-op
const M_CLOSE: u32 = 6; // close()
const M_FINI: u32 = u32::MAX;

#[derive(Default)]
struct EguiInstance {
    width: i32,
    height: i32,
    title: String,
    labels: Vec<String>,
}

static INST: Lazy<Mutex<HashMap<u32, EguiInstance>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

// ===== TypeBox ABI (resolve/invoke_id) =====
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,
    pub version: u16,
    pub struct_size: u16,
    pub name: *const std::os::raw::c_char,
    pub resolve: Option<extern "C" fn(*const std::os::raw::c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}

// This simple POD struct contains raw pointers; mark as Sync for static export
unsafe impl Sync for NyashTypeBoxFfi {}

const ABI_TAG: u32 = 0x58594254; // 'T''Y''B''X' little-endian (TYBX)

extern "C" fn tb_resolve(name: *const std::os::raw::c_char) -> u32 {
    unsafe {
        if name.is_null() {
            return 0;
        }
        let s = std::ffi::CStr::from_ptr(name).to_string_lossy();
        match s.as_ref() {
            "birth" => M_BIRTH,
            "open" => M_OPEN,
            "uiLabel" => M_UI_LABEL,
            "uiButton" => M_UI_BUTTON,
            "pollEvent" => M_POLL_EVENT,
            "run" => M_RUN,
            "close" => M_CLOSE,
            "fini" => M_FINI,
            _ => 0,
        }
    }
}

extern "C" fn tb_invoke_id(
    _method_id: u32,
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    nyash_plugin_invoke(
        TID_EGUI,
        _method_id,
        instance_id,
        args,
        args_len,
        result,
        result_len,
    )
}

static TYPE_NAME: &[u8] = b"EguiBox\0";
#[no_mangle]
pub static nyash_typebox_EguiBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
    abi_tag: ABI_TAG,
    version: 1,
    struct_size: std::mem::size_of::<NyashTypeBoxFfi>() as u16,
    name: TYPE_NAME.as_ptr() as *const _,
    resolve: Some(tb_resolve),
    invoke_id: Some(tb_invoke_id),
    capabilities: 0,
};

// legacy v1 abi/init removed

// ===== TLV helpers (version=1) =====
fn write_tlv_result(payloads: &[(u8, &[u8])], result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return E_ARGS;
    }
    let mut buf: Vec<u8> =
        Vec::with_capacity(4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes());
    for (tag, payload) in payloads {
        buf.push(*tag);
        buf.push(0);
        buf.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        buf.extend_from_slice(payload);
    }
    unsafe {
        let need = buf.len();
        if result.is_null() || *result_len < need {
            *result_len = need;
            return E_SHORT;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, need);
        *result_len = need;
    }
    OK
}
fn write_tlv_void(result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(9u8, &[])], result, result_len)
}
fn write_tlv_string(s: &str, result: *mut u8, result_len: *mut usize) -> i32 {
    write_tlv_result(&[(6u8, s.as_bytes())], result, result_len)
}

unsafe fn tlv_parse_header(data: *const u8, len: usize) -> Option<(u16, u16, usize)> {
    if data.is_null() || len < 4 {
        return None;
    }
    let b = std::slice::from_raw_parts(data, len);
    let ver = u16::from_le_bytes([b[0], b[1]]);
    let argc = u16::from_le_bytes([b[2], b[3]]);
    if ver != 1 {
        return None;
    }
    Some((ver, argc, 4))
}
unsafe fn tlv_read_entry_at(
    data: *const u8,
    len: usize,
    mut pos: usize,
) -> Option<(u8, usize, usize)> {
    let b = std::slice::from_raw_parts(data, len);
    if pos + 4 > len {
        return None;
    }
    let tag = b[pos];
    let _ = b[pos + 1];
    let size = u16::from_le_bytes([b[pos + 2], b[pos + 3]]) as usize;
    pos += 4;
    if pos + size > len {
        return None;
    }
    Some((tag, size, pos))
}
unsafe fn tlv_read_i64(data: *const u8, len: usize, index: usize) -> Option<i64> {
    let (_, argc, mut pos) = tlv_parse_header(data, len)?;
    if argc < (index as u16 + 1) {
        return None;
    }
    for i in 0..=index {
        let (tag, size, p) = tlv_read_entry_at(data, len, pos)?;
        if tag == 3 && size == 8 {
            if i == index {
                let b = std::slice::from_raw_parts(data.add(p), 8);
                let mut t = [0u8; 8];
                t.copy_from_slice(b);
                return Some(i64::from_le_bytes(t));
            }
        }
        pos = p + size;
    }
    None
}
unsafe fn tlv_read_string(data: *const u8, len: usize, index: usize) -> Option<String> {
    let (_, argc, mut pos) = tlv_parse_header(data, len)?;
    if argc < (index as u16 + 1) {
        return None;
    }
    for i in 0..=index {
        let (tag, size, p) = tlv_read_entry_at(data, len, pos)?;
        if tag == 6 || tag == 7 {
            if i == index {
                let s = std::slice::from_raw_parts(data.add(p), size);
                return Some(String::from_utf8_lossy(s).to_string());
            }
        }
        pos = p + size;
    }
    None
}
unsafe fn tlv_read_open_args(args: *const u8, len: usize) -> Option<(i32, i32, String)> {
    let w = tlv_read_i64(args, len, 0)? as i32;
    let h = tlv_read_i64(args, len, 1)? as i32;
    let t = tlv_read_string(args, len, 2)?;
    Some((w, h, t))
}

// ===== GUI 実行（with-egui, クロスプラットフォーム） =====
#[cfg(feature = "with-egui")]
mod guirun {
    use super::*;
    use eframe::egui;

    pub fn run_window(w: i32, h: i32, title: &str, labels: Vec<String>) {
        eprintln!("[EGUI] run_window: w={} h={} title='{}'", w, h, title);
        let diag = std::env::var("NYASH_EGUI_DIAG").ok().as_deref() == Some("1");
        let scale_override = std::env::var("NYASH_EGUI_SCALE")
            .ok()
            .and_then(|s| s.parse::<f32>().ok());
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([w.max(100) as f32, h.max(100) as f32])
                .with_title(title.to_string()),
            ..Default::default()
        };

        struct App {
            labels: Vec<String>,
            diag: bool,
            printed: bool,
            init_w: i32,
            init_h: i32,
            scale: Option<f32>,
        }
        impl eframe::App for App {
            fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
                if self.diag && !self.printed {
                    let ppp = ctx.pixels_per_point();
                    let rect = ctx.screen_rect();
                    let (lw, lh) = (rect.width(), rect.height());
                    let (pw, ph) = ((lw * ppp).round() as i32, (lh * ppp).round() as i32);
                    eprintln!("[EGUI][diag] ppp={:.3} logical={:.1}x{:.1} physical={}x{} init={}x{} scale_override={}",
                        ppp, lw, lh, pw, ph, self.init_w, self.init_h,
                        self.scale.map(|v| format!("{:.3}", v)).unwrap_or_else(|| "<none>".into())
                    );
                    self.printed = true;
                }
                egui::TopBottomPanel::top("diag_bar").show(ctx, |ui| {
                    if self.diag {
                        let ppp = ctx.pixels_per_point();
                        let rect = ctx.screen_rect();
                        ui.small(format!(
                            "DPI: ppp={:.3} logical={:.1}x{:.1} init={}x{} scale={}",
                            ppp,
                            rect.width(),
                            rect.height(),
                            self.init_w,
                            self.init_h,
                            self.scale
                                .map(|v| format!("{:.2}", v))
                                .unwrap_or_else(|| "auto".into())
                        ));
                    }
                });
                egui::CentralPanel::default().show(ctx, |ui| {
                    for s in &self.labels {
                        ui.label(s);
                    }
                });
            }
        }

        let res = eframe::run_native(
            title,
            options,
            Box::new({
                let labels = labels;
                let diag = diag;
                let init_w = w;
                let init_h = h;
                let scale_override = scale_override;
                move |cc| {
                    if let Some(ppp) = scale_override {
                        cc.egui_ctx.set_pixels_per_point(ppp);
                        eprintln!("[EGUI][diag] override pixels_per_point to {:.3}", ppp);
                    }
                    Box::new(App {
                        labels,
                        diag,
                        printed: false,
                        init_w,
                        init_h,
                        scale: scale_override,
                    })
                }
            }),
        );
        eprintln!("[EGUI] run_native returned: {:?}", res);
    }
}
