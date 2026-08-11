//! 帰路主 — haptic帰路 実行体 (任A支援, 08-11 Pascal指示 + 甲追令 08-11 批根丙先行審 取込).
//! 文書/環制御.md §帰路: haptics=場振幅@手元, 既定律動2Hz. 場応答r (梯4未着地の間は
//! `--径` 暫定param) → DualSense rumble を試みる.
//!
//! **甲追令 (08-11)**: macOSの gilrs は force feedback API呼出に成功 (`Ok`) を返すが
//! 実振動は零 (gilrs-core-0.6.8 `platform/macos/ff.rs::Device::set_ff_state` 実測=空実装 —
//! wa::帰路::帰路結果 doc参照)。故に API成功=「動いた」と書かない。実振動は常に
//! **正直にUNVERIFIED** と申告し、**代替帰路 (帰路log出力)** をAPI結果に関わらず常時併記する
//! (param既定つき, 捏造proof禁 — 殿律)。

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::Parser;
use gilrs::Gilrs;

use wa::実機::{起動温機, 温機param};
use wa::帰路::{実機起動, 周期ms, 帰路param, 帰路結果, 強度写像, off_ms, on_ms};

#[derive(Parser, Debug)]
#[command(
    name = "帰路",
    about = "haptic帰路: 場応答r→DualSense rumble 試行 (2Hz搬送既定). 実振動は常にUNVERIFIED申告 (macOS既知no-op). 文書/環制御.md 参照."
)]
struct 引数 {
    /// 場振幅 r [0,1] — 梯4 (z→場注入) 未着地の間の暫定入力.
    #[arg(long, default_value_t = 1.0)]
    径: f64,

    /// 搬送周波数 (Hz).
    #[arg(long, default_value_t = 2.0)]
    搬送hz: f64,

    /// duty比 [0,1].
    #[arg(long, default_value_t = 0.5)]
    duty: f64,

    /// 最大強度 [0,65535].
    #[arg(long, default_value_t = 39_321)]
    最大強度: u16,

    /// 最小強度 [0,65535] (下駄, 既定0=下駄なし).
    #[arg(long, default_value_t = 0)]
    最小強度: u16,

    /// 継続秒 (安全弁).
    #[arg(long, default_value_t = 4.0)]
    継続秒: f64,

    /// 実機列挙前の温機ms.
    #[arg(long, default_value_t = 400)]
    温機ms: u64,

    /// 温機中のevent汲取り間隔 (ms).
    #[arg(long, default_value_t = 10)]
    温機poll_ms: u64,

    /// 帰路log出力先 (API結果に関わらず常時書く — 代替帰路 兼 実走証跡).
    /// 既定 proof/環制御/帰路log.txt.
    #[arg(long)]
    出力: Option<PathBuf>,
}

fn 既定出力() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proof/環制御/帰路log.txt")
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

/// 代替帰路 — API結果に関わらず常時書く param通りのon/offスケジュール
/// (捏造禁: 実機体感を主張せず「paramならこう鳴るはず」の仕様行としてのみ書く).
fn 代替帰路書出(log: &mut File, 径: f64, param: &帰路param) {
    let 周期 = 周期ms(param);
    let on = on_ms(param);
    let off = off_ms(param);
    let 強度 = 強度写像(径, param);
    let 反復 = ((param.継続秒 * 1000.0) / (周期 as f64)).ceil().max(1.0) as u32;
    書(
        log,
        &format!(
            "# 代替帰路 (param通りの仕様行 — 実振動の証跡ではない) 周期ms={周期} on_ms={on} off_ms={off} 強度={強度} 反復={反復}"
        ),
    );
    for i in 0..反復 {
        let t0 = i * 周期;
        書(log, &format!("HAPTIC t_ms={} phase=on dur_ms={} 強度={}", t0, on, 強度));
        書(
            log,
            &format!("HAPTIC t_ms={} phase=off dur_ms={} 強度=0", t0 + on, off),
        );
    }
}

fn main() -> io::Result<()> {
    let args = 引数::parse();
    let param = 帰路param {
        搬送hz: args.搬送hz,
        duty: args.duty,
        最大強度: args.最大強度,
        最小強度: args.最小強度,
        継続秒: args.継続秒,
    };

    let 出力 = args.出力.clone().unwrap_or_else(既定出力);
    if let Some(親) = 出力.parent() {
        fs::create_dir_all(親)?;
    }
    let mut log = File::create(&出力)?;

    書(
        &mut log,
        &format!("# 帰路 起動 ts={} param={param:?} 径={}", 時刻ms(), args.径),
    );
    書(
        &mut log,
        "# 注: macOS gilrsのforce feedbackはAPI成功でも実振動0が既知 (gilrs-core-0.6.8 platform/macos/ff.rs::Device::set_ff_state=空実装). 実振動は常にUNVERIFIED申告する.",
    );

    let mut gilrs: Option<Gilrs> = 起動温機(温機param {
        温機ms: args.温機ms,
        温機poll_ms: args.温機poll_ms,
    })
    .ok();

    match gilrs.as_mut() {
        None => {
            書(&mut log, "# UNVERIFIED: Gilrs::new()失敗 (device層起動不可)");
        }
        Some(g) => {
            let (結果, effect) = 実機起動(g, args.径, &param);
            match 結果 {
                帰路結果::Api受理 {
                    対象,
                    周期ms,
                    on_ms,
                    off_ms,
                    強度,
                } => {
                    書(
                        &mut log,
                        &format!(
                            "# API受理 対象={対象:?} 周期ms={周期ms} on_ms={on_ms} off_ms={off_ms} 強度={強度}"
                        ),
                    );
                    書(
                        &mut log,
                        "# UNVERIFIED: 実振動 — macOS既知no-opの為, API成功のみでは体感確認にならない (人間の触知報告が別途必要)",
                    );
                    std::thread::sleep(Duration::from_secs_f64(param.継続秒.max(0.0)));
                    if let Some(e) = effect {
                        let _ = e.stop();
                    }
                    書(&mut log, &format!("# API発火終了 継続秒={}", param.継続秒));
                }
                帰路結果::Unverified { 理由 } => {
                    書(&mut log, &format!("# UNVERIFIED: {理由}"));
                }
            }
        }
    }

    // 代替帰路 — API結果に関わらず常時書く (捏造proof禁: 実振動主張はしない).
    代替帰路書出(&mut log, args.径, &param);

    書(&mut log, &format!("# 帰路 終了 ts={}", 時刻ms()));
    log.flush()?;
    Ok(())
}
