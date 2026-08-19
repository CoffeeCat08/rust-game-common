// common/src/utility/time.rs

//! # ゲーム時間管理モジュール
//!
//! このモジュールは、現実時間からゲーム内時間への換算、
//! およびポーズ（一時停止）に対応した経過時間の計算を提供します。
//!
//! ## 主な機能
//! - `GameTime`: 構造体。
//!   * init(): 初期化
//!   * update(): 最後のupdateから経過時間を返す
//!   * reset(): 一時停止など、現実世界の時間経過をゲーム内で反映したくないときに使う
//! - `convert_to_game_duration`: ゲーム内で経過した時間を返す関数
//! - `what_time_is_it_now_in_world`: ゲーム内での24時間表記のための値を返す関数

use std::time::{Duration, Instant};

pub struct GameTime {
    start_time: Instant,
    last_update_time: Instant,
}

/// Gameを開始した際の初期化
impl GameTime {
    /// Gameの開始時等の初期化
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    ///
    /// let mut game_time = GameTime::init();
    /// ```
    ///
    /// # Returns
    /// 初期化されたGameTimeのインスタンス
    pub fn init() -> Self {
        let now = Instant::now();

        Self {
            start_time: now,
            last_update_time: now,
        }
    }

    /// 前回の処理から経過した時間を測定して返す関数
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    /// let mut game_time = GameTime::init();
    ///
    /// let current_duration = game_time.update();
    /// ```
    ///
    /// # Returns
    /// 経過した時間を返す(Duration)
    pub fn update(&mut self) -> Duration {
        let now = Instant::now();

        let current_duration = now.duration_since(self.last_update_time);
        self.last_update_time = now;

        current_duration
    }

    /// 経過した時間の内、Game内では反映したくない場合、これを使用して経過時間を安全に破棄する
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    /// let mut game_time = GameTime::init();
    ///
    /// game_time.reset();
    /// ```
    pub fn reset(&mut self) {
        self.last_update_time = Instant::now();
    }

    /// ゲーム開始（init）時からの総経過時間を取得する
    pub fn total_elapsed(&self) -> Duration {
        // 内部フィールドには impl ブロック内からのみアクセス可能
        Instant::now().duration_since(self.start_time)
    }

    /// 前回の update 時の時刻を取得する（必要であれば）
    pub fn last_update_at(&self) -> Instant {
        self.last_update_time
    }
}

/// real_durationを元にtime_weightに応じて、game時間に換算する関数
/// ex: in_game_world : real_world = 24h : 24h 同じなら、time_weightは"1.0"
/// ex: in_game_world : real_world = 24h : 3h 同じなら、time_weightは"8.0"
/// `time_weight`は,各projectでconst定義ほうが良い。
///
/// # Examples
/// ```rust
/// # use rust_game_common::utility::time::convert_to_game_duration;
/// # use std::time::Duration;
///
/// let game_duration = convert_to_game_duration(Duration::from_secs(60),0.8);
/// ```
///
/// # Arguments
/// * `real_duration` - `Duration` 現実世界で経過した時間
/// * `time_weight` - `f64` game内で経過した時間にする重み
///
/// # Returns
/// game内で経過した時間(Duration)
pub fn convert_to_game_duration(real_duration: Duration, time_weight: f64) -> Duration {
    real_duration.mul_f64(time_weight)
}

// TODO:累積時間を用意して、そちらで更新するようにする。
// 現状で行けば、経過時間を最初の基準値に足す形になるため、時間が経過していないような状態になる。
/// Game内での24時間を返す関数
///
/// # Examples
/// ```rust
/// # use rust_game_common::utility::time::what_time_is_it_now_in_world;
/// # use std::time::Duration;
///
/// let (hour,minute) = what_time_is_it_now_in_world(8,Duration::from_secs(4500));
/// ```
///
/// # Arguments
/// * `start_time` - `u32` 前回の時間
/// * `game_duration` - `Duration` ゲーム内での経過時間
///
/// # Returns
/// start_timeにgame_durationを加算し、Game内の24時間を(時間,分)のタプルで返す。
pub fn what_time_is_it_now_in_world(start_time: u32, game_duration: Duration) -> (u32, u32) {
    let total_game_minutes = game_duration.as_secs() / 60;

    let total_hour = start_time + (total_game_minutes / 60) as u32;
    let current_hour = total_hour % 24;
    let current_minute = (total_game_minutes % 60) as u32;

    (current_hour, current_minute)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_game_duration() {
        let real_time = Duration::from_secs(60); // 現実の1分
        let weight = 8.0; // 8倍速

        let game_time = convert_to_game_duration(real_time, weight);

        // ゲーム内では 60s * 8 = 480s（8分）になっているか？
        assert_eq!(game_time.as_secs(), 480);
    }

    #[test]
    fn test_what_time_is_it_now_in_world() {
        // 朝8時スタートで、ゲーム内で「25時間（1500分）」経過した場合
        let start_hour = 8;
        let game_duration = Duration::from_secs(25 * 3600);

        let (hour, minute) = what_time_is_it_now_in_world(start_hour, game_duration);

        // 8時 + 25時間 = 33時 ＝＞ 24時間でループして「9時0分」になるか？
        assert_eq!(hour, 9);
        assert_eq!(minute, 0);
    }
}
