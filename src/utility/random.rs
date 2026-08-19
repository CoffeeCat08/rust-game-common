// common/src/utility/random.rs

use rand::distr::{Distribution, weighted::WeightedIndex};
use std::ops::RangeInclusive;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RandomError {
    // good
    NoSelectionMade,
    // bad
    InvalidWeights,
}

/// 指定された確率に基づき、成功か失敗かを判定する
/// ```rust
/// # use rust_game_common::utility::random::chance;
/// if chance(0.32){
///   println!("32%の確率で成功します");
/// }
/// ```
///
/// # Arguments
/// * `success_rate` - 成功確率 (0.0 ～ 1.0)
///
/// # Returns
/// 成功した場合は `true`、失敗した場合は `false`
pub fn chance(success_rate: f32) -> bool {
    rand::random_bool(success_rate as f64)
}

/// 基準値に対して、指定された範囲の乱数倍率を乗算して返す
///
/// 主にダメージ計算のゆらぎ（0.8倍 ～ 1.25倍など）に使用する
///
/// # Examples
/// ```
/// # use rust_game_common::utility::random::range;
/// let damage = range(100.0, 0.8..=1.25);
/// ```
///
/// # Arguments
/// * `base_value` - 基準となる数値
/// * `range` - 乗算する倍率の範囲
///
/// # Returns
/// 乱数を乗算した結果
pub fn range(base_value: f32, range: RangeInclusive<f32>) -> f32 {
    base_value * rand::random_range(range)
}

/// リストを先頭から順番に抽選し、最初に当選した要素のインデックスを返す
///
/// 「毒判定、次に麻痺判定」のように、各項目が独立して判定されるケースに使用する
///
/// # Arguments
/// * `list` - 抽選したい数値のVec
///
/// # Returns
/// 当選した要素のインデックス番号 (`usize`)
///
/// # Errors
/// リスト内のすべての抽選に外れた場合、`RandomError::NoSelectionMade` を返す
pub fn weighted_choice_or_failed(list: &Vec<f32>) -> Result<usize, RandomError> {
    for (i, &rate) in list.iter().enumerate() {
        if chance(rate) {
            return Ok(i);
        }
    }

    Err(RandomError::NoSelectionMade)
}

/// リスト内のすべての要素に対して独立して抽選を行い、当選したすべてのインデックスを返す
///
/// 「一度の判定で複数のアイテムを同時にドロップする」ようなケースに使用する
///
/// # Arguments
/// * `list` - 抽選したい数値のVec
///
/// # Returns
/// 当選した全インデックスのリスト (`Vec<usize>`)
///
/// # Errors
/// 一つも当選しなかった場合、`RandomError::NoSelectionMade` を返す
pub fn weighted_choice_all(list: &Vec<f32>) -> Result<Vec<usize>, RandomError> {
    let result: Vec<usize> = list
        .iter()
        .enumerate()
        .filter(|&(_, &rate)| chance(rate))
        .map(|(i, _)| i)
        .collect();

    if result.is_empty() {
        Err(RandomError::NoSelectionMade)
    } else {
        Ok(result)
    }
}

/// 重みの比重に基づき、リストの中から一つの要素を抽選する
///
/// 合計値に対する比重で抽選を行うため、必ずどれか一つを選びたい場合（ドロップテーブルなど）に使用する
///
/// # Arguments
/// * `weights` - 各項目の重みのスライス
///
/// # Returns
/// 抽選された項目のインデックス番号 (`usize`)
///
/// # Errors
/// - 重みの合計が0以下の場合、またはリストが空の場合に `RandomError::InvalidWeights` を返す
pub fn weighted_choice(weights: &[f32]) -> Result<usize, RandomError> {
    let mut rng = rand::rng(); // 最新のRNG取得方法 [3]

    // WeightedIndex を作成重みの合計が0以下だとエラーになるため Result で扱う
    let dist = WeightedIndex::new(weights).map_err(|_| RandomError::InvalidWeights)?; // [4], [6]

    // 抽選を実行
    Ok(dist.sample(&mut rng))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chance_boundaries() {
        assert!(chance(1.0)); // 100%は必ず成功
        assert!(!chance(0.0)); // 0%は必ず失敗
    }

    // 1万回ほど試行して、結果が統計的な期待値(例えば50%なら5000回前後)に収まっているかを確認
    // 運が悪いと、失敗に終わるテストです

    #[test]
    fn test_chance_statistics() {
        let mut success_count = 0;
        let trials = 10000;
        for _ in 0..trials {
            if chance(0.5) {
                success_count += 1;
            }
        }
        // 50%（5000回）に近いかどうか（誤差範囲を許容する）
        assert!(success_count > 4500 && success_count < 5500);
    }

    #[test]
    fn test_weighted_choice_error() {
        let empty_weights: Vec<f32> = vec![];
        let result = weighted_choice(&empty_weights);
        assert_eq!(result, Err(RandomError::InvalidWeights));
    }
}
