//! 梯2 実行体 — stick生値 → z stream. 文書/環統合.md §主座標.
//! 入力源 = param {実機 | 再生 | 自動}. 自動 = 実機を試し、gamepad 0台なら再生へ落ちる
//! (実機不在のMac単体でも、既定再生元が可読·出力先が作成可なら再生路で走る = 既定fallback.
//!  file不可読時は失敗する — 「必ず走る」ではない).


use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::{Parser, ValueEnum};
use gilrs::{Axis, Gilrs};

use wa::z::{Z, Z変param, Z変換器};
use wa::入力源::log読込;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum 源 {
    /// gilrs実機 (DualSense).
    実機,
    /// 梯1入力logの決定論再生.
    再生,
    /// 実機を試し、不在なら再生へ落ちる (既定).
    自動,
}

#[derive(Parser, Debug)]
#[command(name = "環z", about = "梯2: stick生値→z変換器. 文書/環統合.md 参照.")]
struct 引数 {
    /// 入力源.
    #[arg(long, value_enum, default_value_t = 源::自動)]
    源: 源,

    /// 再生元log (梯1形式). 既定 = proof/環制御/入力log.txt.
    #[arg(long)]
    再生元: Option<PathBuf>,

    /// z出力log. 既定 = proof/環制御/z再生log.txt.
    #[arg(long)]
    出力: Option<PathBuf>,

    /// 中央死域 [0,1].
    #[arg(long, default_value_t = 0.08)]
    死域: f64,

    /// 死域外を [0,1] へ再写像 (出現律: 連続立上り).
    #[arg(long, default_value_t = true)]
    死域再正規化: bool,

    /// 八家 45° snap (既定 off).
    #[arg(long, default_value_t = false)]
    八家snap: bool,

    /// snap分割数 (8=八卦).
    #[arg(long, default_value_t = 8)]
    家数: u32,

    /// r上限clamp.
    #[arg(long, default_value_t = 1.0)]
    r上限: f64,

    /// 実機poll周波数 (Hz).
    #[arg(long, default_value_t = 60.0)]
    poll_hz: f64,

    /// 実機実行時間 (秒). 0 = 無限.
    #[arg(long, default_value_t = 15)]
    実行秒: u64,

    /// 実機列挙前の温機 (ms) — macOS IOHIDの既接続通知は非同期.
    #[arg(long, default_value_t = 400)]
    温機ms: u64,

    /// 再生時に log の実時間で待つか (false = 全速, 既定 false → test/CI決定論).
    #[arg(long, default_value_t = false)]
    実時間再生: bool,

    /// 再生の標本上限 (0 = 全部).
    #[arg(long, default_value_t = 0)]
    再生上限: usize,

    /// 温機中のevent汲取り間隔 (ms).
    #[arg(long, default_value_t = 10)]
    温機poll_ms: u64,

    /// poll周波数の下限 (Hz) — 零除算防止の床.
    #[arg(long, default_value_t = 0.001)]
    最小poll_hz: f64,

    /// 実時間再生の一標本あたり待機上限 (ms) — logの時刻跳びで固まらぬ為.
    #[arg(long, default_value_t = 1000)]
    再生最大待機ms: u64,
}

fn 既定再生元() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proof/環制御/入力log.txt")
}
fn 既定出力() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proof/環制御/z再生log.txt")
}

fn 時刻ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn 書(log: &mut File, 行: &str) {
    println!("{行}");
    let _ = writeln!(log, "{行}");
}

fn z行(ts: u128, x: f64, y: f64, z: &Z) -> String {
    format!(
        "Z ts={} x={:.4} y={:.4} theta={:.6} r={:.6} lap={}",
        ts, x, y, z.theta, z.r, z.lap
    )
}

fn main() -> io::Result<()> {
    let args = 引数::parse();
    let 出力 = args.出力.clone().unwrap_or_else(既定出力);
    if let Some(親) = 出力.parent() {
        fs::create_dir_all(親)?;
    }
    let mut log = File::create(&出力)?;

    let param = Z変param {
        死域: args.死域,
        死域再正規化: args.死域再正規化,
        八家snap: args.八家snap,
        家数: args.家数,
        r上限: args.r上限,
        ..Default::default()
    };
    let mut 変 = Z変換器::新(param);

    書(&mut log, &format!("# 環z 梯2 起動 ts={}", 時刻ms()));
    書(&mut log, &format!("# param {param:?}"));

    // 源決定 — 自動は実機を試して不在なら再生.
    let mut gilrs = if matches!(args.源, 源::実機 | 源::自動) {
        Gilrs::new().ok()
    } else {
        None
    };
    let mut 実機台数 = 0usize;
    if let Some(g) = gilrs.as_mut() {
        let 締 = Instant::now() + Duration::from_millis(args.温機ms);
        while Instant::now() < 締 {
            while g.next_event().is_some() {}
            std::thread::sleep(Duration::from_millis(args.温機poll_ms));
        }
        for (id, gp) in g.gamepads() {
            実機台数 += 1;
            書(
                &mut log,
                &format!("DEVICE id={id:?} name=\"{}\" connected={}", gp.name(), gp.is_connected()),
            );
        }
    }

    let 実機使用 = match args.源 {
        源::再生 => false,
        源::実機 => 実機台数 > 0,
        源::自動 => 実機台数 > 0,
    };

    if args.源 == 源::実機 && !実機使用 {
        書(&mut log, "# UNVERIFIED: --源 実機 指定だが gamepad 0台 — 走行中止");
        return Ok(());
    }

    if 実機使用 {
        書(&mut log, &format!("# 源=実機 ({実機台数}台)"));
        let g = gilrs.as_mut().expect("実機使用時はGilrs有");
        let 刻 = Duration::from_secs_f64(1.0 / args.poll_hz.max(args.最小poll_hz));
        let 無限 = args.実行秒 == 0;
        let 締 = Instant::now() + Duration::from_secs(args.実行秒);
        let mut 次 = Instant::now();
        let mut 数 = 0u64;
        loop {
            if !無限 && Instant::now() >= 締 {
                break;
            }
            while g.next_event().is_some() {}
            let ids: Vec<_> = g.gamepads().map(|(id, _)| id).collect();
            for id in ids {
                if let Some(gp) = g.connected_gamepad(id) {
                    let x = gp.value(Axis::LeftStickX) as f64;
                    let y = gp.value(Axis::LeftStickY) as f64;
                    let zz = 変.変換(x, y);
                    書(&mut log, &z行(時刻ms(), x, y, &zz));
                    数 += 1;
                }
            }
            次 += 刻;
            let 今 = Instant::now();
            if 次 > 今 {
                std::thread::sleep(次 - 今);
            } else {
                次 = 今;
            }
        }
        書(&mut log, &format!("# 実機標本 {数} · 終巻 lap={}", 変.巻()));
    } else {
        let 元 = args.再生元.clone().unwrap_or_else(既定再生元);
        書(&mut log, &format!("# 源=再生 元={}", 元.display()));
        let mut 列 = log読込(&元)?;
        if args.再生上限 > 0 && 列.len() > args.再生上限 {
            列.truncate(args.再生上限);
        }
        if 列.is_empty() {
            書(&mut log, "# UNVERIFIED: 再生元に TICK 行なし — z列 空");
        }
        let mut 前ts: Option<u128> = None;
        for s in &列 {
            if args.実時間再生 {
                if let Some(p) = 前ts {
                    let 差 = s.ts.saturating_sub(p);
                    if 差 > 0 && 差 < args.再生最大待機ms as u128 {
                        std::thread::sleep(Duration::from_millis(差 as u64));
                    }
                }
                前ts = Some(s.ts);
            }
            let zz = 変.変換(s.x, s.y);
            書(&mut log, &z行(s.ts, s.x, s.y, &zz));
        }
        書(
            &mut log,
            &format!("# 再生標本 {} · 終巻 lap={}", 列.len(), 変.巻()),
        );
    }

    書(&mut log, &format!("# 環z 梯2 終了 ts={}", 時刻ms()));
    log.flush()?;
    Ok(())
}
