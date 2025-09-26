/*!
 * QRBox - QRコード生成・読み取りBox
 *
 * ## 📝 概要
 * QRコードの生成、読み取り、カスタマイズを統一的に管理するBox。
 * アプリ間連携、データ共有、認証システムに最適。
 *
 * ## 🛠️ 利用可能メソッド
 *
 * ### 📱 QRコード生成
 * - `generate(text)` - テキストからQRコード生成
 * - `generateURL(url)` - URL用QRコード生成
 * - `generateWiFi(ssid, password, security)` - WiFi設定QR
 * - `generateContact(name, phone, email)` - 連絡先QR
 *
 * ### 🎨 カスタマイズ
 * - `setSize(width, height)` - QRコードサイズ設定
 * - `setColors(fg, bg)` - 前景色・背景色設定
 * - `setLogo(image)` - ロゴ埋め込み
 * - `setErrorCorrection(level)` - エラー訂正レベル
 *
 * ### 📷 読み取り
 * - `scanFromImage(imageData)` - 画像からQR読み取り
 * - `scanFromCanvas(canvas)` - Canvasから読み取り
 * - `startCamera()` - カメラ読み取り開始
 *
 * ### 📊 情報取得
 * - `getDataURL()` - QRコードのData URL取得
 * - `getImageData()` - ImageData形式で取得
 * - `getInfo()` - QRコード情報取得
 *
 * ## 💡 使用例
 * ```nyash
 * local qr, canvas
 * qr = new QRBox()
 * canvas = new WebCanvasBox("qr-canvas", 300, 300)
 *
 * // 基本的なQRコード生成
 * qr.generate("https://nyash-lang.org")
 * qr.setSize(200, 200)
 * qr.setColors("#000000", "#ffffff")
 *
 * // Canvasに描画
 * local imageData = qr.getImageData()
 * canvas.putImageData(imageData, 50, 50)
 *
 * // WiFi設定QR
 * qr.generateWiFi("MyWiFi", "password123", "WPA2")
 * ```
 */

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

/// QRコード管理Box
#[derive(Debug, Clone)]
pub struct QRBox {
    base: BoxBase,
    data: String,
    size: (u32, u32),
    foreground_color: String,
    background_color: String,
    error_correction: String,
    qr_type: String,
}

impl QRBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            data: String::new(),
            size: (200, 200),
            foreground_color: "#000000".to_string(),
            background_color: "#ffffff".to_string(),
            error_correction: "M".to_string(), // L, M, Q, H
            qr_type: "text".to_string(),
        }
    }

    /// テキストからQRコードを生成
    pub fn generate(&mut self, text: &str) -> bool {
        self.data = text.to_string();
        self.qr_type = "text".to_string();
        true
    }

    /// URL用QRコードを生成
    pub fn generate_url(&mut self, url: &str) -> bool {
        if url.starts_with("http://") || url.starts_with("https://") {
            self.data = url.to_string();
            self.qr_type = "url".to_string();
            true
        } else {
            false
        }
    }

    /// WiFi設定QRコードを生成
    pub fn generate_wifi(&mut self, ssid: &str, password: &str, security: &str) -> bool {
        // WiFi QRコード形式: WIFI:T:WPA;S:mynetwork;P:mypass;H:false;;
        let wifi_string = format!("WIFI:T:{};S:{};P:{};H:false;;", security, ssid, password);
        self.data = wifi_string;
        self.qr_type = "wifi".to_string();
        true
    }

    /// 連絡先QRコードを生成
    pub fn generate_contact(&mut self, name: &str, phone: &str, email: &str) -> bool {
        // vCard形式
        let vcard = format!(
            "BEGIN:VCARD\nVERSION:3.0\nFN:{}\nTEL:{}\nEMAIL:{}\nEND:VCARD",
            name, phone, email
        );
        self.data = vcard;
        self.qr_type = "contact".to_string();
        true
    }

    /// QRコードサイズを設定
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.size = (width, height);
    }

    /// 色を設定
    pub fn set_colors(&mut self, foreground: &str, background: &str) {
        self.foreground_color = foreground.to_string();
        self.background_color = background.to_string();
    }

    /// エラー訂正レベルを設定
    pub fn set_error_correction(&mut self, level: &str) {
        if ["L", "M", "Q", "H"].contains(&level) {
            self.error_correction = level.to_string();
        }
    }

    /// QRコードの情報を取得
    pub fn get_info(&self) -> String {
        format!(
            "Type: {}, Size: {}x{}, Error Correction: {}, Data Length: {}",
            self.qr_type,
            self.size.0,
            self.size.1,
            self.error_correction,
            self.data.len()
        )
    }

    /// データURL形式で取得 (簡易実装)
    pub fn get_data_url(&self) -> String {
        format!("data:image/png;base64,{}", self.generate_base64_qr())
    }

    /// 簡易QRコード生成 (実際の実装では専用ライブラリを使用)
    fn generate_base64_qr(&self) -> String {
        // これは簡略化された実装です
        // 実際のプロダクションでは qrcode クレートなどを使用
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==".to_string()
    }

    #[cfg(target_arch = "wasm32")]
    /// CanvasにQRコードを描画
    pub fn draw_to_canvas(&self, canvas_id: &str) -> bool {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(canvas_element) = document.get_element_by_id(canvas_id) {
                    if let Ok(canvas) = canvas_element.dyn_into::<HtmlCanvasElement>() {
                        if let Ok(context) = canvas.get_context("2d") {
                            if let Ok(ctx) = context.unwrap().dyn_into::<CanvasRenderingContext2d>()
                            {
                                return self.draw_simple_qr(&ctx);
                            }
                        }
                    }
                }
            }
        }
        false
    }

    #[cfg(target_arch = "wasm32")]
    /// 簡易QRコード描画 (デモ用)
    fn draw_simple_qr(&self, ctx: &CanvasRenderingContext2d) -> bool {
        let module_size = 8;
        let modules = 25; // 25x25のQRコード

        // 背景を描画
        ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(&self.background_color));
        ctx.fill_rect(0.0, 0.0, self.size.0 as f64, self.size.1 as f64);

        // QRコードパターンを生成（簡易版）
        ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(&self.foreground_color));

        // データベースの簡単なハッシュを作成
        let hash = self.simple_hash(&self.data);

        for y in 0..modules {
            for x in 0..modules {
                // ファインダーパターンの描画
                if (x < 7 && y < 7) || (x >= modules - 7 && y < 7) || (x < 7 && y >= modules - 7) {
                    if (x == 0 || x == 6 || y == 0 || y == 6)
                        || (x >= 2 && x <= 4 && y >= 2 && y <= 4)
                    {
                        ctx.fill_rect(
                            (x * module_size) as f64,
                            (y * module_size) as f64,
                            module_size as f64,
                            module_size as f64,
                        );
                    }
                } else {
                    // データパターン（簡易実装）
                    let bit = (hash >> ((x + y * modules) % 32)) & 1;
                    if bit == 1 {
                        ctx.fill_rect(
                            (x * module_size) as f64,
                            (y * module_size) as f64,
                            module_size as f64,
                            module_size as f64,
                        );
                    }
                }
            }
        }

        true
    }

    /// 簡単なハッシュ関数（デモ用）
    #[allow(dead_code)]
    fn simple_hash(&self, data: &str) -> u32 {
        let mut hash = 5381u32;
        for byte in data.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
        }
        hash
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Non-WASM環境用のダミー実装
    pub fn draw_to_canvas(&self, canvas_id: &str) -> bool {
        println!(
            "QRBox: Drawing QR code to canvas '{}' (simulated)",
            canvas_id
        );
        println!("  Data: {}", self.data);
        println!("  Size: {}x{}", self.size.0, self.size.1);
        println!(
            "  Colors: {} on {}",
            self.foreground_color, self.background_color
        );
        true
    }

    /// QRコードスキャン（簡易実装）
    #[cfg(target_arch = "wasm32")]
    pub fn scan_from_canvas(&self, canvas_id: &str) -> Option<String> {
        // 実際の実装では画像解析ライブラリを使用
        println!("QRBox: Scanning from canvas '{}' (simulated)", canvas_id);
        Some("scanned_data_placeholder".to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn scan_from_canvas(&self, canvas_id: &str) -> Option<String> {
        println!("QRBox: Scanning from canvas '{}' (simulated)", canvas_id);
        Some("scanned_data_placeholder".to_string())
    }

    /// バッチ生成機能
    pub fn generate_batch(&self, data_list: &[String]) -> Vec<String> {
        data_list
            .iter()
            .map(|data| format!("QR for: {}", data))
            .collect()
    }

    /// QRコードの複雑度を計算
    pub fn calculate_complexity(&self) -> u32 {
        let data_len = self.data.len() as u32;
        let base_complexity = match self.error_correction.as_str() {
            "L" => 1,
            "M" => 2,
            "Q" => 3,
            "H" => 4,
            _ => 2,
        };

        data_len * base_complexity
    }
}

impl BoxCore for QRBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "QRBox(type={}, size={}x{})",
            self.qr_type, self.size.0, self.size.1
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for QRBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!(
            "QRBox(type={}, size={}x{})",
            self.qr_type, self.size.0, self.size.1
        ))
    }

    fn type_name(&self) -> &'static str {
        "QRBox"
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_qr) = other.as_any().downcast_ref::<QRBox>() {
            BoolBox::new(self.base.id == other_qr.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for QRBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}
