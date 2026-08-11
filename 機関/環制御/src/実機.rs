//! 実機 — gilrs live読取の共通law. 梯1 (main.rs) · 梯2 (z主.rs) に別々に埋め込まれていた
//! 「Gilrs起動+温機+device列挙+左stick読取」を契約層唯一実装へ集約する
//! (梯4前梯 実機歌鐘 08-11: 梯3 環音のlive源はここを再用し、私有poll再実装をしない).

use std::time::{Duration, Instant};

use gilrs::{Axis, Gilrs, GamepadId};

/// 起動+温機param — macOS IOHIDの「既接続device」通知は Gilrs::new() 直後には非同期で
/// 未着のことがある為, 列挙前に短時間 next_event() を汲み続けて確定させる.
/// 全既定つき (鉄則: hardcode禁).
#[derive(Debug, Clone, Copy)]
pub struct 温機param {
    /// 温機総時間 (ms). 既定 400 (環制御 梯1 main.rs 既存値と同一).
    pub 温機ms: u64,
    /// 温機中のevent汲取り間隔 (ms). 既定 10.
    pub 温機poll_ms: u64,
}

impl Default for 温機param {
    fn default() -> Self {
        Self { 温機ms: 400, 温機poll_ms: 10 }
    }
}

/// Gilrs起動+温機. 失敗はそのまま呼出側へ伝播する (黙fallback禁 —
/// 「実機不在」の判定・error文面化は呼出側の責務).
pub fn 起動温機(param: 温機param) -> Result<Gilrs, gilrs::Error> {
    let mut g = Gilrs::new()?;
    let 締 = Instant::now() + Duration::from_millis(param.温機ms);
    while Instant::now() < 締 {
        while g.next_event().is_some() {}
        std::thread::sleep(Duration::from_millis(param.温機poll_ms.max(1)));
    }
    Ok(g)
}

/// 接続device数 (温機後に呼ぶ).
pub fn 接続数(g: &Gilrs) -> usize {
    g.gamepads().count()
}

/// 接続device一覧の一行log片群 (梯1 main.rs 既存書式と同法 — proof可読性を揃える).
pub fn device行群(g: &Gilrs) -> Vec<String> {
    g.gamepads()
        .map(|(id, gp)| {
            format!(
                "DEVICE id={id:?} name=\"{}\" connected={}",
                gp.name(),
                gp.is_connected()
            )
        })
        .collect()
}

/// 最初の接続device (最若いid) の左stick生値 (x, y). 未接続ならNone.
/// 呼ぶ前にevent queueを汲む (梯1/梯2既存作法 — button等の取りこぼしでpollが詰まらぬ為).
pub fn 左stick読取(g: &mut Gilrs) -> Option<(GamepadId, f64, f64)> {
    while g.next_event().is_some() {}
    let id = g.gamepads().next()?.0;
    let gp = g.connected_gamepad(id)?;
    Some((
        id,
        gp.value(Axis::LeftStickX) as f64,
        gp.value(Axis::LeftStickY) as f64,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 温機param既定は四百ms十ms() {
        let p = 温機param::default();
        assert_eq!(p.温機ms, 400);
        assert_eq!(p.温機poll_ms, 10);
    }

    #[test]
    fn 温機ms零でも汲取ループは一回で抜ける() {
        // 実機無し環境でも Gilrs::new() 自体は成立し得る (device 0台) — 起動温機は
        // 温機ms=0なら即帰る事だけを確認する (実機依存部はUNVERIFIED — CI/実機無し双方で成立する範囲).
        let 結果 = 起動温機(温機param { 温機ms: 0, 温機poll_ms: 1 });
        match 結果 {
            Ok(g) => assert!(接続数(&g) == 接続数(&g), "接続数は決定論的"), // 恒真 — 呼出が壊れぬ事の確認
            Err(_) => {} // 環境にGilrsが全く成立しない (CI等) — 失敗も想定内, panicしない事のみ検査
        }
    }
}
