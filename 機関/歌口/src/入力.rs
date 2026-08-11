//! 既定mic→f32単声道ring。標本率は機器既定設定の実値。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SizedSample, Stream, StreamConfig};

#[derive(Debug)]
pub struct 収録結果 {
    pub 標本率: u32,
    pub 標本: Vec<f32>,
}

fn 流<T>(
    機器: &cpal::Device,
    設定: &StreamConfig,
    環: Arc<Mutex<VecDeque<f32>>>,
    容量: usize,
) -> Result<Stream, String>
where
    T: SizedSample,
    f32: cpal::FromSample<T>,
{
    let 道 = 設定.channels as usize;
    機器
        .build_input_stream(
            設定,
            move |列: &[T], _| {
                let Ok(mut ring) = 環.lock() else { return };
                for frame in 列.chunks(道) {
                    let 和: f32 = frame.iter().map(|x| x.to_sample::<f32>()).sum();
                    let 単声 = (和 / frame.len() as f32).clamp(-1.0, 1.0);
                    if ring.len() >= 容量 {
                        ring.pop_front();
                    }
                    ring.push_back(単声);
                }
            },
            |誤| eprintln!("# mic流誤 {誤}"),
            None,
        )
        .map_err(|誤| format!("mic流構築失敗: {誤}"))
}

pub fn mic収録(秒: f64) -> Result<収録結果, String> {
    if !秒.is_finite() || 秒 <= 0.0 {
        return Err("mic秒は正有限値が要る".into());
    }
    let 宿主 = cpal::default_host();
    let 機器 = 宿主.default_input_device().ok_or("既定入力機器なし")?;
    let 支持 = 機器
        .default_input_config()
        .map_err(|誤| format!("入力設定取得失敗: {誤}"))?;
    let 標本率 = 支持.sample_rate().0;
    let 容量 = ((秒 * 標本率 as f64).ceil() as usize).max(1);
    let 環 = Arc::new(Mutex::new(VecDeque::with_capacity(容量)));
    let 設定: StreamConfig = 支持.config();
    let 流 = match 支持.sample_format() {
        SampleFormat::F32 => 流::<f32>(&機器, &設定, Arc::clone(&環), 容量),
        SampleFormat::F64 => 流::<f64>(&機器, &設定, Arc::clone(&環), 容量),
        SampleFormat::I8 => 流::<i8>(&機器, &設定, Arc::clone(&環), 容量),
        SampleFormat::I16 => 流::<i16>(&機器, &設定, Arc::clone(&環), 容量),
        SampleFormat::I32 => 流::<i32>(&機器, &設定, Arc::clone(&環), 容量),
        SampleFormat::I64 => 流::<i64>(&機器, &設定, Arc::clone(&環), 容量),
        SampleFormat::U8 => 流::<u8>(&機器, &設定, Arc::clone(&環), 容量),
        SampleFormat::U16 => 流::<u16>(&機器, &設定, Arc::clone(&環), 容量),
        SampleFormat::U32 => 流::<u32>(&機器, &設定, Arc::clone(&環), 容量),
        SampleFormat::U64 => 流::<u64>(&機器, &設定, Arc::clone(&環), 容量),
        形式 => Err(format!("非対応入力標本形式: {形式}")),
    }?;
    流.play().map_err(|誤| format!("mic流開始失敗: {誤}"))?;
    std::thread::sleep(Duration::from_secs_f64(秒));
    drop(流);
    let 標本 = 環
        .lock()
        .map_err(|_| "mic ring施錠破損".to_string())?
        .iter()
        .copied()
        .collect();
    Ok(収録結果 { 標本率, 標本 })
}
