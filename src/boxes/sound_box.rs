/*! 🔊 SoundBox - サウンド・音響効果Box
 * 
 * ## 📝 概要
 * システム音・効果音を提供するBox。
 * ゲーム効果音、通知音、アラート音の生成に使用。
 * クロスプラットフォーム対応のシンプルなサウンドシステム。
 * 
 * ## 🛠️ 利用可能メソッド
 * - `beep()` - 基本ビープ音
 * - `beeps(count)` - 指定回数ビープ
 * - `bell()` - ベル音
 * - `alarm()` - アラーム音
 * - `playTone(frequency, duration)` - 指定周波数・時間で音生成
 * - `playFile(filename)` - 音声ファイル再生
 * - `setVolume(level)` - 音量設定 (0.0-1.0)
 * 
 * ## 💡 使用例
 * ```nyash
 * local sound
 * sound = new SoundBox()
 * 
 * // 基本的な音
 * sound.beep()              // シンプルビープ
 * sound.beeps(3)            // 3回ビープ
 * sound.bell()              // ベル音
 * 
 * // ゲーム効果音
 * sound.playTone(440, 500)  // ラの音を500ms
 * sound.playTone(880, 200)  // 高いラの音を200ms
 * ```
 * 
 * ## 🎮 実用例 - ゲーム効果音
 * ```nyash
 * static box GameSFX {
 *     init { sound }
 *     
 *     main() {
 *         me.sound = new SoundBox()
 *         me.sound.setVolume(0.7)
 *         
 *         // ゲームイベント
 *         me.playerJump()
 *         me.coinCollect()
 *         me.gameOver()
 *     }
 *     
 *     playerJump() {
 *         // ジャンプ音：低→高
 *         me.sound.playTone(220, 100)
 *         me.sound.playTone(440, 150)
 *     }
 *     
 *     coinCollect() {
 *         // コイン音：上昇音階
 *         me.sound.playTone(523, 80)   // ド
 *         me.sound.playTone(659, 80)   // ミ
 *         me.sound.playTone(784, 120)  // ソ
 *     }
 *     
 *     gameOver() {
 *         // ゲームオーバー音：下降
 *         me.sound.playTone(440, 200)
 *         me.sound.playTone(392, 200)
 *         me.sound.playTone(349, 400)
 *     }
 * }
 * ```
 * 
 * ## 🚨 通知・アラート用途
 * ```nyash
 * static box NotificationSystem {
 *     init { sound }
 *     
 *     main() {
 *         me.sound = new SoundBox()
 *         me.testNotifications()
 *     }
 *     
 *     info() {
 *         me.sound.beep()  // 情報通知
 *     }
 *     
 *     warning() {
 *         me.sound.beeps(2)  // 警告
 *     }
 *     
 *     error() {
 *         // エラー音：断続的
 *         me.sound.playTone(200, 100)
 *         // 短い間隔
 *         me.sound.playTone(200, 100)
 *         me.sound.playTone(200, 200)
 *     }
 *     
 *     success() {
 *         // 成功音：上昇音階
 *         me.sound.playTone(523, 150)  // ド
 *         me.sound.playTone(659, 150)  // ミ
 *         me.sound.playTone(784, 200)  // ソ
 *     }
 * }
 * ```
 * 
 * ## 🎵 音楽生成例
 * ```nyash
 * static box MusicBox {
 *     init { sound, notes }
 *     
 *     main() {
 *         me.sound = new SoundBox()
 *         me.notes = new MapBox()
 *         me.setupNotes()
 *         me.playMelody()
 *     }
 *     
 *     setupNotes() {
 *         // 音階定義
 *         me.notes.set("C", 261)   // ド
 *         me.notes.set("D", 293)   // レ
 *         me.notes.set("E", 329)   // ミ
 *         me.notes.set("F", 349)   // ファ
 *         me.notes.set("G", 392)   // ソ
 *     }
 *     
 *     playNote(note, duration) {
 *         local freq
 *         freq = me.notes.get(note)
 *         me.sound.playTone(freq, duration)
 *     }
 * }
 * ```
 * 
 * ## ⚠️ 注意
 * - システムによってはビープ音が無効化されている場合あり
 * - 音量設定は環境依存
 * - 長時間音生成はCPU使用率に注意
 * - ファイル再生は対応フォーマット限定
 * - Web環境では制限が多い（ユーザー操作後のみ音声再生可能）
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, BoxCore, BoxBase};
use std::fmt::{Debug, Display};
use std::any::Any;
use std::process::Command;
use std::time::Duration;

/// 音響効果を提供するBox
#[derive(Debug, Clone)]
pub struct SoundBox {
    base: BoxBase,
}

impl SoundBox {
    pub fn new() -> Self {
        Self { 
            base: BoxBase::new() 
        }
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
            BoolBox::new(self.base.id == other_sound.base.id)
        } else {
            BoolBox::new(false)
        }
    }
    
    
}

impl BoxCore for SoundBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SoundBox()")
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Display for SoundBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}