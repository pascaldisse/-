//! 環音 — z→wav→CoreAudio。stickを回すと歌う梯3実行体。

#[path = "合成.rs"]
mod 合成;
#[path = "契約.rs"]
mod 契約;
#[path = "波形.rs"]
mod 波形;
#[path = "源.rs"]
mod 源;
#[path = "音高.rs"]
mod 音高;

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

use clap::{Parser, ValueEnum};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use 合成::{合成器, 律動param, 補間param};
use 契約::Z;
use 波形::{wav仕様, wav書出};
use 源::log再生;
use 音高::{周波数上限param, 律, 音高律};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum 源選択 {
    #[value(name = "log")]
    Log,
    #[value(name = "stick")]
    Stick,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum 律選択 {
    #[value(name = "八家")]
    八家,
    #[value(name = "十二平均律")]
    十二平均律,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum 切替 {
    #[value(name = "on")]
    On,
    #[value(name = "off")]
    Off,
}

#[derive(Parser, Debug)]
#[command(name = "環音", about = "梯3: z→音。stickを回すと歌う。")]
struct Args {
    #[arg(long, value_enum, default_value_t = 源選択::Log)]
    源: 源選択,
    #[arg(long, default_value_os_t = 既定log())]
    log_path: PathBuf,
    #[arg(long, default_value_t = 220.0)]
    基音: f64,
    #[arg(long, value_enum, default_value_t = 律選択::八家)]
    律: 律選択,
    #[arg(long, value_enum, default_value_t = 切替::On)]
    律動: 切替,
    #[arg(long, default_value_t = 2.0)]
    律動_hz: f64,
    #[arg(long, default_value_t = 48_000)]
    sample率: u32,
    #[arg(long, default_value_t = 8.0)]
    秒: f64,
    #[arg(long, default_value_os_t = 既定wav())]
    wav: PathBuf,
    #[arg(long, value_enum, default_value_t = 切替::On)]
    実音: 切替,
    #[arg(long, value_enum, default_value_t = 切替::On)]
    掃引: 切替,
    #[arg(long, default_value_t = 60.0)]
    入力hz: f64,
    #[arg(long, default_value_t = 0.08)]
    deadzone: f64,
    /// 周波数上限比 (欠3是正) — 上限Hz = sample率 · この比率. 既定0.45 (Nyquist境界0.5に余裕込み).
    /// 監査 docs/adversary/2026-08-11-環統合審.md 乙.4 欠3 参照.
    #[arg(long, default_value_t = 0.45)]
    周波数上限比: f64,
    /// frame間補間標本数 (欠4是正) — 既定=frame長そのもの (未指定時 z毎sample数 を使用).
    #[arg(long)]
    補間標本: Option<usize>,
    /// r直接指定 (B5可測化) — 指定時は源選択を無視し θ=0・lap=0・r=この値 を全z区間へ充てる
    /// (振幅線形性のRMS実測用, 監査 乙.4-d B5 UNVERIFIED是正).
    #[arg(long)]
    径: Option<f64>,
}

fn 既定log() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proof/環制御/入力log.txt")
}

fn 既定wav() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proof/環音/環音.wav")
}

fn 律へ(選択: 律選択) -> 律 {
    match 選択 {
        律選択::八家 => 律::八家,
        律選択::十二平均律 => 律::十二平均律,
    }
}

fn 掃引z(番号: usize, 総数: usize) -> Z {
    let 分母 = 総数.max(1) as f64;
    Z {
        theta: std::f64::consts::TAU * 番号 as f64 / 分母,
        r: 1.0,
        lap: 0,
    }
}

/// 要求時間へz列を揃える。足りない尾部だけ一周掃引、掃引offなら無。
fn 長さを揃える(mut zs: Vec<Z>, 必要: usize, 掃引: bool) -> Vec<Z> {
    zs.truncate(必要);
    while zs.len() < 必要 {
        let 次 = if 掃引 {
            掃引z(zs.len(), 必要)
        } else {
            Z::無()
        };
        zs.push(次);
    }
    zs
}

/// `stick` は梯2公開入力源+Z変換器を通す。実機stream公開口は未提供のためlog再生を使う。
fn stick再生(path: &std::path::Path, deadzone: f64) -> std::io::Result<Vec<Z>> {
    use wa::z::{Z変param, Z変換器};
    let 標本 = wa::入力源::log読込(path)?;
    let mut 変換 = Z変換器::新(Z変param {
        死域: deadzone,
        ..Default::default()
    });
    Ok(標本.into_iter().map(|s| 変換.変換(s.x, s.y)).collect())
}

fn fill_f32(data: &mut [f32], channels: usize, samples: &Arc<Vec<f32>>, index: &AtomicUsize) {
    for frame in data.chunks_mut(channels) {
        let value = samples
            .get(index.fetch_add(1, Ordering::Relaxed))
            .copied()
            .unwrap_or(0.0);
        frame.fill(value);
    }
}

fn fill_i16(data: &mut [i16], channels: usize, samples: &Arc<Vec<f32>>, index: &AtomicUsize) {
    for frame in data.chunks_mut(channels) {
        let value = (samples
            .get(index.fetch_add(1, Ordering::Relaxed))
            .copied()
            .unwrap_or(0.0)
            .clamp(-1.0, 1.0)
            * i16::MAX as f32) as i16;
        frame.fill(value);
    }
}

fn fill_u16(data: &mut [u16], channels: usize, samples: &Arc<Vec<f32>>, index: &AtomicUsize) {
    for frame in data.chunks_mut(channels) {
        let value = ((samples
            .get(index.fetch_add(1, Ordering::Relaxed))
            .copied()
            .unwrap_or(0.0)
            .clamp(-1.0, 1.0)
            + 1.0)
            * 0.5
            * u16::MAX as f32) as u16;
        frame.fill(value);
    }
}

/// CoreAudio実音を試す。device/config不在は警告へ畳み、wav生成を失敗させない。
fn 実音再生(samples: Vec<f32>, sample率: u32) -> Result<(), String> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("出力device無し")?;
    let supported = device.default_output_config().map_err(|e| e.to_string())?;
    let config = cpal::StreamConfig {
        channels: supported.channels(),
        sample_rate: cpal::SampleRate(sample率),
        buffer_size: cpal::BufferSize::Default,
    };
    let channels = config.channels as usize;
    let samples = Arc::new(samples);
    let index = Arc::new(AtomicUsize::new(0));
    let error = |e| eprintln!("# 実音callback警告: {e}");
    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => {
            let samples = Arc::clone(&samples);
            let index = Arc::clone(&index);
            device.build_output_stream(
                &config,
                move |data: &mut [f32], _| fill_f32(data, channels, &samples, &index),
                error,
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let samples = Arc::clone(&samples);
            let index = Arc::clone(&index);
            device.build_output_stream(
                &config,
                move |data: &mut [i16], _| fill_i16(data, channels, &samples, &index),
                error,
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let samples = Arc::clone(&samples);
            let index = Arc::clone(&index);
            device.build_output_stream(
                &config,
                move |data: &mut [u16], _| fill_u16(data, channels, &samples, &index),
                error,
                None,
            )
        }
        format => return Err(format!("未対応sample format: {format:?}")),
    }
    .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    std::thread::sleep(Duration::from_secs_f64(
        samples.len() as f64 / sample率.max(1) as f64,
    ));
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if !args.秒.is_finite()
        || args.秒 <= 0.0
        || args.sample率 == 0
        || !args.入力hz.is_finite()
        || args.入力hz <= 0.0
    {
        return Err("秒>0・sample率>0・入力hz>0 必須".into());
    }
    let 必要z = (args.秒 * args.入力hz).ceil() as usize;
    let zs = match args.源 {
        源選択::Log => log再生(&args.log_path, args.deadzone)?,
        源選択::Stick => {
            eprintln!("# 警告: 実機stream公開口未提供 — 梯2入力logをstick再生へ使用");
            stick再生(&args.log_path, args.deadzone)?
        }
    };
    let zs = 長さを揃える(zs, 必要z, args.掃引 == 切替::On);
    // B5可測化: 径直接指定時は全zのrを上書き (θ/lapは源のまま — 音高は不変で振幅のみ実測する場合は別途θ固定で呼ぶ).
    let zs = if let Some(r) = args.径 {
        zs.into_iter()
            .map(|z| Z {
                theta: z.theta,
                r: r.clamp(0.0, 1.0),
                lap: z.lap,
            })
            .collect()
    } else {
        zs
    };
    let z毎sample数 = (args.sample率 as f64 / args.入力hz).round().max(1.0) as usize;
    let 必要sample = (args.秒 * args.sample率 as f64).round() as usize;
    let mut 合成 = 合成器::新(
        args.sample率,
        音高律 {
            基音: args.基音,
            律: 律へ(args.律),
        },
        律動param {
            有効: args.律動 == 切替::On,
            律動Hz: args.律動_hz,
            ..Default::default()
        },
        周波数上限param {
            比率: args.周波数上限比,
        },
        補間param {
            標本数: args.補間標本.unwrap_or(z毎sample数),
        },
    );
    let mut samples = 合成.描画(&zs, z毎sample数);
    samples.truncate(必要sample);
    while samples.len() < 必要sample {
        samples.push(0.0);
    }
    wav書出(
        &args.wav,
        &wav仕様 {
            sample率: args.sample率,
            ..Default::default()
        },
        &samples,
    )?;
    println!(
        "# wav {} samples={} 秒={:.3}",
        args.wav.display(),
        samples.len(),
        samples.len() as f64 / args.sample率 as f64
    );

    if args.実音 == 切替::On {
        match 実音再生(samples, args.sample率) {
            Ok(()) => println!("# 実音再生: 再生要求完了"),
            Err(e) => eprintln!("# UNVERIFIED 実音再生: {e}"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 掃引補完は要求長() {
        assert_eq!(長さを揃える(Vec::new(), 8, true).len(), 8);
        assert!(長さを揃える(Vec::new(), 8, false).iter().all(Z::無か));
    }

    #[test]
    fn 掃引は一周する() {
        let first = 掃引z(0, 8).theta;
        let last = 掃引z(7, 8).theta;
        assert_eq!(first, 0.0);
        assert!(last > 5.0);
    }
}
