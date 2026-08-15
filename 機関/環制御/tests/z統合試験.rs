//! 梯2 統合試験 — 実proof (proof/環制御/入力log.txt) を外部から log解析→列変換 して検査.
//! 文書/環統合.md §主座標. src/* は不変のまま — 本fileはlibの公開契約のみを叩く.
#![allow(mixed_script_confusables)]

use std::fs;
use std::path::PathBuf;

use wa::z::{Z変param, Z変換器, 列変換, 家snap, 環正規化, Z};
use wa::z::{半環, LOVE};
use wa::入力源::log解析;

/// 実測回帰pin — proof/環制御/入力log.txt (900標本, 既定param) を通した現測定の終巻.
/// hardcode禁の例外的常数 (回帰基準値そのものが本testの主張) — 名前つき宣言で明示.
const 終巻期待値: i64 = -2;

/// 八家snap 家数既定 (Z変param::default().家数 と同値だが直接計算検証用に明示).
const 家数既定: u32 = 8;

/// 死域を段階的に上げた時の r=0 標本数単調性を見る候補列 (昇順, 全て r上限未満).
const 死域候補: [f64; 5] = [0.05, 0.08, 0.15, 0.30, 0.50];

/// 実proof log の絶対path (機関/環制御/../../proof/環制御/入力log.txt — z主.rs既定再生元と同一規約).
fn 実proof路() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proof/環制御/入力log.txt")
}

/// 実proof logを読み log解析 に通し、(x, y) 生値列へ薄く写す (外部io + lib解析口のみ使用).
fn 実log列() -> Vec<(f64, f64)> {
    let path = 実proof路();
    let 本文 = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("実proof log読込失敗 path={} err={e}", path.display()));
    let 標本 = log解析(&本文);
    assert!(
        !標本.is_empty(),
        "実proof logにTICK行なし path={}",
        path.display()
    );
    標本.iter().map(|s| (s.x, s.y)).collect()
}

#[test]
fn 実logの決定論再生はbit同一() {
    let 列 = 実log列();
    let a = 列変換(Z変param::default(), &列);
    let b = 列変換(Z変param::default(), &列);
    assert_eq!(a, b, "同じ入力列から二度目で異なるz列が生じた — 決定論違反");
}

#[test]
fn 実logの決定論再生は手動loopとも一致する() {
    // 列変換一つに閉じた検査ではない独立径路: Z変換器を手動でtickごとに回して照合する
    // (列変換自体の実装欠陥がある場合でも二重呼出しだけでは検出できない為).
    let 列 = 実log列();
    let 一括 = 列変換(Z変param::default(), &列);
    let mut 変 = Z変換器::新(Z変param::default());
    let 手動: Vec<Z> = 列.iter().map(|&(x, y)| 変.変換(x, y)).collect();
    assert_eq!(
        一括, 手動,
        "列変換と手動loopの結果が食い違う — 状態機械の非決定性の疑い"
    );
}

#[test]
fn 実logの終巻はマイナス二で回帰pinされる() {
    let 列 = 実log列();
    let z列 = 列変換(Z変param::default(), &列);
    let 終 = z列.last().expect("実proof logは空でない").lap;
    assert_eq!(終, 終巻期待値, "終巻が実測値から動いた 終={終}");
}

#[test]
fn 実logの全標本でtheta_r_lap契約を満たす() {
    let 列 = 実log列();
    let z列 = 列変換(Z変param::default(), &列);

    for (i, z) in z列.iter().enumerate() {
        // θ ∈ (-π, π] — 環正規化との往復で独立に確認 (idempotent性=既に正規化済の証).
        assert!(
            z.theta > -半環 - 1e-9 && z.theta <= 半環 + 1e-9,
            "θ範囲外 i={i} θ={}",
            z.theta
        );
        assert!(
            (z.theta - 環正規化(z.theta)).abs() < 1e-12,
            "θが未正規化 i={i} θ={}",
            z.theta
        );
        // r ∈ [0, 1] (r上限既定=LOVE).
        assert!(z.r >= 0.0 && z.r <= LOVE + 1e-9, "r範囲外 i={i} r={}", z.r);
    }

    // lap は1tickで高々±1しか動かぬ (±π横断は一tickにつき一度が契約前提).
    for (i, w) in z列.windows(2).enumerate() {
        let 差 = (w[1].lap - w[0].lap).abs();
        assert!(
            差 <= 1,
            "lapが1tickで2以上動いた i={i} {:?} → {:?}",
            w[0],
            w[1]
        );
    }
}

#[test]
fn 八家snap_onoffでlap列は完全一致する() {
    let 列 = 実log列();
    let 生 = 列変換(Z変param::default(), &列);
    let snap = 列変換(
        Z変param {
            八家snap: true,
            ..Default::default()
        },
        &列,
    );
    let 生lap: Vec<i64> = 生.iter().map(|z| z.lap).collect();
    let snaplap: Vec<i64> = snap.iter().map(|z| z.lap).collect();
    assert_eq!(
        生lap, snaplap,
        "八家snap on/offでlap列が食い違う — snapが巻計数を汚した"
    );
}

#[test]
fn 八家snapのthetaは家snap直接計算と独立に一致する() {
    // 独立径路: Z変換器の内部を経ずraw(x,y)からその場で 生θ→家snap を再計算し照合する.
    let 列 = 実log列();
    let param = Z変param {
        八家snap: true,
        ..Default::default()
    };
    let z列 = 列変換(param, &列);
    let 死域 = param.死域;

    for (i, (&(x, y), z)) in 列.iter().zip(z列.iter()).enumerate() {
        let 生r = x.hypot(y);
        if 生r < 死域 {
            continue; // 死域中は保持θを返す仕様 — このtestは活性標本のみ照合する.
        }
        let 生θ = 環正規化(y.atan2(x));
        let 期待θ = 家snap(生θ, 家数既定);
        assert!(
            (z.theta - 期待θ).abs() < 1e-9,
            "i={i} θ={} 期待={期待θ}",
            z.theta
        );
    }
}

#[test]
fn 死域を上げるとr零標本数は単調に増える() {
    let 列 = 実log列();
    let mut 前回: Option<usize> = None;
    for &死域 in 死域候補.iter() {
        let param = Z変param {
            死域,
            ..Default::default()
        };
        let z列 = 列変換(param, &列);
        let count = z列.iter().filter(|z| z.r == 0.0).count();
        if let Some(前) = 前回 {
            assert!(
                count >= 前,
                "死域={死域} で r=0標本数が減少した 前={前} 今={count}"
            );
        }
        前回 = Some(count);
    }
}
