use crate::consts::{E_INV_ARGS, E_SHORT, OK};

#[inline]
fn tlv_result_size(payloads: &[(u8, &[u8])]) -> usize {
    4 + payloads.iter().map(|(_, p)| 4 + p.len()).sum::<usize>()
}

#[inline]
pub fn ensure_result_capacity(res: *mut u8, res_len: *mut usize, need: usize) -> Result<(), i32> {
    if res_len.is_null() {
        return Err(E_INV_ARGS);
    }
    unsafe {
        if res.is_null() || *res_len < need {
            *res_len = need;
            return Err(E_SHORT);
        }
    }
    Ok(())
}

#[inline]
unsafe fn write_bytes_unchecked(bytes: &[u8], res: *mut u8, res_len: *mut usize) {
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), res, bytes.len());
    *res_len = bytes.len();
}

pub fn write_u32(v: u32, res: *mut u8, res_len: *mut usize) -> i32 {
    let bytes = v.to_le_bytes();
    if let Err(err) = ensure_result_capacity(res, res_len, bytes.len()) {
        return err;
    }
    unsafe {
        write_bytes_unchecked(&bytes, res, res_len);
    }
    OK
}

pub fn write_tlv_result(payloads: &[(u8, &[u8])], res: *mut u8, res_len: *mut usize) -> i32 {
    let need = tlv_result_size(payloads);
    if let Err(err) = ensure_result_capacity(res, res_len, need) {
        return err;
    }
    let mut buf = Vec::with_capacity(need);
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&(payloads.len() as u16).to_le_bytes());
    for (tag, p) in payloads {
        buf.push(*tag);
        buf.push(0);
        buf.extend_from_slice(&(p.len() as u16).to_le_bytes());
        buf.extend_from_slice(p);
    }
    unsafe {
        write_bytes_unchecked(&buf, res, res_len);
    }
    OK
}

pub fn write_tlv_void(res: *mut u8, res_len: *mut usize) -> i32 {
    write_tlv_result(&[(9u8, &[])], res, res_len)
}
pub fn write_tlv_string(s: &str, res: *mut u8, res_len: *mut usize) -> i32 {
    write_tlv_result(&[(6u8, s.as_bytes())], res, res_len)
}
pub fn write_tlv_bytes(b: &[u8], res: *mut u8, res_len: *mut usize) -> i32 {
    write_tlv_result(&[(7u8, b)], res, res_len)
}
pub fn write_tlv_i32(v: i32, res: *mut u8, res_len: *mut usize) -> i32 {
    write_tlv_result(&[(2u8, &v.to_le_bytes())], res, res_len)
}
pub fn write_tlv_handle(t: u32, id: u32, res: *mut u8, res_len: *mut usize) -> i32 {
    let mut payload = [0u8; 8];
    payload[0..4].copy_from_slice(&t.to_le_bytes());
    payload[4..8].copy_from_slice(&id.to_le_bytes());
    write_tlv_result(&[(8u8, &payload)], res, res_len)
}

pub fn tlv_parse_header(data: &[u8]) -> Result<(u16, u16, usize), ()> {
    if data.len() < 4 {
        return Err(());
    }
    let ver = u16::from_le_bytes([data[0], data[1]]);
    let argc = u16::from_le_bytes([data[2], data[3]]);
    if ver != 1 {
        return Err(());
    }
    Ok((ver, argc, 4))
}
pub fn tlv_parse_string(data: &[u8]) -> Result<String, ()> {
    let (_, argc, pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    let (tag, size, p) = tlv_parse_entry_hdr(data, pos)?;
    if tag != 6 {
        return Err(());
    }
    Ok(std::str::from_utf8(&data[p..p + size])
        .map_err(|_| ())?
        .to_string())
}
pub fn tlv_parse_two_strings(data: &[u8]) -> Result<(String, String), ()> {
    let (_, argc, mut pos) = tlv_parse_header(data)?;
    if argc < 2 {
        return Err(());
    }
    let (tag1, size1, p1) = tlv_parse_entry_hdr(data, pos)?;
    if tag1 != 6 {
        return Err(());
    }
    let s1 = std::str::from_utf8(&data[p1..p1 + size1])
        .map_err(|_| ())?
        .to_string();
    pos = p1 + size1;
    let (tag2, size2, p2) = tlv_parse_entry_hdr(data, pos)?;
    if tag2 != 6 {
        return Err(());
    }
    let s2 = std::str::from_utf8(&data[p2..p2 + size2])
        .map_err(|_| ())?
        .to_string();
    Ok((s1, s2))
}
pub fn tlv_parse_bytes(data: &[u8]) -> Result<Vec<u8>, ()> {
    let (_, argc, pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    let (tag, size, p) = tlv_parse_entry_hdr(data, pos)?;
    if tag != 6 && tag != 7 {
        return Err(());
    }
    Ok(data[p..p + size].to_vec())
}
pub fn tlv_parse_i32(data: &[u8]) -> Result<i32, ()> {
    let (_, argc, pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    let (tag, size, p) = tlv_parse_entry_hdr(data, pos)?;
    match (tag, size) {
        (2, 4) => {
            let mut b = [0u8; 4];
            b.copy_from_slice(&data[p..p + 4]);
            Ok(i32::from_le_bytes(b))
        }
        (5, 8) => {
            let mut b = [0u8; 8];
            b.copy_from_slice(&data[p..p + 8]);
            Ok(i64::from_le_bytes(b) as i32)
        }
        _ => Err(()),
    }
}
pub fn tlv_parse_handle(data: &[u8]) -> Result<(u32, u32), ()> {
    let (_, argc, pos) = tlv_parse_header(data)?;
    if argc < 1 {
        return Err(());
    }
    let (tag, size, p) = tlv_parse_entry_hdr(data, pos)?;
    if tag != 8 || size != 8 {
        return Err(());
    }
    let mut t = [0u8; 4];
    let mut i = [0u8; 4];
    t.copy_from_slice(&data[p..p + 4]);
    i.copy_from_slice(&data[p + 4..p + 8]);
    Ok((u32::from_le_bytes(t), u32::from_le_bytes(i)))
}
pub fn tlv_parse_entry_hdr(data: &[u8], pos: usize) -> Result<(u8, usize, usize), ()> {
    if pos + 4 > data.len() {
        return Err(());
    }
    let tag = data[pos];
    let size = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
    let p = pos + 4;
    if p + size > data.len() {
        return Err(());
    }
    Ok((tag, size, p))
}
