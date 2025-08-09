/*!
 * Nyash Sound Box - Simple sound generation
 * 
 * 音響効果を提供するBox型
 * Everything is Box哲学に基づく音響ライブラリ
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox};
use std::fmt::{Debug, Display};
use std::any::Any;
use std::process::Command;
use std::time::Duration;

/// 音響効果を提供するBox
#[derive(Debug, Clone)]
pub struct SoundBox {
    id: u64,
}

impl SoundBox {
    pub fn new() -> Self {
        static mut COUNTER: u64 = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        
        Self { id }
    }
    
    /// ビープ音を鳴らす（基本）
    pub fn beep(&self) -> Box<dyn NyashBox> {
        // 端末ベル文字を出力
        print!("\x07");
        Box::new(StringBox::new("Beep!"))
    }
    
    /// 指定回数ビープ
    pub fn beeps(&self, count: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(count_int) = count.as_any().downcast_ref::<IntegerBox>() {
            if count_int.value <= 0 {
                return Box::new(StringBox::new("Beep count must be positive"));
            }
            
            for i in 0..count_int.value {
                print!("\x07");
                if i < count_int.value - 1 {
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
            
            Box::new(StringBox::new(&format!("Beeped {} times", count_int.value)))
        } else {
            Box::new(StringBox::new("Error: beeps() requires integer input"))
        }
    }
    
    /// 指定周波数のビープ（Linuxのみ）
    pub fn tone(&self, frequency: Box<dyn NyashBox>, duration: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let (Some(freq_int), Some(dur_int)) = (
            frequency.as_any().downcast_ref::<IntegerBox>(),
            duration.as_any().downcast_ref::<IntegerBox>()
        ) {
            if freq_int.value <= 0 || dur_int.value <= 0 {
                return Box::new(StringBox::new("Frequency and duration must be positive"));
            }
            
            // Linuxのbeepコマンドを試行
            match Command::new("beep")
                .arg("-f")
                .arg(&freq_int.value.to_string())
                .arg("-l")
                .arg(&dur_int.value.to_string())
                .output() 
            {
                Ok(_) => Box::new(StringBox::new(&format!("Played {}Hz for {}ms", freq_int.value, dur_int.value))),
                Err(_) => {
                    // beepコマンドが無い場合は端末ベルを使用
                    print!("\x07");
                    std::thread::sleep(Duration::from_millis(dur_int.value as u64));
                    Box::new(StringBox::new(&format!("Fallback beep ({}Hz, {}ms)", freq_int.value, dur_int.value)))
                }
            }
        } else {
            Box::new(StringBox::new("Error: tone() requires two integer inputs (frequency, duration)"))
        }
    }
    
    /// 警告音
    pub fn alert(&self) -> Box<dyn NyashBox> {
        // 3回短いビープ
        for i in 0..3 {
            print!("\x07");
            if i < 2 {
                std::thread::sleep(Duration::from_millis(150));
            }
        }
        Box::new(StringBox::new("Alert sound played"))
    }
    
    /// 成功音
    pub fn success(&self) -> Box<dyn NyashBox> {
        // 1回長めのビープ
        print!("\x07");
        std::thread::sleep(Duration::from_millis(50));
        print!("\x07");
        Box::new(StringBox::new("Success sound played"))
    }
    
    /// エラー音
    pub fn error(&self) -> Box<dyn NyashBox> {
        // 2回素早いビープ
        print!("\x07");
        std::thread::sleep(Duration::from_millis(80));
        print!("\x07");
        Box::new(StringBox::new("Error sound played"))
    }
    
    /// カスタムビープパターン
    pub fn pattern(&self, pattern: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let Some(pattern_str) = pattern.as_any().downcast_ref::<StringBox>() {
            let mut beep_count = 0;
            
            for ch in pattern_str.value.chars() {
                match ch {
                    '.' => {
                        // 短いビープ
                        print!("\x07");
                        std::thread::sleep(Duration::from_millis(100));
                        beep_count += 1;
                    }
                    '-' => {
                        // 長いビープ
                        print!("\x07");
                        std::thread::sleep(Duration::from_millis(300));
                        beep_count += 1;
                    }
                    ' ' => {
                        // 無音（待機）
                        std::thread::sleep(Duration::from_millis(200));
                    }
                    _ => {
                        // その他の文字は無視
                    }
                }
                
                // 文字間の短い間隔
                std::thread::sleep(Duration::from_millis(50));
            }
            
            Box::new(StringBox::new(&format!("Played pattern '{}' ({} beeps)", pattern_str.value, beep_count)))
        } else {
            Box::new(StringBox::new("Error: pattern() requires string input (use '.' for short, '-' for long, ' ' for pause)"))
        }
    }
    
    /// システム音量チェック（簡易）
    pub fn volumeTest(&self) -> Box<dyn NyashBox> {
        print!("\x07");
        Box::new(StringBox::new("Volume test beep - can you hear it?"))
    }
    
    /// 指定間隔でビープ
    pub fn interval(&self, times: Box<dyn NyashBox>, interval_ms: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
        if let (Some(times_int), Some(interval_int)) = (
            times.as_any().downcast_ref::<IntegerBox>(),
            interval_ms.as_any().downcast_ref::<IntegerBox>()
        ) {
            if times_int.value <= 0 || interval_int.value < 0 {
                return Box::new(StringBox::new("Times must be positive, interval must be non-negative"));
            }
            
            for i in 0..times_int.value {
                print!("\x07");
                if i < times_int.value - 1 {
                    std::thread::sleep(Duration::from_millis(interval_int.value as u64));
                }
            }
            
            Box::new(StringBox::new(&format!("Played {} beeps with {}ms intervals", times_int.value, interval_int.value)))
        } else {
            Box::new(StringBox::new("Error: interval() requires two integer inputs (times, interval_ms)"))
        }
    }
}

impl NyashBox for SoundBox {
    fn type_name(&self) -> &'static str {
        "SoundBox"
    }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new("SoundBox()")
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_sound) = other.as_any().downcast_ref::<SoundBox>() {
            BoolBox::new(self.id == other_sound.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn box_id(&self) -> u64 {
        self.id
    }
}

impl Display for SoundBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SoundBox()")
    }
}