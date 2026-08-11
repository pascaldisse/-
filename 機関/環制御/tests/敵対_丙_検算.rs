//! 批根丙 独立検算 — 子鐘(@solas)申告の再測定. 主張を鵜呑みにせず自路で測る.
use wa::z::{家snap, Z変param, Z変換器, 全環};

fn 極(r: f64, deg: f64) -> (f64, f64) {
    let t = deg.to_radians();
    (r * t.cos(), r * t.sin())
}

/// solas #2b「静止微振動でlapが暴走」→ 実際は ±1 の振動か、単調drift(暴走)か.
#[test]
fn 丙_縫目微振動1000回のlap範囲を測る() {
    let mut v = Z変換器::既定();
    let mut 最小 = 0i64;
    let mut 最大 = 0i64;
    for i in 0..1000 {
        let deg = if i % 2 == 0 { 179.999 } else { 180.001 };
        let (x, y) = 極(0.9, deg);
        let z = v.変換(x, y);
        最小 = 最小.min(z.lap);
        最大 = 最大.max(z.lap);
    }
    println!("chatter lap範囲 = [{最小}, {最大}]");
    // D1是正 (敵対審査乙.3, 08-11): 旧pinは<=1 (有界フリッカーを容認)だったが,
    // 縫目ヒステリシス(既定0.005rad)が本ジッタ幅(0.001°≈1.7e-5rad)を完全吸収し, lapは一度も
    // 動かなくなった (本値を ==0 へ引き締め, 欠陥消滅の記録とする).
    assert_eq!(最大 - 最小, 0, "D1が復活している (微振動でlapが動いた): 幅={}", 最大 - 最小);
}

/// 縫目を跨いでも **総角 θ+2π·lap** は連続か (下流はθ単体でなく総角を使うべき、の検証).
#[test]
fn 丙_総角は縫目で連続() {
    let mut v = Z変換器::既定();
    let mut 前総 = f64::NAN;
    let mut 最大跳 = 0.0f64;
    for i in 0..=720 {
        let deg = i as f64 * 0.5; // 0.5°刻で2周
        let (x, y) = 極(0.9, deg);
        let z = v.変換(x, y);
        let 総 = z.theta + 全環 * z.lap as f64;
        if 前総.is_finite() {
            最大跳 = 最大跳.max((総 - 前総).abs());
        }
        前総 = 総;
    }
    println!("総角の最大tick差 = {最大跳} rad (刻=0.00873 rad)");
    assert!(最大跳 < 0.02, "総角が縫目で跳んだ: {最大跳}");
}

/// solas #7「202.5°だけ下位家へ反転」の再測定.
/// D2是正 (敵対審査乙.3, 08-11): 家snapを(-π,π]符号域のround()から[0,2π)正域のfloor()
/// に修正した後は, 202.5°の例外(家数のみ下位へ倒れていた)も消え, 8境界全数一貫するはず —
/// 旧pin(<=1, 既知例外を容認)を ==0 (例外零) へ引き締めた (欠陥消滅の記録).
#[test]
fn 丙_八家境界の帰属を全数表示() {
    let 家数 = Z変param::default().家数;
    let mut 例外 = vec![];
    for k in 0..8 {
        let deg = 22.5 + 45.0 * k as f64;
        let θ = deg.to_radians();
        let s = 家snap(if θ > std::f64::consts::PI { θ - 全環 } else { θ }, 家数);
        let 家 = ((s / (全環 / 家数 as f64)).round() as i64).rem_euclid(家数 as i64);
        let 上位 = ((k + 1) % 8) as i64;
        println!("境界{deg}° → 家{家} (上位={上位})");
        if 家 != 上位 {
            例外.push((deg, 家));
        }
    }
    println!("規約外れ = {例外:?}");
    assert!(例外.is_empty(), "D2が復活している — 境界tie-breakが非一貫: {例外:?}");
}
