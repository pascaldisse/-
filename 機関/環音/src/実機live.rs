//! 実機live — 梯4前梯 (live音線のみ). gilrs実機event → wa::z::Z変換器 → 合成器 → cpal連続出力.
//! 「stickを回すと今歌う」— device読取は wa::実機 (環制御 契約層共通law) を再用し,
//! 私有poll再実装はしない. 場注入/haptic は対象外 (触らない).
//!
//! frame境界の総角補間 (欠4是正) は既存 合成器::次sample が既にframe変化検知でやる —
//! liveでは「frame境界」= 共有Zが更新された瞬間になるだけで, 合成側の変更は不要.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use wa::z::{Z変param, Z変換器};
use wa::実機::{device行群, 左stick読取, 接続数, 温機param, 起動温機};

use crate::合成::{合成器, 律動param, 補間param};
use crate::契約::Z;
use crate::音高::{周波数上限param, 音高律};

/// live実行param — main.rs Args から抽出 (全既定はArgs側が持つ — 本structはhardcode禁の
/// 実利: 呼出側が明示的に埋める).
pub struct Liveparam {
    pub sample率: u32,
    pub 秒: f64,
    pub 音高: 音高律,
    pub 律動: 律動param,
    /// z更新率 (Hz) — poll thread周期 & frame長算出の双方に使う既存param (--入力hz 流用).
    pub 入力hz: f64,
    pub deadzone: f64,
    pub 周波数上限比: f64,
    pub 補間標本: Option<usize>,
    pub 温機ms: u64,
    pub 温機poll_ms: u64,
}

pub struct Live結果 {
    pub device行: Vec<String>,
    pub z流入数: usize,
    pub samples: Vec<f32>,
}

/// z毎sample数 (frame長) — sample率/入力hz. 純粋計算部 (test容易).
pub fn z毎sample数(sample率: u32, 入力hz: f64) -> usize {
    (sample率 as f64 / 入力hz).round().max(1.0) as usize
}

/// 必要sample数 — 秒*sample率. 純粋計算部 (test容易).
pub fn 必要sample数(秒: f64, sample率: u32) -> usize {
    (秒 * sample率 as f64).round() as usize
}

struct 共有状態 {
    合成: 合成器,
    現z: Z,
    捕獲: Vec<f32>,
}

/// live実行 — device不在は明確1行errorで即Err (黙fallback禁, 梯1/log/stickへは絶対に落ちない).
/// 実測: 実機接続時のみ本関数を通し実走できる — 実機無し環境での戻り値はUNVERIFIED.
pub fn 実行(p: &Liveparam) -> Result<Live結果, String> {
    if !p.秒.is_finite() || p.秒 <= 0.0 || p.sample率 == 0 || !p.入力hz.is_finite() || p.入力hz <= 0.0 {
        return Err("秒>0・sample率>0・入力hz>0 必須".into());
    }

    let mut g = 起動温機(温機param { 温機ms: p.温機ms, 温機poll_ms: p.温機poll_ms })
        .map_err(|e| format!("device不在: Gilrs起動失敗 ({e:?})"))?;
    if 接続数(&g) == 0 {
        return Err("device不在: 接続gamepad0台 — Bluetooth/ペアリング確認要".into());
    }
    let device行 = device行群(&g);

    let frame長 = z毎sample数(p.sample率, p.入力hz);
    let 必要sample = 必要sample数(p.秒, p.sample率);

    let 合成 = 合成器::新(
        p.sample率,
        p.音高,
        p.律動,
        周波数上限param { 比率: p.周波数上限比 },
        補間param { 標本数: p.補間標本.unwrap_or(frame長) },
    );

    let 共有 = Arc::new(Mutex::new(共有状態 {
        合成,
        現z: Z::無(),
        捕獲: Vec::with_capacity(必要sample),
    }));

    // --- cpal device+stream 準備 (連続出力 — 実音再生の事前描画版とは別経路: 毎sample即時合成) ---
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("出力device無し")?;
    let supported = device.default_output_config().map_err(|e| e.to_string())?;
    let config = cpal::StreamConfig {
        channels: supported.channels(),
        sample_rate: cpal::SampleRate(p.sample率),
        buffer_size: cpal::BufferSize::Default,
    };
    let channels = config.channels as usize;
    let error = |e| eprintln!("# 実機live callback警告: {e}");

    macro_rules! build_stream {
        ($sample_ty:ty, $convert:expr) => {{
            let 共有 = Arc::clone(&共有);
            let convert: fn(f32) -> $sample_ty = $convert;
            device.build_output_stream(
                &config,
                move |data: &mut [$sample_ty], _: &cpal::OutputCallbackInfo| {
                    let mut s = 共有.lock().unwrap();
                    for frame in data.chunks_mut(channels) {
                        let z = s.現z;
                        let v = s.合成.次sample(&z);
                        s.捕獲.push(v);
                        frame.fill(convert(v));
                    }
                },
                error,
                None,
            )
        }};
    }

    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => build_stream!(f32, |v: f32| v),
        cpal::SampleFormat::I16 => {
            build_stream!(i16, |v: f32| (v.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        }
        cpal::SampleFormat::U16 => {
            build_stream!(u16, |v: f32| ((v.clamp(-1.0, 1.0) + 1.0) * 0.5 * u16::MAX as f32) as u16)
        }
        format => return Err(format!("未対応sample format: {format:?}")),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;

    // --- poll thread — wa::実機::左stick読取 再用 (私有再実装禁) → Z変換器 (甲契約, 梯2既存) ---
    let z変param = Z変param { 死域: p.deadzone, ..Default::default() };
    let poll共有 = Arc::clone(&共有);
    let 実行中 = Arc::new(AtomicBool::new(true));
    let poll実行中 = Arc::clone(&実行中);
    let 刻 = Duration::from_secs_f64(1.0 / p.入力hz);
    let z流入数 = Arc::new(AtomicUsize::new(0));
    let poll流入数 = Arc::clone(&z流入数);
    let poll = thread::spawn(move || {
        let mut 変換 = Z変換器::新(z変param);
        let mut 次 = Instant::now();
        while poll実行中.load(Ordering::Relaxed) {
            if let Some((_, x, y)) = 左stick読取(&mut g) {
                let z = 変換.変換(x, y);
                poll共有.lock().unwrap().現z = z;
                poll流入数.fetch_add(1, Ordering::Relaxed);
            }
            次 += 刻;
            let 今 = Instant::now();
            if 次 > 今 {
                thread::sleep((次 - 今).min(刻));
            } else {
                次 = 今;
            }
        }
    });

    thread::sleep(Duration::from_secs_f64(p.秒));
    実行中.store(false, Ordering::Relaxed);
    let _ = poll.join();
    drop(stream);

    let mut samples = {
        let mut s = 共有.lock().unwrap();
        std::mem::take(&mut s.捕獲)
    };
    samples.truncate(必要sample);
    while samples.len() < 必要sample {
        samples.push(0.0);
    }

    Ok(Live結果 {
        device行,
        z流入数: z流入数.load(Ordering::Relaxed),
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame長は既定入力hz六十で分割() {
        assert_eq!(z毎sample数(48_000, 60.0), 800);
    }

    #[test]
    fn frame長は最低一() {
        assert_eq!(z毎sample数(1, 1_000_000.0), 1);
    }

    #[test]
    fn 必要sample数は秒掛sample率() {
        assert_eq!(必要sample数(30.0, 48_000), 1_440_000);
        assert_eq!(必要sample数(8.0, 48_000), 384_000);
    }

    #[test]
    fn 秒零以下はerror() {
        let p = Liveparam {
            sample率: 48_000,
            秒: 0.0,
            音高: 音高律::default(),
            律動: 律動param::default(),
            入力hz: 60.0,
            deadzone: 0.08,
            周波数上限比: 0.45,
            補間標本: None,
            温機ms: 400,
            温機poll_ms: 10,
        };
        assert!(実行(&p).is_err());
    }

    #[test]
    fn 入力hz零以下はerror() {
        let p = Liveparam {
            sample率: 48_000,
            秒: 1.0,
            音高: 音高律::default(),
            律動: 律動param::default(),
            入力hz: 0.0,
            deadzone: 0.08,
            周波数上限比: 0.45,
            補間標本: None,
            温機ms: 400,
            温機poll_ms: 10,
        };
        assert!(実行(&p).is_err());
    }
}
