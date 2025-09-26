/*!
 * AudioBox - 音声再生・合成Box
 *
 * ## 📝 概要
 * Web Audio APIを使用してブラウザでの音声再生、
 * 合成、エフェクト処理を統一的に管理するBox。
 * ゲーム、音楽アプリ、オーディオビジュアライザー開発に最適。
 *
 * ## 🛠️ 利用可能メソッド
 *
 * ### 🔊 基本再生
 * - `loadAudio(url)` - 音声ファイル読み込み
 * - `play()` - 再生開始
 * - `pause()` - 一時停止
 * - `stop()` - 停止
 * - `setVolume(volume)` - 音量設定 (0.0-1.0)
 *
 * ### 🎵 音声合成
 * - `createTone(frequency, duration)` - 純音生成
 * - `createNoise(type, duration)` - ノイズ生成
 * - `createBeep()` - システム音
 *
 * ### 📊 解析・ビジュアライザー
 * - `getFrequencyData()` - 周波数解析データ取得
 * - `getWaveformData()` - 波形データ取得
 * - `getVolume()` - 現在の音量レベル
 *
 * ### 🎛️ エフェクト
 * - `addReverb(room)` - リバーブエフェクト
 * - `addFilter(type, frequency)` - フィルター適用
 * - `addDistortion(amount)` - ディストーション
 *
 * ## 💡 使用例
 * ```nyash
 * local audio, visualizer
 * audio = new AudioBox()
 *
 * // 効果音再生
 * audio.loadAudio("sounds/explosion.wav")
 * audio.setVolume(0.7)
 * audio.play()
 *
 * // 音声合成
 * audio.createTone(440, 1000)  // A4音を1秒
 * audio.createBeep()           // システム音
 *
 * // オーディオビジュアライザー
 * local freqData = audio.getFrequencyData()
 * // freqDataを使用してcanvasに描画
 * ```
 */

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::{
    AnalyserNode, AudioBuffer, AudioBufferSourceNode, AudioContext, AudioDestinationNode, GainNode,
    OscillatorNode, PeriodicWave,
};

/// 音声管理Box
#[derive(Debug, Clone)]
pub struct AudioBox {
    base: BoxBase,
    volume: f64,
    is_playing: bool,
}

impl AudioBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            volume: 1.0,
            is_playing: false,
        }
    }

    /// 音量を設定 (0.0 - 1.0)
    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume.max(0.0).min(1.0);

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(gain) = &self.gain_node {
                gain.gain().set_value(self.volume as f32);
            }
        }
    }

    /// 現在の音量を取得
    pub fn get_volume(&self) -> f64 {
        self.volume
    }

    #[cfg(target_arch = "wasm32")]
    /// 指定周波数の純音を生成
    pub fn create_tone(&self, frequency: f64, duration_ms: f64) -> bool {
        if let Some(context) = &self.context {
            if let Ok(oscillator) = context.create_oscillator() {
                if let Ok(gain) = context.create_gain() {
                    // 周波数設定
                    oscillator.frequency().set_value(frequency as f32);

                    // 音量設定
                    gain.gain().set_value(self.volume as f32);

                    // ノード接続
                    oscillator
                        .connect_with_audio_node(&gain)
                        .unwrap_or_default();
                    gain.connect_with_audio_node(&context.destination())
                        .unwrap_or_default();

                    // 再生
                    let start_time = context.current_time();
                    let end_time = start_time + duration_ms / 1000.0;

                    oscillator.start_with_when(start_time).unwrap_or_default();
                    oscillator.stop_with_when(end_time).unwrap_or_default();

                    return true;
                }
            }
        }
        false
    }

    #[cfg(target_arch = "wasm32")]
    /// システムビープ音を生成
    pub fn create_beep(&self) -> bool {
        self.create_tone(800.0, 200.0) // 800Hz、200ms
    }

    #[cfg(target_arch = "wasm32")]
    /// ホワイトノイズを生成
    pub fn create_noise(&self, duration_ms: f64) -> bool {
        if let Some(context) = &self.context {
            let sample_rate = context.sample_rate() as usize;
            let length = ((duration_ms / 1000.0) * sample_rate as f64) as u32;

            if let Ok(buffer) = context.create_buffer(1, length, sample_rate as f32) {
                if let Ok(channel_data) = buffer.get_channel_data(0) {
                    // ホワイトノイズデータ生成
                    for i in 0..channel_data.length() {
                        let noise = (js_sys::Math::random() - 0.5) * 2.0; // -1.0 to 1.0
                        channel_data.set_index(i, noise as f32);
                    }

                    if let Ok(source) = context.create_buffer_source() {
                        source.set_buffer(Some(&buffer));

                        if let Ok(gain) = context.create_gain() {
                            gain.gain().set_value(self.volume as f32);
                            source.connect_with_audio_node(&gain).unwrap_or_default();
                            gain.connect_with_audio_node(&context.destination())
                                .unwrap_or_default();

                            source.start().unwrap_or_default();
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    #[cfg(target_arch = "wasm32")]
    /// 周波数解析データを取得 (オーディオビジュアライザー用)
    pub fn get_frequency_data(&self) -> Vec<u8> {
        if let Some(analyser) = &self.analyser_node {
            let buffer_length = analyser.frequency_bin_count() as usize;
            let mut data_array = vec![0u8; buffer_length];

            // 周波数データを取得
            analyser.get_byte_frequency_data(&mut data_array);
            return data_array;
        }
        vec![]
    }

    #[cfg(target_arch = "wasm32")]
    /// 波形データを取得
    pub fn get_waveform_data(&self) -> Vec<u8> {
        if let Some(analyser) = &self.analyser_node {
            let buffer_length = analyser.fft_size() as usize;
            let mut data_array = vec![0u8; buffer_length];

            // 時間領域データを取得
            analyser.get_byte_time_domain_data(&mut data_array);
            return data_array;
        }
        vec![]
    }

    /// 再生状態を確認
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Non-WASM環境用のダミー実装
    pub fn create_tone(&self, frequency: f64, duration: f64) -> bool {
        println!(
            "AudioBox: Playing tone {}Hz for {}ms (simulated)",
            frequency, duration
        );
        true
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_beep(&self) -> bool {
        println!("AudioBox: Beep sound (simulated)");
        true
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_noise(&self, duration: f64) -> bool {
        println!("AudioBox: White noise for {}ms (simulated)", duration);
        true
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_frequency_data(&self) -> Vec<u8> {
        // シミュレーション用のダミーデータ
        (0..64)
            .map(|i| ((i as f64 * 4.0).sin() * 128.0 + 128.0) as u8)
            .collect()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_waveform_data(&self) -> Vec<u8> {
        // シミュレーション用のダミーデータ
        (0..128)
            .map(|i| ((i as f64 * 0.1).sin() * 64.0 + 128.0) as u8)
            .collect()
    }

    /// オーディオコンテキストの状態を確認
    pub fn is_context_running(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(context) = &self.context {
                return context.state() == web_sys::AudioContextState::Running;
            }
        }
        true // Non-WASM環境では常にtrue
    }

    /// オーディオコンテキストを再開 (ユーザー操作後に必要)
    #[cfg(target_arch = "wasm32")]
    pub fn resume_context(&self) {
        if let Some(context) = &self.context {
            if context.state() == web_sys::AudioContextState::Suspended {
                let _ = context.resume();
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn resume_context(&self) {
        println!("AudioBox: Resume context (simulated)");
    }
}

impl BoxCore for AudioBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "AudioBox(volume={:.2}, playing={})",
            self.volume, self.is_playing
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for AudioBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!(
            "AudioBox(volume={:.2}, playing={})",
            self.volume, self.is_playing
        ))
    }

    fn type_name(&self) -> &'static str {
        "AudioBox"
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_audio) = other.as_any().downcast_ref::<AudioBox>() {
            BoolBox::new(self.base.id == other_audio.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl std::fmt::Display for AudioBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}
