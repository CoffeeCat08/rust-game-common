// common/src/model/status.rs

//! # ステータス管理モジュール
//!
//! このモジュールは、時間経過による増減や即時変化を、
//! 安全な数値範囲内で制御する `BoundedStatus` を提供します。
//!
//! ## 主な機能
//! - `StatusError`: 列挙型
//!   * MinLimitReached: 最小値に到達したというシグナル ex:HPが0になった。
//!   * MaxLimitReached: 最大値に到達したというシグナル ex:HPが全回復した。
//! - `BoundedStatus`: 構造体。
//!   * `new`: 新しいstatusを作成する
//!   * `tick_subtract`: 時間経過による減産処理
//!   * `tick_add`: 時間経過による加算処理
//!   * `apply_action_subtract`: actionによる減産処理. ex:攻撃する等
//!   * `apply_action_add`: actionによる加算処理

use std::time::Duration;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StatusError {
    MinLimitReached,
    MaxLimitReached,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundedStatus {
    pub current: f32,
    pub min: f32,
    pub max: f32,
    pub weight: f32, // game内1秒の変化量
}

impl BoundedStatus {
    /// 新しいstatusを構築する
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::status::BoundedStatus;
    ///
    /// let hit_point = BoundedStatus::new(100.0,0.0,150.0,0.1);  
    /// ```
    ///
    /// # Arguments
    /// * `current` - `f32` statusの現在値
    /// * `min` - `f32` statusの最小値
    /// * `max` - `f32` statusの最大値
    /// * `weight` - `f32` statusの時間経過による変化率
    pub fn new(current: f32, min: f32, max: f32, weight: f32) -> Self {
        Self {
            current,
            min,
            max,
            weight,
        }
    }

    /// 時間経過時のstatsuの減算変化
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::status::{BoundedStatus,StatusError};
    /// # use std::time::Duration;
    ///
    /// let mut hit_point = BoundedStatus::new(100.0,0.0,150.0,0.1);
    /// match hit_point.tick_subtract(Duration::from_secs(120)){
    ///   Err(StatusError::MinLimitReached) => println!("傷の悪化で、死亡しました"),
    ///   _ => {},
    /// }
    /// ```
    ///
    /// # Arguments
    /// * `duration_time`- `Duration` 経過した時間
    ///
    /// # Errors
    /// `duration_time`に`weight`を乗算した値を減算した際、最小値に届いた場合に`MinLimitReached`を返す
    pub fn tick_subtract(&mut self, duration_time: Duration) -> Result<(), StatusError> {
        let damage = duration_time.as_secs_f32() * self.weight;
        self.current -= damage;

        if self.current <= self.min {
            self.current = self.min;
            Err(StatusError::MinLimitReached)
        } else {
            Ok(())
        }
    }

    /// 時間経過時のstatsuの加算変化
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::status::{BoundedStatus,StatusError};
    /// # use std::time::Duration;
    ///
    /// let mut poison = BoundedStatus::new(0.0,0.0,120.0,1.0);
    /// match poison.tick_add(Duration::from_secs(1200)){
    ///   Err(StatusError::MaxLimitReached) => println!("毒を自然治癒しました"),
    ///   _ => {},
    /// }
    /// ```
    ///
    /// # Arguments
    /// * `duration_time` - `Duration` 経過した時間
    ///
    /// # Errors
    /// `duration_time`に`weight`を乗算した値を加算した際、最大値に届いた場合に`MaxLimitReached`を返す
    pub fn tick_add(&mut self, duration_time: Duration) -> Result<(), StatusError> {
        let heal = duration_time.as_secs_f32() * self.weight;
        self.current += heal;

        if self.current >= self.max {
            self.current = self.max;
            Err(StatusError::MaxLimitReached)
        } else {
            Ok(())
        }
    }

    /// 攻撃等のactionによるstatsuの減算変化
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::status::{BoundedStatus,StatusError};
    /// # use std::time::Duration;
    ///
    /// let mut hit_point = BoundedStatus::new(100.0,0.0,120.0,1.0);
    /// match hit_point.apply_action_subtract(100.0){
    ///   Err(StatusError::MinLimitReached) => println!("攻撃を受けて死亡しました"),
    ///   _ => {},
    /// }
    /// ```
    ///
    /// # Arguments
    /// * `amount` - `f32` 直接の変化量
    ///
    /// # Errors
    /// `amount`を減算した際、最小値に届いた場合に`MinLimitReached`を返す
    pub fn apply_action_subtract(&mut self, amount: f32) -> Result<(), StatusError> {
        self.current -= amount;

        if self.current <= self.min {
            self.current = self.min;
            Err(StatusError::MinLimitReached)
        } else {
            Ok(())
        }
    }

    /// 回復等のactionによるstatsuの加算変化
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::status::{BoundedStatus,StatusError};
    /// # use std::time::Duration;
    ///
    /// let mut poison = BoundedStatus::new(40.0,0.0,40.0,1.0);
    /// match poison.apply_action_add(100.0){
    ///   Err(StatusError::MaxLimitReached) => println!("解毒薬で毒状態が回復しました"),
    ///   _ => {},
    /// }
    /// ```
    ///
    /// # Arguments
    /// * `amount` - `f32` 直接の変化量
    ///
    /// # Errors
    /// `amount`を加算した際、最大値に届いた場合に`MaxLimitReached`を返す
    pub fn apply_action_add(&mut self, amount: f32) -> Result<(), StatusError> {
        self.current += amount;

        if self.current >= self.max {
            self.current = self.max;
            Err(StatusError::MaxLimitReached)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_subtract() {
        let mut test = BoundedStatus::new(100.0, 0.0, 150.0, 10.0);
        let duration = Duration::from_secs(5);
        let mut result = test.tick_subtract(duration);
        assert_eq!(test.current, 50.0);
        assert_eq!(result, Ok(()));

        result = test.tick_subtract(duration);
        assert_eq!(test.current, 0.0);
        assert_eq!(result, Err(StatusError::MinLimitReached));
    }

    #[test]
    fn test_tick_add() {
        let mut test = BoundedStatus::new(100.0, 0.0, 150.0, 5.0);
        let duration = Duration::from_secs(5);
        let mut result = test.tick_add(duration);
        assert_eq!(test.current, 125.0);
        assert_eq!(result, Ok(()));

        result = test.tick_add(duration);
        assert_eq!(test.current, 150.0);
        assert_eq!(result, Err(StatusError::MaxLimitReached));
    }
    #[test]
    fn test_apply_action_subtract() {
        let mut test = BoundedStatus::new(100.0, 0.0, 150.0, 10.0);
        let mut result = test.apply_action_subtract(50.0);
        assert_eq!(test.current, 50.0);
        assert_eq!(result, Ok(()));

        result = test.apply_action_subtract(50.0);
        assert_eq!(test.current, 0.0);
        assert_eq!(result, Err(StatusError::MinLimitReached));
    }
    #[test]
    fn test_apply_action_add() {
        let mut test = BoundedStatus::new(100.0, 0.0, 150.0, 10.0);
        let mut result = test.apply_action_add(25.0);
        assert_eq!(test.current, 125.0);
        assert_eq!(result, Ok(()));

        result = test.apply_action_add(25.0);
        assert_eq!(test.current, 150.0);
        assert_eq!(result, Err(StatusError::MaxLimitReached));
    }
}
