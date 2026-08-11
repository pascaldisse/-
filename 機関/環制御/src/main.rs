//! 環制御 — DualSense→場 制御界面, 梯1 (読取).
//! 文書/環制御.md 参照. crate選択: gilrs (pure rust, mac Bluetooth gamepad対応,
//! SDL2/hidapiのC依存無しで完結するため).

mod polar;

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::Parser;
use gilrs::{Axis, Button, Event, EventType, Gilrs};

use polar::{clamp_trigger, stick_to_polar};

/// 既知の Button 全種 (Unknown除く, gilrs::Button 定義順).
const ALL_BUTTONS: &[Button] = &[
    Button::South,
    Button::East,
    Button::North,
    Button::West,
    Button::C,
    Button::Z,
    Button::LeftTrigger,
    Button::LeftTrigger2,
    Button::RightTrigger,
    Button::RightTrigger2,
    Button::Select,
    Button::Start,
    Button::Mode,
    Button::LeftThumb,
    Button::RightThumb,
    Button::DPadUp,
    Button::DPadDown,
    Button::DPadLeft,
    Button::DPadRight,
];

/// 既知の Axis 全種 (Unknown除く, gilrs::Axis 定義順).
const ALL_AXES: &[Axis] = &[
    Axis::LeftStickX,
    Axis::LeftStickY,
    Axis::LeftZ,
    Axis::RightStickX,
    Axis::RightStickY,
    Axis::RightZ,
    Axis::DPadX,
    Axis::DPadY,
];

#[derive(Parser, Debug)]
#[command(
    name = "環制御",
    about = "DualSense→場 制御界面, 梯1 (読取). 文書/環制御.md 参照."
)]
struct Args {
    /// 中央deadzone (0.0-1.0) — stick押幅がこれ未満なら家=無 (中央春=場への自動帰還).
    #[arg(long, default_value_t = 0.15)]
    deadzone: f32,

    /// poll周波数 (Hz) — 生値stream採取率.
    #[arg(long, default_value_t = 60.0)]
    poll_hz: f64,

    /// 入力log出力先. 未指定なら crate直下からの相対 ../../proof/環制御/入力log.txt
    /// (= project root proof/環制御/入力log.txt) を既定とする.
    #[arg(long)]
    log_path: Option<PathBuf>,

    /// 実行時間 (秒). 0 = 無限 (Ctrl-C終了).
    #[arg(long, default_value_t = 15)]
    duration_secs: u64,

    /// 装置列挙前の温機時間 (ms). macOS IOHIDの「既接続device」通知は
    /// Gilrs::new() 直後には非同期で未着のことがあるため, 列挙前に
    /// 短時間 next_event() を汲み続けて確定させる (鉄則: hardcode禁 → param化).
    #[arg(long, default_value_t = 400)]
    warmup_ms: u64,
}

fn default_log_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proof/環制御/入力log.txt")
}

/// stdoutとlogファイルの両方に一行書く (proof用二重化).
fn emit(log: &mut File, line: &str) {
    println!("{line}");
    let _ = writeln!(log, "{line}");
}

fn ts_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let log_path = args.log_path.clone().unwrap_or_else(default_log_path);

    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut log = File::create(&log_path)?;

    let mut gilrs = match Gilrs::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Gilrs::new() 失敗: {e:?}");
            emit(&mut log, &format!("# FATAL Gilrs::new() failed: {e:?}"));
            return Ok(());
        }
    };

    emit(&mut log, &format!("# 環制御 梯1 (読取) 起動 ts={}", ts_ms()));
    emit(
        &mut log,
        &format!(
            "# param deadzone={} poll_hz={} duration_secs={} log_path={}",
            args.deadzone,
            args.poll_hz,
            args.duration_secs,
            log_path.display()
        ),
    );

    // 温機 — macOS IOHIDの「既接続検出」は Gilrs::new() 直後には非同期で未届きのことがあるため,
    // 列挙前に next_event() を汲み続けて登録を確定させる.
    let warmup_deadline = Instant::now() + Duration::from_millis(args.warmup_ms);
    while Instant::now() < warmup_deadline {
        while gilrs.next_event().is_some() {}
        std::thread::sleep(Duration::from_millis(10));
    }

    // 出力① — 装置列挙+接続証明.
    let mut device_count = 0usize;
    for (id, gamepad) in gilrs.gamepads() {
        device_count += 1;
        let axis_count = ALL_AXES
            .iter()
            .filter(|a| gamepad.axis_code(**a).is_some())
            .count();
        let button_count = ALL_BUTTONS
            .iter()
            .filter(|b| gamepad.button_code(**b).is_some())
            .count();
        emit(
            &mut log,
            &format!(
                "DEVICE id={:?} name=\"{}\" os_name=\"{}\" vendor_id={:?} product_id={:?} axes={} buttons={} connected={}",
                id,
                gamepad.name(),
                gamepad.os_name(),
                gamepad.vendor_id(),
                gamepad.product_id(),
                axis_count,
                button_count,
                gamepad.is_connected(),
            ),
        );
    }
    if device_count == 0 {
        emit(
            &mut log,
            "# UNVERIFIED: 接続gamepad 0台 — DualSense未検出 (Bluetooth権限/ペアリング確認要)",
        );
    } else {
        emit(&mut log, &format!("# 接続gamepad {device_count}台 検出"));
    }

    // 出力② — 生値stream (60Hz既定poll) + button落起 (event駆動).
    let tick = Duration::from_secs_f64(1.0 / args.poll_hz.max(0.001));
    let run_forever = args.duration_secs == 0;
    let deadline = Instant::now() + Duration::from_secs(args.duration_secs);
    let mut next_tick = Instant::now();

    loop {
        if !run_forever && Instant::now() >= deadline {
            break;
        }

        // button落起 (edge event) — poll間隔に依らず即時処理.
        while let Some(Event { id, event, .. }) = gilrs.next_event() {
            match event {
                EventType::ButtonPressed(btn, _) => {
                    emit(
                        &mut log,
                        &format!("EDGE ts={} id={:?} button={:?} state=down", ts_ms(), id, btn),
                    );
                }
                EventType::ButtonReleased(btn, _) => {
                    emit(
                        &mut log,
                        &format!("EDGE ts={} id={:?} button={:?} state=up", ts_ms(), id, btn),
                    );
                }
                EventType::Connected => {
                    emit(&mut log, &format!("EDGE ts={} id={:?} event=connected", ts_ms(), id));
                }
                EventType::Disconnected => {
                    emit(&mut log, &format!("EDGE ts={} id={:?} event=disconnected", ts_ms(), id));
                }
                _ => {}
            }
        }

        // 生値stream — 各接続gamepadを poll_hz で採取.
        let ids: Vec<_> = gilrs.gamepads().map(|(id, _)| id).collect();
        for id in ids {
            if let Some(gamepad) = gilrs.connected_gamepad(id) {
                let lx = gamepad.value(Axis::LeftStickX);
                let ly = gamepad.value(Axis::LeftStickY);
                let rx = gamepad.value(Axis::RightStickX);
                let ry = gamepad.value(Axis::RightStickY);
                let l2 = clamp_trigger(gamepad.value(Axis::LeftZ));
                let r2 = clamp_trigger(gamepad.value(Axis::RightZ));

                let left = stick_to_polar(lx, ly, args.deadzone);
                let right = stick_to_polar(rx, ry, args.deadzone);

                emit(
                    &mut log,
                    &format!(
                        "TICK ts={} id={:?} L(x={:.4} y={:.4} angle={:.2} mag={:.4} house={}) R(x={:.4} y={:.4} angle={:.2} mag={:.4} house={}) L2={:.4} R2={:.4}",
                        ts_ms(),
                        id,
                        lx, ly, left.angle_deg, left.magnitude,
                        left.house.map(|h| h.to_string()).unwrap_or_else(|| "無".into()),
                        rx, ry, right.angle_deg, right.magnitude,
                        right.house.map(|h| h.to_string()).unwrap_or_else(|| "無".into()),
                        l2, r2,
                    ),
                );
            }
        }

        next_tick += tick;
        let now = Instant::now();
        if next_tick > now {
            std::thread::sleep(next_tick - now);
        } else {
            next_tick = now;
        }
    }

    emit(&mut log, &format!("# 環制御 梯1 終了 ts={}", ts_ms()));
    log.flush()?;
    Ok(())
}
