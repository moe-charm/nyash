#[cfg(target_arch = "wasm32")]
pub mod wasm_test {
    use wasm_bindgen::prelude::*;
    use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d};
    
    #[wasm_bindgen]
    pub fn test_direct_canvas_draw() -> Result<(), JsValue> {
        // Get window and document
        let window = window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;
        
        // Get canvas element
        let canvas = document
            .get_element_by_id("test-canvas")
            .ok_or("canvas not found")?
            .dyn_into::<HtmlCanvasElement>()?;
        
        // Set canvas size
        canvas.set_width(400);
        canvas.set_height(300);
        
        // Get 2D context
        let context = canvas
            .get_context("2d")?
            .ok_or("no 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;
        
        // Draw black background
        context.set_fill_style(&JsValue::from_str("black"));
        context.fill_rect(0.0, 0.0, 400.0, 300.0);
        
        // Draw red rectangle
        context.set_fill_style(&JsValue::from_str("red"));
        context.fill_rect(50.0, 50.0, 100.0, 80.0);
        
        // Draw blue circle
        context.set_fill_style(&JsValue::from_str("blue"));
        context.begin_path();
        context.arc(250.0, 100.0, 40.0, 0.0, 2.0 * std::f64::consts::PI)?;
        context.fill();
        
        // Draw text
        context.set_font("20px Arial");
        context.set_fill_style(&JsValue::from_str("white"));
        context.fill_text("Hello Direct Canvas!", 100.0, 200.0)?;
        
        Ok(())
    }
}