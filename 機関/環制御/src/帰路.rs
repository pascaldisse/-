//! 帰路 — 場応答 (r) → DualSense rumble haptic (任A支援, 08-11 Pascal指示).
//! 文書/環制御.md §帰路: haptics=場振幅@手元, 既定律動2Hz (生命搬送波).
//! 梯4 (z→場注入) 未着地の間は r を暫定param (帰路主.rs `--径`) で代入する.
//!
//! 実機 force feedback 不可 (BT不在・ff未対応・OS拒否) の場合は呼出側へ **正直な
//! UNVERIFIED** を返す — 捏造proof禁 (殿律)。呼出側 (帰路主.rs) は同一スケジュールを
//! 代替帰路 (帰路log出力) へ書き出す事で、実機無しでも param通りの帰路仕様を検証可能にする。
//!
//! 全定数 = param既定つき (鉄則: hardcode禁, 例外=LOVE=1のみ)。純粋計算部
//! (周期ms/on_ms/off_ms/強度写像) は cargo test 対象 (機器非依存)。gilrs配線部
//! (実機起動) は実gamepad前提の為、単体testでは検証しない — 帰路主.rs の実走で検証する。

use gilrs::ff::{BaseEffect, BaseEffectType, EffectBuilder, Replay, Ticks};
use gilrs::{GamepadId, Gilrs};

/// haptic帰路 param — 全て既定つき (鉄則: hardcode禁, 例外=LOVE=1のみ).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct 帰路param {
    /// 搬送周波数 (Hz). 既定2.0 — 生命搬送波 (文書/環統合.md `mama=0101`, 文書/環制御.md §帰路).
    pub 搬送hz: f64,
    /// on/off duty比 [0,1]. 既定0.5 (対称矩形 — 出現律に対する明示的例外: haptic gateは
    /// 意図的な離散on/offであり、連続場の pop-in 禁とは別の律動系).
    pub duty: f64,
    /// r=1 (最大振幅) 時のrumble強度 [0, 65535]. 既定 39321 (u16::MAX の約六割 — 全力振動回避,
    /// DualSense実機での過振動を避ける安全域. 直書きに見えるが「他の値でも成立し得る選択」の
    /// 具体値そのものであり param化必須 — CLI `--最大強度` で上書き可).
    pub 最大強度: u16,
    /// r=0付近でも触知を完全に消さない為の下駄. 既定0 (下駄なし = r=0で無音, 場の無=沈黙契約と整合).
    pub 最小強度: u16,
    /// 一回発火の持続秒 (安全弁 — 無限発火を防ぐ). 既定4.0.
    pub 継続秒: f64,
}

impl Default for 帰路param {
    fn default() -> Self {
        帰路param {
            搬送hz: 2.0,
            duty: 0.5,
            最大強度: 39_321,
            最小強度: 0,
            継続秒: 4.0,
        }
    }
}

/// 搬送周期 (ms). 零除算防止: 搬送hz の床 1e-6.
pub fn 周期ms(param: &帰路param) -> u32 {
    (1000.0 / param.搬送hz.max(1e-6)).round().max(1.0) as u32
}

/// on区間長 (ms) = 周期 × duty (duty は [0,1] へclamp, on は最低1ms).
pub fn on_ms(param: &帰路param) -> u32 {
    let 周期 = 周期ms(param) as f64;
    ((周期 * param.duty.clamp(0.0, 1.0)).round()).clamp(1.0, 周期) as u32
}

/// off区間長 (ms) = 周期 − on.
pub fn off_ms(param: &帰路param) -> u32 {
    周期ms(param).saturating_sub(on_ms(param))
}

/// 場振幅 r∈[0,1] → rumble強度 [最小強度, 最大強度] へ線形写像 (r=0=無=最小強度).
pub fn 強度写像(r: f64, param: &帰路param) -> u16 {
    let r = r.clamp(0.0, 1.0);
    let 下 = param.最小強度 as f64;
    let 上 = param.最大強度 as f64;
    let 幅 = (上 - 下).max(0.0);
    (下 + r * 幅).round().clamp(0.0, u16::MAX as f64) as u16
}

/// 帰路発火の結果.
///
/// **macOS既知欠陥 (甲追令, 08-11 批根丙先行審)**: gilrs-core-0.6.8
/// `src/platform/macos/ff.rs` の `Device::set_ff_state` は実測=**空実装**
/// (`pub fn set_ff_state(&mut self, _strong: u16, _weak: u16, _min_duration: Duration) {}` —
/// 引数を捨てて何もしない)。故に `EffectBuilder::finish`+`Effect::play` が `Ok(())` を
/// 返しても **macOSでは実振動が起こらない事が実装から確定している**。
/// 本型は「APIが受理した」と「実機体感で振動が起きた」を明確に分離する —
/// `Api受理` を「動いた」「haptic確認」と書くな (捏造proof禁, 殿律)。
#[derive(Debug)]
pub enum 帰路結果 {
    /// ff対応device 1台以上に API発火 (`Ok`) した。**実振動は未確認**
    /// (macOS=既知no-op で確実に無振動 · 他OS=本実装では未検証, 双方UNVERIFIED扱い).
    Api受理 {
        対象: Vec<GamepadId>,
        周期ms: u32,
        on_ms: u32,
        off_ms: u32,
        強度: u16,
    },
    /// ff対応device無し or API自体が失敗 (EffectBuilder/play失敗).
    Unverified {
        理由: String,
    },
}

/// r (場振幅) から DualSense rumble を搬送hz (既定2Hz) の on/off で発火させる試み.
/// gilrs::Gilrs は呼出側所有 (帰路主.rs で `Gilrs::new()` 済のものを渡す).
///
/// 実gamepad接続が必要な為、本関数自体は #[cfg(test)] では検証しない (機器非依存な
/// 純粋計算部=周期ms/on_ms/off_ms/強度写像 のみ下記testで検証する) — 実機検証は
/// 帰路主.rs の実走ログで行う (殿律: 実走証跡無き「動く」を言うな).
pub fn 実機起動(
    gilrs: &mut Gilrs,
    r: f64,
    param: &帰路param,
) -> (帰路結果, Option<gilrs::ff::Effect>) {
    let 対象: Vec<GamepadId> = gilrs
        .gamepads()
        .filter_map(|(id, gp)| if gp.is_ff_supported() { Some(id) } else { None })
        .collect();

    if 対象.is_empty() {
        return (
            帰路結果::Unverified {
                理由: "force feedback対応gamepad 0台 (実機BT不在 or ff未対応)".into(),
            },
            None,
        );
    }

    let 強度 = 強度写像(r, param);
    let on = Ticks::from_ms(on_ms(param));
    let off = Ticks::from_ms(off_ms(param));

    let build = EffectBuilder::new()
        .add_effect(BaseEffect {
            kind: BaseEffectType::Strong { magnitude: 強度 },
            scheduling: Replay {
                after: Ticks::from_ms(0),
                play_for: on,
                with_delay: off,
            },
            envelope: Default::default(),
        })
        .gamepads(&対象)
        .finish(gilrs);

    match build {
        Ok(effect) => match effect.play() {
            Ok(()) => (
                帰路結果::Api受理 {
                    対象,
                    周期ms: 周期ms(param),
                    on_ms: on_ms(param),
                    off_ms: off_ms(param),
                    強度,
                },
                Some(effect),
            ),
            Err(e) => (
                帰路結果::Unverified {
                    理由: format!("Effect::play失敗: {e:?}"),
                },
                None,
            ),
        },
        Err(e) => (
            帰路結果::Unverified {
                理由: format!("EffectBuilder::finish失敗: {e:?}"),
            },
            None,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 既定は二Hz対称() {
        let p = 帰路param::default();
        assert_eq!(p.搬送hz, 2.0);
        assert_eq!(p.duty, 0.5);
    }

    #[test]
    fn 二hz既定の周期は五百ms() {
        let p = 帰路param::default();
        assert_eq!(周期ms(&p), 500);
        assert_eq!(on_ms(&p), 250);
        assert_eq!(off_ms(&p), 250);
    }

    #[test]
    fn 搬送hzはparamで変わる() {
        let p = 帰路param {
            搬送hz: 4.0,
            ..Default::default()
        };
        assert_eq!(周期ms(&p), 250);
        assert_eq!(on_ms(&p), 125);
        assert_eq!(off_ms(&p), 125);
    }

    #[test]
    fn dutyは非対称も表現できる() {
        let p = 帰路param {
            搬送hz: 2.0,
            duty: 0.25,
            ..Default::default()
        };
        assert_eq!(周期ms(&p), 500);
        assert_eq!(on_ms(&p), 125);
        assert_eq!(off_ms(&p), 375);
    }

    #[test]
    fn 強度写像は境界で最小最大() {
        let p = 帰路param::default();
        assert_eq!(強度写像(0.0, &p), p.最小強度);
        assert_eq!(強度写像(1.0, &p), p.最大強度);
    }

    #[test]
    fn 強度写像は単調増加() {
        let p = 帰路param::default();
        let a = 強度写像(0.2, &p);
        let b = 強度写像(0.6, &p);
        let c = 強度写像(1.0, &p);
        assert!(a <= b && b <= c, "a={a} b={b} c={c}");
    }

    #[test]
    fn 強度写像は範囲外rをclampする() {
        let p = 帰路param::default();
        assert_eq!(強度写像(-1.0, &p), p.最小強度);
        assert_eq!(強度写像(2.0, &p), p.最大強度);
    }

    #[test]
    fn 最小強度の下駄が効く() {
        let p = 帰路param {
            最小強度: 5000,
            最大強度: 40000,
            ..Default::default()
        };
        assert_eq!(強度写像(0.0, &p), 5000);
        assert_eq!(強度写像(1.0, &p), 40000);
    }

    #[test]
    fn 周期は搬送hz零でも暴走しない() {
        let p = 帰路param {
            搬送hz: 0.0,
            ..Default::default()
        };
        assert!(周期ms(&p) > 0);
    }
}
