//! 歌口実行体 — mic / wav / 合成を検出し、環z形式でZ列を記す。

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, ValueEnum};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use utagu::入力::mic収録;
use utagu::写像::{写像param, 声z};
use utagu::検出::{検出param, 検出法, 音高検出};
use utagu::音高律::律;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum 源 {
    #[value(name = "mic")]
    Mic,
    #[value(name = "wav")]
    Wav,
    合成,
}

#[allow(non_snake_case)]
#[derive(Parser, Debug)]
#[command(name = "歌口", about = "梯5: 声→音高→Z。")]
struct 引数 {
    #[arg(long, value_enum, default_value_t = 源::Mic)]
    源: 源,
    #[arg(long)]
    wav: Option<PathBuf>,
    #[arg(long)]
    合成hz: Option<f64>,
    #[arg(long, default_value_t = 10.0)]
    秒: f64,
    #[arg(long)]
    出力: Option<PathBuf>,
    #[arg(long)]
    標本率: Option<u32>,
    #[arg(long)]
    窓長: Option<usize>,
    #[arg(long)]
    跳幅: Option<usize>,
    #[arg(long, value_enum)]
    法: Option<法引数>,
    #[arg(long)]
    下限hz: Option<f64>,
    #[arg(long)]
    上限hz: Option<f64>,
    #[arg(long)]
    YIN谷閾: Option<f64>,
    #[arg(long)]
    明瞭閾: Option<f64>,
    #[arg(long)]
    無音閾rms: Option<f64>,
    #[arg(long)]
    入力Nyquist比: Option<f64>,
    #[arg(long)]
    入力帯域上限hz: Option<f64>,
    #[arg(long)]
    基音: Option<f64>,
    #[arg(long, value_enum)]
    律: Option<律>,
    #[arg(long)]
    家snap: bool,
    #[arg(long)]
    家数: Option<u32>,
    #[arg(long)]
    満音rms: Option<f64>,
    #[arg(long)]
    lap下限: Option<i64>,
    #[arg(long)]
    lap上限: Option<i64>,
    #[arg(long)]
    合成振幅: Option<f64>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum 法引数 {
    #[value(name = "YIN")]
    Yin,
    自己相関,
}

impl From<法引数> for 検出法 {
    fn from(法: 法引数) -> Self {
        match 法 {
            法引数::Yin => 検出法::YIN,
            法引数::自己相関 => 検出法::自己相関,
        }
    }
}

fn 既定出力() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proof/歌口/歌z.txt")
}

fn 時刻ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn z行(ts: u128, z: &wa::z::Z, hz: Option<f64>) -> String {
    let (x, y) = z.直交();
    let hz = hz
        .map(|v| format!("{v:.6}"))
        .unwrap_or_else(|| "none".into());
    format!(
        "Z ts={} x={:.4} y={:.4} theta={:.6} r={:.6} lap={} hz={}",
        ts, x, y, z.theta, z.r, z.lap, hz
    )
}

fn param(引: &引数) -> (検出param, 写像param) {
    let mut 検 = 検出param::default();
    let mut 写 = 写像param::default();
    if let Some(v) = 引.標本率 {
        検.標本率 = v;
    }
    if let Some(v) = 引.窓長 {
        検.窓長 = v;
    }
    if let Some(v) = 引.跳幅 {
        検.跳幅 = v;
    }
    if let Some(v) = 引.法 {
        検.法 = v.into();
    }
    if let Some(v) = 引.下限hz {
        検.下限hz = v;
    }
    if let Some(v) = 引.上限hz {
        検.上限hz = v;
    }
    if let Some(v) = 引.YIN谷閾 {
        検.YIN谷閾 = v;
    }
    if let Some(v) = 引.明瞭閾 {
        検.明瞭閾 = v;
        写.明瞭閾 = v;
    }
    if let Some(v) = 引.無音閾rms {
        検.無音閾rms = v;
        写.無音閾rms = v;
    }
    if let Some(v) = 引.入力Nyquist比 {
        検.入力Nyquist比 = v;
    }
    if let Some(v) = 引.入力帯域上限hz {
        検.入力帯域上限hz = v;
    }
    if let Some(v) = 引.基音 {
        写.基音 = v;
    }
    if let Some(v) = 引.律 {
        写.律 = v;
        // 律選択はL域の実家数へ直結。--家数 はこの後の明示override。
        写.家数 = match v { 律::八家 => 8, 律::十二平均律 => 12 };
    }
    写.家snap = 引.家snap;
    if let Some(v) = 引.家数 {
        写.家数 = v;
    }
    if let Some(v) = 引.満音rms {
        写.満音rms = v;
    }
    if let Some(v) = 引.lap下限 {
        写.lap下限 = v;
    }
    if let Some(v) = 引.lap上限 {
        写.lap上限 = v;
    }
    (検, 写)
}

fn wav読(道: &Path) -> Result<(u32, Vec<f32>), String> {
    let mut 読 = WavReader::open(道).map_err(|誤| format!("wav開失敗 {}: {誤}", 道.display()))?;
    let 規 = 読.spec();
    if 規.channels == 0 {
        return Err("wav道数零".into());
    }
    let 生: Vec<f32> = match 規.sample_format {
        SampleFormat::Float => 読
            .samples::<f32>()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|誤| format!("wav読失敗: {誤}"))?,
        SampleFormat::Int => {
            let 分母 = 2f64.powi(規.bits_per_sample as i32 - 1) as f32;
            読.samples::<i32>()
                .map(|v| v.map(|x| (x as f32 / 分母).clamp(-1.0, 1.0)))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|誤| format!("wav読失敗: {誤}"))?
        }
    };
    let 道数 = 規.channels as usize;
    let 単声 = 生
        .chunks(道数)
        .map(|f| f.iter().sum::<f32>() / f.len() as f32)
        .collect();
    Ok((規.sample_rate, 単声))
}

fn 合成(道: &Path, hz: f64, 秒: f64, 標本率: u32, 振幅: f64) -> Result<Vec<f32>, String> {
    if !hz.is_finite() || hz <= 0.0 || !秒.is_finite() || 秒 <= 0.0 || 標本率 == 0 {
        return Err("合成param不正".into());
    }
    let 数 = (秒 * 標本率 as f64).round() as usize;
    let 標本: Vec<f32> = (0..数)
        .map(|i| {
            (振幅 * (std::f64::consts::TAU * hz * i as f64 / 標本率 as f64).sin()).clamp(-1.0, 1.0)
                as f32
        })
        .collect();
    let 規 = WavSpec {
        channels: 1,
        sample_rate: 標本率,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut 書 =
        WavWriter::create(道, 規).map_err(|誤| format!("合成wav作失敗 {}: {誤}", 道.display()))?;
    for x in &標本 {
        書.write_sample(*x)
            .map_err(|誤| format!("合成wav書失敗: {誤}"))?;
    }
    書.finalize().map_err(|誤| format!("合成wav終失敗: {誤}"))?;
    Ok(標本)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let 引 = 引数::parse();
    let (mut 検param, 写param) = param(&引);
    let 出力 = 引.出力.clone().unwrap_or_else(既定出力);
    if let Some(親) = 出力.parent() {
        fs::create_dir_all(親)?;
    }
    let mut log = File::create(&出力)?;
    let (標本率, 標本, 源記) = match 引.源 {
        源::Mic => {
            let r = mic収録(引.秒).map_err(io::Error::other)?;
            (r.標本率, r.標本, "mic".to_string())
        }
        源::Wav => {
            let 道 = 引.wav.as_deref().ok_or("--源 wav には --wav が要る")?;
            let (率, 標本) = wav読(道).map_err(io::Error::other)?;
            (率, 標本, format!("wav:{}", 道.display()))
        }
        源::合成 => {
            let hz = 引.合成hz.unwrap_or(写param.基音);
            let 振幅 = 引.合成振幅.unwrap_or(0.5);
            let wav = 出力
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("合成.wav");
            let 標本 = 合成(&wav, hz, 引.秒, 検param.標本率, 振幅).map_err(io::Error::other)?;
            (検param.標本率, 標本, format!("合成:{}", wav.display()))
        }
    };
    // mic/wavは実入力の標本率が音高周期の基準。指定値での上書きは物理入力を歪める為しない。
    検param.標本率 = 標本率;
    writeln!(
        log,
        "# 歌口 梯5 起動 ts={} 源={} 標本率={}",
        時刻ms(),
        源記,
        標本率
    )?;
    writeln!(log, "# 検出param {:?}", 検param)?;
    writeln!(log, "# 写像param {:?}", 写param)?;
    let 開始時刻 = 時刻ms();
    let mut 数 = 0usize;
    if 検param.窓長 == 0 || 検param.跳幅 == 0 {
        writeln!(log, "# UNVERIFIED: 窓長又は跳幅が零")?;
    } else {
        for 始 in (0usize..)
            .step_by(検param.跳幅)
            .take_while(|始| 始.saturating_add(検param.窓長) <= 標本.len())
        {
            let 検出 = 音高検出(&標本[始..始 + 検param.窓長], &検param);
            let z = 声z(&検出, &写param);
            let ts = 開始時刻 + (始 as u128 * 1_000 / 標本率 as u128);
            // 環音/src/源.rsの唯一parserは`Z … theta=… r=… lap=…`を直読する。
            // 行末hzは観測注釈のみ。下流音高はZの総角・巻を唯一の入力契約とする。
            writeln!(log, "{}", z行(ts, &z, 検出.hz))?;
            writeln!(
                log,
                "# 検出 hz={:?} 明瞭度={:.6} rms={:.6}",
                検出.hz, 検出.明瞭度, 検出.rms
            )?;
            数 += 1;
        }
    }
    if 数 == 0 {
        writeln!(
            log,
            "# UNVERIFIED: 完全窓なし 標本={} 窓長={}",
            標本.len(),
            検param.窓長
        )?;
    }
    writeln!(log, "# 歌口 梯5 終了 ts={} Z標本={}", 時刻ms(), 数)?;
    log.flush()?;
    Ok(())
}
