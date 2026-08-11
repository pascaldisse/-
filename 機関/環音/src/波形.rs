//! 波形 — wav書出/読取 + 音響解析 (実効値・支配周波数・無音判定).
//! 契約: proof/環音/ 配下へ書く想定 (呼出側がpath決定, 本fileはIO実装のみ).

use std::path::Path;

/// wav仕様 — sample率[Hz]・bit深[bit]. 既定=48000/16.
#[derive(Debug, Clone, Copy)]
pub struct wav仕様 {
    pub sample率: u32,
    pub bit深: u16,
}

impl Default for wav仕様 {
    fn default() -> Self {
        Self { sample率: 48000, bit深: 16 }
    }
}

/// wav書出 — f32 samples [-1,1] を i16 PCM へ変換し書く. clip防止 (飽和).
/// 親dirが無ければ自動作成. samples空なら0長wav.
pub fn wav書出(path: &Path, 仕様: &wav仕様, samples: &[f32]) -> std::io::Result<()> {
    if let Some(親) = path.parent() {
        if !親.as_os_str().is_empty() {
            std::fs::create_dir_all(親)?;
        }
    }
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 仕様.sample率,
        bits_per_sample: 仕様.bit深,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(hound_err_変換)?;
    let 最大: f32 = ((1i32 << (仕様.bit深 - 1)) - 1) as f32; // 16bit→32767
    for &s in samples {
        let clip = s.clamp(-1.0, 1.0); // 飽和 (clip防止)
        let i = (clip * 最大).round() as i32;
        let i = i.clamp(-(最大 as i32) - 1, 最大 as i32);
        writer.write_sample(i as i16).map_err(hound_err_変換)?;
    }
    writer.finalize().map_err(hound_err_変換)?;
    Ok(())
}

/// wav読取 — path → (仕様, samples f32[-1,1]).
pub fn wav読取(path: &Path) -> std::io::Result<(wav仕様, Vec<f32>)> {
    let mut reader = hound::WavReader::open(path).map_err(hound_err_変換)?;
    let spec = reader.spec();
    let 仕様 = wav仕様 { sample率: spec.sample_rate, bit深: spec.bits_per_sample };
    let 最大: f32 = ((1i32 << (仕様.bit深 - 1)) - 1) as f32;
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => reader
            .samples::<i32>()
            .map(|r| r.map(|v| v as f32 / 最大))
            .collect::<Result<_, _>>()
            .map_err(hound_err_変換)?,
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<Result<_, _>>()
            .map_err(hound_err_変換)?,
    };
    Ok((仕様, samples))
}

fn hound_err_変換(e: hound::Error) -> std::io::Error {
    match e {
        hound::Error::IoError(io) => io,
        other => std::io::Error::new(std::io::ErrorKind::Other, other.to_string()),
    }
}

/// 実効値 (RMS) — 空slice→0.0.
pub fn 実効値(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let 二乗和: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (二乗和 / samples.len() as f64).sqrt()
}

/// 無音か — RMS<閾.
pub fn 無音か(samples: &[f32], 閾: f64) -> bool {
    実効値(samples) < 閾
}

/// 支配周波数 — Goertzelアルゴリズムで候補周波数群を走査し, 応答最大の周波数を返す.
/// 単一sine波の基本周波数推定. 空slice或sample率0→0.0.
///
/// 探索範囲 min_hz..max_hz を step_hz刻みで粗探索した後, 最良点周辺を細分探索する
/// 二段Goertzel (param既定: 20Hz-8000Hz, 1Hz刻み → 精度<1Hz).
pub fn 支配周波数(samples: &[f32], sample率: u32) -> f64 {
    支配周波数_探索(samples, sample率, 20.0, 8000.0, 1.0)
}

/// 支配周波数_探索 — 範囲・刻み幅を指定できる版 (paramなし既定のhardcode回避).
pub fn 支配周波数_探索(
    samples: &[f32],
    sample率: u32,
    min_hz: f64,
    max_hz: f64,
    step_hz: f64,
) -> f64 {
    if samples.is_empty() || sample率 == 0 || step_hz <= 0.0 || max_hz <= min_hz {
        return 0.0;
    }
    let n = samples.len();
    let sr = sample率 as f64;
    let 粗刻み = step_hz;
    let mut 最良周波数 = min_hz;
    let mut 最良応答 = f64::MIN;

    let mut freq = min_hz;
    while freq <= max_hz {
        let 応答 = goertzel応答(samples, n, sr, freq);
        if 応答 > 最良応答 {
            最良応答 = 応答;
            最良周波数 = freq;
        }
        freq += 粗刻み;
    }

    // 細分探索 (最良点±粗刻み を 0.1Hz刻みで再走査 → <1Hz精度)
    let 細刻み = (粗刻み / 10.0).max(0.01);
    let mut fine = (最良周波数 - 粗刻み).max(min_hz);
    let fine_max = (最良周波数 + 粗刻み).min(max_hz);
    while fine <= fine_max {
        let 応答 = goertzel応答(samples, n, sr, fine);
        if 応答 > 最良応答 {
            最良応答 = 応答;
            最良周波数 = fine;
        }
        fine += 細刻み;
    }

    最良周波数
}

/// goertzel応答 — 指定周波数のGoertzelパワー応答.
fn goertzel応答(samples: &[f32], n: usize, sample率: f64, freq_hz: f64) -> f64 {
    let k = freq_hz * n as f64 / sample率;
    let omega = std::f64::consts::TAU * k / n as f64;
    let coeff = 2.0 * omega.cos();
    let (mut s_prev, mut s_prev2) = (0.0f64, 0.0f64);
    for &x in samples {
        let s = x as f64 + coeff * s_prev - s_prev2;
        s_prev2 = s_prev;
        s_prev = s;
    }
    s_prev2 * s_prev2 + s_prev * s_prev - coeff * s_prev * s_prev2
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::TAU;
    use std::path::PathBuf;

    fn 出力先(名: &str) -> PathBuf {
        // 殿内 機関/環音/target/試験出力/ — /tmp禁.
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("target");
        p.push("試験出力");
        std::fs::create_dir_all(&p).unwrap();
        p.push(名);
        p
    }

    fn sine合成(freq: f64, sample率: u32, 秒: f64, amp: f32) -> Vec<f32> {
        let n = (sample率 as f64 * 秒) as usize;
        (0..n)
            .map(|i| {
                let t = i as f64 / sample率 as f64;
                (amp as f64 * (TAU * freq * t).sin()) as f32
            })
            .collect()
    }

    #[test]
    fn 仕様既定確認() {
        let 仕様 = wav仕様::default();
        assert_eq!(仕様.sample率, 48000);
        assert_eq!(仕様.bit深, 16);
    }

    #[test]
    fn round_trip_長さと誤差() {
        let path = 出力先("roundtrip.wav");
        let 仕様 = wav仕様::default();
        let samples = sine合成(440.0, 仕様.sample率, 0.2, 0.5);
        wav書出(&path, &仕様, &samples).unwrap();
        let (読仕様, 読samples) = wav読取(&path).unwrap();
        assert_eq!(読仕様.sample率, 仕様.sample率);
        assert_eq!(読仕様.bit深, 仕様.bit深);
        assert_eq!(読samples.len(), samples.len());
        let 許容 = 1.0 / 32767.0 * 2.0;
        for (a, b) in samples.iter().zip(読samples.iter()) {
            assert!((a - b).abs() <= 許容 as f32, "誤差過大: {} vs {}", a, b);
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn 支配周波数_440hz() {
        let path = 出力先("sine440.wav");
        let 仕様 = wav仕様::default();
        let samples = sine合成(440.0, 仕様.sample率, 0.5, 0.8);
        wav書出(&path, &仕様, &samples).unwrap();
        let (読仕様, 読samples) = wav読取(&path).unwrap();
        let 周波数 = 支配周波数(&読samples, 読仕様.sample率);
        assert!((周波数 - 440.0).abs() < 1.0, "440Hz推定誤差: {}", 周波数);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn 支配周波数_220hz() {
        let path = 出力先("sine220.wav");
        let 仕様 = wav仕様::default();
        let samples = sine合成(220.0, 仕様.sample率, 0.5, 0.8);
        wav書出(&path, &仕様, &samples).unwrap();
        let (読仕様, 読samples) = wav読取(&path).unwrap();
        let 周波数 = 支配周波数(&読samples, 読仕様.sample率);
        assert!((周波数 - 220.0).abs() < 1.0, "220Hz推定誤差: {}", 周波数);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn 支配周波数_880hz() {
        let path = 出力先("sine880.wav");
        let 仕様 = wav仕様::default();
        let samples = sine合成(880.0, 仕様.sample率, 0.5, 0.8);
        wav書出(&path, &仕様, &samples).unwrap();
        let (読仕様, 読samples) = wav読取(&path).unwrap();
        let 周波数 = 支配周波数(&読samples, 読仕様.sample率);
        assert!((周波数 - 880.0).abs() < 1.0, "880Hz推定誤差: {}", 周波数);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn 全零は無音() {
        let samples = vec![0.0f32; 4800];
        assert!(無音か(&samples, 0.01));
        assert_eq!(実効値(&samples), 0.0);
    }

    #[test]
    fn clip飽和() {
        let path = 出力先("clip.wav");
        let 仕様 = wav仕様::default();
        let samples = vec![2.0f32, -2.0f32, 1.5f32, -1.5f32, 0.0f32];
        wav書出(&path, &仕様, &samples).unwrap();
        let (_, 読samples) = wav読取(&path).unwrap();
        for &s in &読samples {
            assert!(s >= -1.0 && s <= 1.0, "clip失敗: {}", s);
        }
        // 飽和確認: 正負とも最大振幅付近に張り付く
        assert!(読samples[0] > 0.99);
        assert!(読samples[1] < -0.99);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn 空sliceは0長wav() {
        let path = 出力先("empty.wav");
        let 仕様 = wav仕様::default();
        let samples: Vec<f32> = vec![];
        wav書出(&path, &仕様, &samples).unwrap();
        let (読仕様, 読samples) = wav読取(&path).unwrap();
        assert_eq!(読samples.len(), 0);
        assert_eq!(読仕様.sample率, 仕様.sample率);
        assert_eq!(実効値(&読samples), 0.0);
        assert!(無音か(&読samples, 0.0001));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn 親dir自動作成() {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("target");
        p.push("試験出力");
        p.push("入子");
        p.push("深い");
        p.push("nested.wav");
        let _ = std::fs::remove_dir_all(p.parent().unwrap().parent().unwrap());
        let 仕様 = wav仕様::default();
        let samples = sine合成(440.0, 仕様.sample率, 0.05, 0.3);
        wav書出(&p, &仕様, &samples).unwrap();
        assert!(p.exists());
        let _ = std::fs::remove_dir_all(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("試験出力")
                .join("入子"),
        );
    }

    #[test]
    fn 実効値既知値() {
        // 振幅0.5の一定値列 → RMS=0.5
        let samples = vec![0.5f32; 100];
        let rms = 実効値(&samples);
        assert!((rms - 0.5).abs() < 1e-9, "RMS={}", rms);
    }
}
