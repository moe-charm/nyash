// Windows Icon Extraction Test
// アイコンを実際に取得してICOファイルとして保存するテスト

#[cfg(windows)]
use windows::{
    core::*,
    Win32::{
        Storage::FileSystem::*,
        UI::Shell::*,
        UI::WindowsAndMessaging::*,
        Graphics::Gdi::*,
    },
};

fn main() {
    #[cfg(windows)]
    unsafe {
        println!("Windows Icon Extraction Test");
        
        // C:ドライブのアイコンを取得
        let drive_path = "C:\\";
        let drive_path_wide: Vec<u16> = drive_path.encode_utf16().chain(std::iter::once(0)).collect();
        
        let mut shfi = SHFILEINFOW::default();
        
        let result = SHGetFileInfoW(
            PCWSTR::from_raw(drive_path_wide.as_ptr()),
            FILE_ATTRIBUTE_NORMAL,
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON | SHGFI_USEFILEATTRIBUTES,
        );
        
        println!("SHGetFileInfoW result: {}", result);
        
        if result != 0 && !shfi.hIcon.is_invalid() {
            println!("Icon handle obtained!");
            
            // アイコン情報を取得
            let mut icon_info = ICONINFO::default();
            if GetIconInfo(shfi.hIcon, &mut icon_info).is_ok() {
                println!("GetIconInfo success!");
                println!("fIcon: {}", icon_info.fIcon.as_bool());
                
                // ビットマップ情報を取得
                if !icon_info.hbmColor.is_invalid() {
                    println!("Color bitmap handle obtained!");
                    
                    // ビットマップ情報を取得
                    let mut bitmap = BITMAP::default();
                    let size = GetObjectW(
                        icon_info.hbmColor.into(), 
                        std::mem::size_of::<BITMAP>() as i32,
                        Some(&mut bitmap as *mut _ as *mut _)
                    );
                    
                    if size > 0 {
                        println!("Bitmap info:");
                        println!("  Width: {}", bitmap.bmWidth);
                        println!("  Height: {}", bitmap.bmHeight);
                        println!("  Bits per pixel: {}", bitmap.bmBitsPixel);
                        println!("  Planes: {}", bitmap.bmPlanes);
                        
                        // ピクセルデータを取得
                        let pixel_count = (bitmap.bmWidth * bitmap.bmHeight) as usize;
                        let bytes_per_pixel = (bitmap.bmBitsPixel / 8) as usize;
                        let mut pixels = vec![0u8; pixel_count * bytes_per_pixel];
                        
                        let copied = GetBitmapBits(
                            icon_info.hbmColor,
                            pixels.len() as i32,
                            pixels.as_mut_ptr() as *mut _
                        );
                        
                        println!("Copied {} bytes of pixel data", copied);
                        
                        // 簡易的にBMPファイルとして保存
                        if copied > 0 {
                            save_as_bmp("c_drive_icon.bmp", &pixels, bitmap.bmWidth, bitmap.bmHeight, bitmap.bmBitsPixel);
                            println!("Saved as c_drive_icon.bmp");
                        }
                    }
                    
                    // ビットマップを削除
                    let _ = DeleteObject(icon_info.hbmColor.into());
                }
                
                if !icon_info.hbmMask.is_invalid() {
                    println!("Mask bitmap handle obtained!");
                    let _ = DeleteObject(icon_info.hbmMask.into());
                }
            }
            
            // アイコンを破棄
            let _ = DestroyIcon(shfi.hIcon);
        } else {
            println!("Failed to get icon");
        }
    }
    
    #[cfg(not(windows))]
    println!("This test only works on Windows");
}

#[cfg(windows)]
fn save_as_bmp(filename: &str, pixels: &[u8], width: i32, height: i32, bits_per_pixel: u16) {
    use std::fs::File;
    use std::io::Write;
    
    // 簡易BMPヘッダー（実際の実装はもっと複雑）
    let file_size = 54 + pixels.len() as u32;
    let mut file = File::create(filename).unwrap();
    
    // BMPファイルヘッダー
    file.write_all(b"BM").unwrap(); // マジックナンバー
    file.write_all(&file_size.to_le_bytes()).unwrap();
    file.write_all(&0u32.to_le_bytes()).unwrap(); // 予約
    file.write_all(&54u32.to_le_bytes()).unwrap(); // データオフセット
    
    // BMPインフォヘッダー
    file.write_all(&40u32.to_le_bytes()).unwrap(); // ヘッダーサイズ
    file.write_all(&width.to_le_bytes()).unwrap();
    file.write_all(&height.to_le_bytes()).unwrap();
    file.write_all(&1u16.to_le_bytes()).unwrap(); // プレーン数
    file.write_all(&bits_per_pixel.to_le_bytes()).unwrap();
    file.write_all(&0u32.to_le_bytes()).unwrap(); // 圧縮なし
    file.write_all(&(pixels.len() as u32).to_le_bytes()).unwrap();
    file.write_all(&0i32.to_le_bytes()).unwrap(); // X解像度
    file.write_all(&0i32.to_le_bytes()).unwrap(); // Y解像度
    file.write_all(&0u32.to_le_bytes()).unwrap(); // カラーテーブル数
    file.write_all(&0u32.to_le_bytes()).unwrap(); // 重要な色数
    
    // ピクセルデータ
    file.write_all(pixels).unwrap();
    
    println!("BMP file saved: {}", filename);
}