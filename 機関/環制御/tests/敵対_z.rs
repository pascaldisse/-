//! 敵対試験 — 梯2 (stick→z変換器) 単体, 独立path検証.
//! 文書=docs/adversary/2026-08-11-環統合審.md §甲.1 攻撃目録A1-A15.
//! 法: 既存 src/z.rs #[cfg(test)] は一切書き換えない (別file). z.rs内部実装も無修正.
//! 批側=審査のみ、是正は建根甲の仕事.

use wa::polar::stick_to_polar;
use wa::z::{環正規化, 家snap, 全環, 半環, Z変param, Z変換器, 列変換};

fn 近(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() < eps
}

// ---------- A1 θ縁 (±π wrap) ----------
#[test]
fn a1_負ゼロでもθ跳躍せず単一値() {
    // x=-1, y=+0.0 と y=-0.0 — atan2は理論上 +π / -π に分かれ得るが
    // 環正規化((-π,π]半開)がどちらも同じ側へ畳み込む事を確認.
    let mut v1 = Z変換器::既定();
    let mut v2 = Z変換器::既定();
    let z_pos0 = v1.変換(-1.0, 0.0);
    let z_neg0 = v2.変換(-1.0, -0.0);
    assert!(
        近(z_pos0.theta, z_neg0.theta, 1e-9),
        "θ跳躍あり: +0→{} / -0→{}",
        z_pos0.theta,
        z_neg0.theta
    );
    assert!(近(z_pos0.theta.abs(), 半環, 1e-9), "θ={}", z_pos0.theta);
}

// ---------- A2 θ単位不整合 (polar.rs=度 · z.rs=弧度) ----------
#[test]
fn a2_polarは度z弧度_同一入力で数値系が異なる() {
    // 契約上の危険地帯: 呼出側が単位を取り違えれば無音のbugになる (文書化目的の非回帰test).
    let s0 = stick_to_polar(1.0, 0.0, 0.0);
    let mut v = Z変換器::既定();
    let z0 = v.変換(1.0, 0.0);
    assert!(近(s0.angle_deg, 0.0, 1e-9));
    assert!(近(z0.theta, 0.0, 1e-9));
    let s90 = stick_to_polar(0.0, 1.0, 0.0);
    let z90 = v.変換(0.0, 1.0);
    assert!(近(s90.angle_deg, 90.0, 1e-9), "polar角={}", s90.angle_deg);
    assert!(近(z90.theta, 半環 / 2.0, 1e-9), "z角={}", z90.theta);
    // 90.0 (度) と π/2≈1.5708 (弧度) — 数値として全く別系である事を明示.
    assert_ne!(s90.angle_deg, z90.theta);
}

// ---------- A3 lap carry符号 (往復10回→lap=0) ----------
#[test]
fn a3_往復十回でlap零に戻る() {
    let 順 = {
        let n = 360usize;
        (0..=n)
            .map(|i| {
                let θ = 全環 * (i as f64) / (n as f64);
                (0.9 * θ.cos(), 0.9 * θ.sin())
            })
            .collect::<Vec<_>>()
    };
    let 逆 = {
        let n = 360usize;
        (0..=n)
            .map(|i| {
                let θ = -全環 * (i as f64) / (n as f64);
                (0.9 * θ.cos(), 0.9 * θ.sin())
            })
            .collect::<Vec<_>>()
    };
    let mut 変 = Z変換器::既定();
    for _ in 0..10 {
        for &(x, y) in &順 {
            変.変換(x, y);
        }
        for &(x, y) in &逆 {
            変.変換(x, y);
        }
    }
    assert_eq!(変.巻(), 0, "10往復後 lap={}", 変.巻());
}

// ---------- A4 lap chatter (θ=π近傍±0.001を1000回) ----------
#[test]
fn a4_π近傍微振動1000回のlap挙動を実測する() {
    // 物理的には枝の切断点(π)を跨ぐ真の往復 — 判定式(d<-π→+1, d>π→-1)通りなら
    // 交互+1/-1でnetは有界のはず. 有界性のみ主張し, 実測値をassertする (未検証のまま断言しない).
    let mut 変 = Z変換器::既定();
    let mut laps = Vec::with_capacity(1000);
    for i in 0..1000 {
        let θ = if i % 2 == 0 {
            半環 - 0.001
        } else {
            -(半環 - 0.001) // 環正規化はしないが変換内部で環正規化される
        };
        let z = 変.変換(0.9 * θ.cos(), 0.9 * θ.sin());
        laps.push(z.lap);
    }
    let 最大絶対値 = laps.iter().map(|l| l.abs()).max().unwrap();
    // 出現律違反 (無制限暴走) の検出線: |lap| が振動回数(500往復)近くまで単調成長したら暴走.
    assert!(
        最大絶対値 < 500,
        "lap chatter暴走疑い: 1000tick中 |lap|最大={} (chatter系列末尾10={:?})",
        最大絶対値,
        &laps[990..]
    );
}

// ---------- A5 lap aliasing (半回転/tick超の高速旋回) ----------
#[test]
fn a5_標本化限界_急旋回はlapを実測より過小に数える() {
    // 3周を4標本のみで通過 (Nyquist条件=標本間移動<π を明確に破る).
    let 列 = (0..=4)
        .map(|i| {
            let θ = 3.0 * 全環 * (i as f64) / 4.0;
            (0.9 * θ.cos(), 0.9 * θ.sin())
        })
        .collect::<Vec<_>>();
    let zs = 列変換(Z変param::default(), &列);
    let 実測lap = zs.last().unwrap().lap;
    assert_ne!(
        実測lap, 3,
        "エイリアシング未検出: 4標本で3周を正しく数えてしまった (Nyquist違反が無害化=検出機構自体が無い証跡としては実測lap={})",
        実測lap
    );
}

// ---------- A6 deadzone境界 (>=固定・片側のみ) ----------
#[test]
fn a6_死域境界はgte片側() {
    // 公開API (無か()=r==0) は境界でrが0になる仕様上 (連続立上り), 活性/非活性を
    // rだけでは判別不可. 内部の活性判定(>=)はlap追跡の有無で外部観測する:
    // πを跨ぐ回転列を mag=死域ちょうど で流し、>=なlap追跡が発火する事を確認.
    let dz = Z変param::default().死域;
    let 列: Vec<(f64, f64)> = (0..=20)
        .map(|i| {
            let θ = 半環 - 0.2 + 0.02 * (i as f64); // π-0.2 → π+0.2 を跨ぐ (環正規化後は-π側へ直連続).
            (dz * θ.cos(), dz * θ.sin())
        })
        .collect();
    let mut 変a = Z変換器::既定();
    let mut 最終lap = 0;
    for &(x, y) in &列 {
        最終lap = 変a.変換(x, y).lap;
    }
    assert_ne!(
        最終lap, 0,
        "境界ちょうど(mag==死域)でのπ跨ぎがlap追跡を発火させなかった (>= は活性側とならないはず) lap={}",
        最終lap
    );

    // 対照: 死域直下(dz-ε)は全ティック非活性→前θは毎回リセットされ lapは一度も動かない.
    let 列直下: Vec<(f64, f64)> = 列
        .iter()
        .map(|&(x, y)| {
            let dz下 = dz - 1e-9;
            let r比 = dz下 / dz;
            (x * r比, y * r比)
        })
        .collect();
    let mut 変b = Z変換器::既定();
    let mut 最終lap直下 = 0;
    for &(x, y) in &列直下 {
        最終lap直下 = 変b.変換(x, y).lap;
    }
    assert_eq!(
        最終lap直下, 0,
        "死域直下(全非活性)なのにlapが動いた (非活性は前θを毎回リセットしlap追跡しないはず) lap={}",
        最終lap直下
    );
}

// ---------- A7 deadzone不連続 (独立式で再検算) ----------
#[test]
fn a7_死域再正規化は独立式と一致する() {
    let p = Z変param::default();
    let mut 変 = Z変換器::既定();
    for &生r in &[0.081, 0.1, 0.3, 0.5, 0.9, 1.0] {
        let z = 変.変換(生r, 0.0);
        let 期待 = ((生r - p.死域) / (p.r上限 - p.死域)) * p.r上限;
        let 期待 = 期待.clamp(0.0, p.r上限);
        assert!(近(z.r, 期待, 1e-9), "生r={} z.r={} 期待={}", 生r, z.r, 期待);
    }
}

// ---------- A8 r>1 (方形域の角) ----------
#[test]
fn a8_正方形域の角でもr上限内() {
    let mut 変 = Z変換器::既定();
    let z = 変.変換(1.0, 1.0); // hypot=√2
    assert!(z.r <= 1.0 + 1e-12, "r={}", z.r);
    let mut 変2 = Z変換器::新(Z変param {
        r上限: 2.0,
        ..Default::default()
    });
    let z2 = 変2.変換(1.0, 1.0);
    assert!(z2.r <= 2.0 + 1e-12, "カスタムr上限超過 r={}", z2.r);
}

// ---------- A9 r=0=無中心 (theta自己申告できない事の確認) ----------
#[test]
fn a9_無状態のthetaは自己判別不能_無か併用が必須() {
    let mut 変 = Z変換器::既定();
    let 無z = 変.変換(0.0, 0.0); // atan2(0,0)相当の中心
    assert_eq!(無z.theta, 0.0, "初期無状態のθ (保持θ既定値)");
    assert!(無z.無か());
    // 構造的欠陥確認: 「θ=0, r>0 の正当な活性状態」と「θ=0, r=0 の無状態」は
    // theta field 単独では見分けがつかない — 無か()/r==0 の別途チェックが必須契約である事の実測.
    let mut 変2 = Z変換器::既定();
    let 活性z = 変2.変換(1.0, 0.0); // θ=0, r=1 (真に愛方位)
    assert_eq!(活性z.theta, 無z.theta, "θ値だけでは無/活性を区別不可能 (θ=0が両方に出現)");
    assert_ne!(活性z.r, 無z.r);
}

// ---------- A10 決定論再生 (既存z.rsに同旨testあり. 独立に再確認) ----------
#[test]
fn a10_同一列は二回走でbit一致() {
    let 列: Vec<(f64, f64)> = (0..500)
        .map(|i| {
            let θ = (i as f64) * 0.0173;
            let r = 0.05 + 0.9 * ((i as f64) * 0.011).sin().abs();
            (r * θ.cos(), r * θ.sin())
        })
        .collect();
    let a = 列変換(Z変param::default(), &列);
    let b = 列変換(Z変param::default(), &列);
    assert_eq!(a, b);
}

// ---------- A11 log精度損 (量子化再生 ≠ 生値z列) ----------
#[test]
fn a11_log量子化再生は生値z列とbit一致しない() {
    let 生x = 0.123456789_f64;
    let 生y = 0.987654321_f64;
    // 梯1 emit / z行 は "{:.4}" で量子化してlogへ書く (main.rs / z主.rs 実装参照).
    let ログx: f64 = format!("{:.4}", 生x).parse().unwrap();
    let ログy: f64 = format!("{:.4}", 生y).parse().unwrap();
    assert_ne!(生x, ログx, "量子化前提確認: 生値と量子化値が既に異なる");
    let mut a = Z変換器::既定();
    let mut b = Z変換器::既定();
    let za = a.変換(生x, 生y);
    let zb = b.変換(ログx, ログy);
    assert_ne!(
        za.theta, zb.theta,
        "生値z列と量子化再生z列がtheta bit一致してしまった (量子化幅次第では起こり得るが本例は不一致が期待値)"
    );
}

// ---------- A12 param網羅 (家数含め全knobがparam化されているか) ----------
#[test]
fn a12_z変paramは全既知定数をparam化している() {
    let p = Z変param::default();
    assert_eq!(p.死域, 0.08);
    assert_eq!(p.家数, 8);
    assert_eq!(p.r上限, 1.0);
    // 家数を変えると出力が変わる事= hardcodeでなくparamである実測証跡.
    let mut 変8 = Z変換器::新(Z変param {
        八家snap: true,
        家数: 8,
        ..Default::default()
    });
    let mut 変6 = Z変換器::新(Z変param {
        八家snap: true,
        家数: 6,
        ..Default::default()
    });
    let θ: f64 = 0.5;
    let z8 = 変8.変換(θ.cos(), θ.sin());
    let z6 = 変6.変換(θ.cos(), θ.sin());
    assert_ne!(z8.theta, z6.theta, "家数paramが出力に反映されていない");
}

// ---------- A13 f32/f64型境界 (実機路 vs 再生路の非対称) ----------
#[test]
fn a13_f32往復丸めは実機路のみに存在し再生路には無い() {
    // z主.rs 実機路: gilrs f32値 → `as f64` (main.rs/z主.rs実装参照).
    // 入力源.rs 再生路: logテキスト(4桁) → parse::<f64> (f32を一切経由しない).
    let 表示相当値 = 0.1234_f64;
    let 実機路相当 = (表示相当値 as f32) as f64; // f32往復
    assert_ne!(
        表示相当値, 実機路相当,
        "f32往復で丸め誤差が発生する事の確認 (実機路のみ通る変換, 再生路は通らない → 型経路非対称=A13)"
    );
}

// ---------- A14 非有限 (NaN/∞) ----------
#[test]
fn a14_nan入力はpanicせず無へ落ちる() {
    let mut 変 = Z変換器::既定();
    let z = 変.変換(f64::NAN, 0.0);
    assert!(z.無か(), "NaN入力が無に落ちなかった z={z:?}");
    assert_eq!(z.lap, 0, "NaN入力でlapが汚染された lap={}", z.lap);
}

#[test]
fn a14_nanは活性化後の状態を汚染しない() {
    let mut 変 = Z変換器::既定();
    // 一周させてlap=1を確立.
    let 列 = (0..=360)
        .map(|i| {
            let θ = 全環 * (i as f64) / 360.0;
            (0.9 * θ.cos(), 0.9 * θ.sin())
        })
        .collect::<Vec<_>>();
    for &(x, y) in &列 {
        変.変換(x, y);
    }
    assert_eq!(変.巻(), 1);
    let z_nan = 変.変換(f64::NAN, f64::NAN);
    assert!(z_nan.無か());
    assert!(!z_nan.lap.to_string().contains("NaN")); // lapはi64なのでNaN化は原理上不可, 実測確認のみ
    let z戻 = 変.変換(1.0, 0.0);
    assert!(!z戻.theta.is_nan(), "NaN混入後にthetaがNaN汚染 θ={}", z戻.theta);
}

#[test]
fn a14_無限入力はpanicせずr上限内に収まる() {
    let mut 変 = Z変換器::既定();
    let z = 変.変換(f64::INFINITY, 0.0);
    assert!(z.r.is_finite(), "r=inf漏れ r={}", z.r);
    assert!(z.r <= 1.0 + 1e-9, "r上限超過 r={}", z.r);
    assert!(z.theta.is_finite(), "θ非有限 θ={}", z.theta);
}

// ---------- A15 八家整合 (house() と e^{iθ}45°snap の独立path一致) ----------
#[test]
fn a15_polarのhouseとz家snapが独立pathで一致する() {
    let mut 不一致 = Vec::new();
    for i in 0..360 {
        let deg = i as f64;
        let rad = deg.to_radians();
        let s = stick_to_polar(rad.cos() as f32, rad.sin() as f32, 0.0);
        let house = match s.house {
            Some(h) => h,
            None => continue,
        };
        let 刻 = 全環 / 8.0;
        let z_snap = 家snap(環正規化(rad), 8);
        let house_angle = 環正規化((house as f64) * 刻);
        let 差= (z_snap - house_angle).abs();
        let 差 = 差.min((差 - 全環).abs());
        if 差 > 1e-6 {
            不一致.push((deg, house, house_angle, z_snap));
        }
    }
    assert!(
        不一致.is_empty(),
        "house()とz家snapが不一致 (先頭5件): {:?}",
        &不一致[..不一致.len().min(5)]
    );
}
