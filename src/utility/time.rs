// common/src/utility/time.rs

//! # ゲーム時間管理モジュール
//!
//! このモジュールは、現実時間からゲーム内時間への換算、
//! およびポーズ（一時停止）に対応した経過時間の計算を提供します。
//!
//! ## 主な機能
//! - `GameTime`: 構造体。
//!   * `init`: 初期化。時間倍率（`weight`）と開始時刻（`start_hour`）を設定します。
//!   * `convert_to_game_duration`: 現実世界の経過時間をゲーム内の経過時間に変換します。
//!   * `convert_to_real_duration`: ゲーム内の経過時間を現実世界の経過時間に変換します。
//!   * `advance_by_real_time`: 指定した「現実時間」だけ、ゲーム内の累積時間を強制的に進めます。
//!   * `advance_by_game_time`: 指定した「ゲーム時間」だけ、ゲーム内の累積時間を強制的に進めます。
//!   * `advance_to_game_time`: ゲーム内の指定した「時刻（時, 分）」まで時間を進めます。
//!   * `what_time_is_it_now_in_world`: ゲーム内での現在の時刻を `(時, 分)` のタプルで返します。
//!   * `update`: 前回の `update` 呼び出しからの経過時間を測定し、累積経過時間を更新します。
//!   * `reset`: 一時停止の解除時などに、ポーズ中の経過時間をゲーム内時間に反映させずに破棄します。

use std::time::{Duration, Instant};

pub struct GameTime {
    _start_time: Instant,
    last_update_time: Instant,
    total_real_duration: Duration,
    weight: f64,
    start_hour: u32,
}

/// Gameを開始した際の初期化
impl GameTime {
    /// ゲームの開始時（インスタンス化）の初期化を行います。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    /// let game_time = GameTime::init(8.0, 8);
    /// ```
    ///
    /// # Arguments
    /// * `weight` - `f64` ゲーム時間の進む倍率
    /// * `start_hour` - `u32` ゲーム開始時の時刻（時）
    ///
    /// # Returns
    /// 初期化された `GameTime` インスタンス
    pub fn init(weight: f64, start_hour: u32) -> Self {
        let now = Instant::now();

        Self {
            _start_time: now,
            last_update_time: now,
            total_real_duration: Duration::from_secs(0),
            weight: weight,
            start_hour: start_hour,
        }
    }

    /// 現実の経過時間（`real_duration`）を、設定された倍率（`weight`）に基づいてゲーム内経過時間に変換します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    /// # use std::time::Duration;
    /// let game_time = GameTime::init(8.0, 8);
    /// let game_duration = game_time.convert_to_game_duration(Duration::from_secs(60));
    /// assert_eq!(game_duration.as_secs(), 480);
    /// ```
    ///
    /// # Arguments
    /// * `real_duration` - `Duration` 現実世界の経過時間
    ///
    /// # Returns
    /// ゲーム内世界の経過時間（`Duration`）
    pub fn convert_to_game_duration(&self, real_duration: Duration) -> Duration {
        real_duration.mul_f64(self.weight)
    }

    /// ゲーム内世界の経過時間（`game_duration`）を、設定された倍率（`weight`）に基づいて現実世界の経過時間に変換（逆換算）します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    /// # use std::time::Duration;
    /// let game_time = GameTime::init(8.0, 8);
    /// let real_duration = game_time.convert_to_real_duration(Duration::from_secs(480));
    /// assert_eq!(real_duration.as_secs(), 60);
    /// ```
    ///
    /// # Arguments
    /// * `game_duration` - `Duration` ゲーム内世界の経過時間
    ///
    /// # Returns
    /// 現実世界の経過時間（`Duration`）
    pub fn convert_to_real_duration(&self, game_duration: Duration) -> Duration {
        game_duration.div_f64(self.weight)
    }

    /// 指定した時間（現実時間）だけ、ゲーム内の累積時間を強制的に進めます。
    /// デバッグやテスト、あるいは現実の特定の時間経過を強制反映したい場合に役立ちます。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    /// # use std::time::Duration;
    /// let mut game_time = GameTime::init(8.0, 8);
    /// game_time.advance_by_real_time(Duration::from_secs(3600));
    /// ```
    ///
    /// # Arguments
    /// * `real_duration` - `Duration` 強制的に進めたい現実世界の時間
    pub fn advance_by_real_time(&mut self, real_duration: Duration) {
        self.total_real_duration += real_duration;
    }

    /// 指定した時間（ゲーム時間）だけ、ゲーム内の累積時間を強制的に進めます。
    /// 内部的に設定された倍率に基づいて現実時間に逆換算され、累積時間に足し込まれます。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    /// # use std::time::Duration;
    /// let mut game_time = GameTime::init(8.0, 8);
    /// // ゲーム内で2時間進める
    /// game_time.advance_by_game_time(Duration::from_secs(2 * 3600));
    /// ```
    ///
    /// # Arguments
    /// * `game_duration` - `Duration` 強制的に進めたいゲーム世界の時間
    pub fn advance_by_game_time(&mut self, game_duration: Duration) {
        let real_duration = self.convert_to_real_duration(game_duration);
        self.total_real_duration += real_duration;
    }

    /// ゲーム内での現在の時刻を `(時, 分)` のタプルで返します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    /// let game_time = GameTime::init(8.0, 8);
    /// let (hour, minute) = game_time.what_time_is_it_now_in_world();
    /// ```
    ///
    /// # Returns
    /// 現在のゲーム内時刻 `(時間, 分)` のタプル
    pub fn what_time_is_it_now_in_world(&self) -> (u32, u32) {
        let total_game_time = self.convert_to_game_duration(self.total_real_duration);
        let total_game_minutes = total_game_time.as_secs() / 60;

        let total_hour = self.start_hour + (total_game_minutes / 60) as u32;
        let current_hour = total_hour % 24;
        let current_minute = (total_game_minutes % 60) as u32;

        (current_hour, current_minute)
    }

    /// 指定したゲーム内の時刻 `(target_hour, target_minute)` まで時間を強制的に進めます。
    ///
    /// 宿泊（宿屋）やイベントの暗転などで、ゲーム内の特定の時間まで一気にスキップさせたい場合に非常に便利です。
    /// 目標時刻が現在の時刻よりも前の場合は、翌日のその時刻まで時間を進めます。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    /// let mut game_time = GameTime::init(8.0, 8); // 8:00 スタート
    /// game_time.advance_to_game_time(10, 0); // 10:00 まで進める
    /// let (h, m) = game_time.what_time_is_it_now_in_world();
    /// assert_eq!(h, 10);
    /// assert_eq!(m, 0);
    /// ```
    ///
    /// # Arguments
    /// * `target_hour` - 目標のゲーム内時刻（時）
    /// * `target_minute` - 目標のゲーム内時刻（分）
    pub fn advance_to_game_time(&mut self, target_hour: u32, target_minute: u32) {
        let (current_hour, current_minute) = self.what_time_is_it_now_in_world();

        let current_total_minutes = current_hour * 60 + current_minute;
        let target_total_minutes = target_hour * 60 + target_minute;

        let diff_game_minutes = if target_total_minutes > current_total_minutes {
            target_total_minutes - current_total_minutes
        } else {
            (1440 + target_total_minutes) - current_total_minutes
        };

        let diff_game_duration = Duration::from_secs(diff_game_minutes as u64 * 60);
        self.advance_by_game_time(diff_game_duration);
    }

    /// 前回の `update` 呼び出しからの現実の経過時間（デルタタイム）を測定して返します。
    ///
    /// 測定された現実の経過時間は自動的に `total_real_duration` に加算（累積）されます。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    /// let mut game_time = GameTime::init(8.0, 8);
    /// let delta = game_time.update();
    /// ```
    ///
    /// # Returns
    /// 前回の `update`（または初期化/リセット）からの経過時間（`Duration`）
    pub fn update(&mut self) -> Duration {
        let now = Instant::now();

        let current_duration = now.duration_since(self.last_update_time);
        self.total_real_duration += current_duration;
        self.last_update_time = now;

        current_duration
    }

    /// 経過した時間のうち、ゲーム内に反映したくない（ポーズ中の経過時間など）場合に呼び出します。
    ///
    /// 最後に測定した時間から現在までの経過時間を累積させずに安全に破棄します。
    /// 一時停止解除（ポーズから復帰する瞬間）にこのメソッドを呼ぶことで、時間が跳ね飛ぶバグを防げます。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::utility::time::GameTime;
    /// let mut game_time = GameTime::init(8.0, 8);
    /// // ポーズ解除！
    /// game_time.reset();
    /// ```
    pub fn reset(&mut self) {
        self.last_update_time = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_game_duration() {
        let real_time = Duration::from_secs(60); // 現実の1分
        let weight = 8.0; // 8倍速
        let test = GameTime::init(weight, 8);

        let game_time = test.convert_to_game_duration(real_time);

        // ゲーム内では 60s * 8 = 480s（8分）になっているか？
        assert_eq!(game_time.as_secs(), 480);
    }

    #[test]
    fn test_convert_to_real_duration() {
        let game_time_dur = Duration::from_secs(480); // ゲーム内8分
        let weight = 8.0; // 8倍速
        let test = GameTime::init(weight, 8);

        let real_time = test.convert_to_real_duration(game_time_dur);

        // 現実時間では 480s / 8 = 60s（1分）になっているか？
        assert_eq!(real_time.as_secs(), 60);
    }

    #[test]
    fn test_what_time_is_it_now_in_world() {
        let weight = 8.0; // 8倍速
        let mut test = GameTime::init(weight, 8); // 朝8時スタート

        // ゲーム内で「25時間（25 * 3600秒 = 90,000秒）」経過させる
        let game_duration = Duration::from_secs(25 * 3600);
        test.advance_by_game_time(game_duration);

        let (hour, minute) = test.what_time_is_it_now_in_world();

        // 8時 + 25時間 = 33時 ＝＞ 24時間でループして「9時0分」になるか？
        assert_eq!(hour, 9);
        assert_eq!(minute, 0);
    }

    #[test]
    fn test_advance_to_game_time() {
        let mut test = GameTime::init(8.0, 8); // 朝8:00スタート

        // 1. 同日の 10:30 まで進める
        test.advance_to_game_time(10, 30);
        let (hour, minute) = test.what_time_is_it_now_in_world();
        assert_eq!((hour, minute), (10, 30));

        // 2. 翌日の朝 08:15（10:30より前の時間）まで進める
        test.advance_to_game_time(8, 15);
        let (hour, minute) = test.what_time_is_it_now_in_world();
        assert_eq!((hour, minute), (8, 15));
    }

    #[test]
    fn test_update_and_reset() {
        let mut test = GameTime::init(1.0, 8);

        // 瞬間的なupdateのテスト
        let delta1 = test.update();
        assert!(delta1 >= Duration::from_secs(0));

        // 少しだけsleepして時間の経過を模倣
        std::thread::sleep(Duration::from_millis(15));
        let delta2 = test.update();
        assert!(delta2 >= Duration::from_millis(10));

        // resetのテスト： sleepしてもresetを呼べばその間の時間は累積されない（破棄される）
        std::thread::sleep(Duration::from_millis(15));
        test.reset(); // 現在時刻に last_update_time を同期させ、今までの経過を破棄

        // reset直後のupdateでの経過時間はほぼ0になるはず
        let delta3 = test.update();
        assert!(delta3 < Duration::from_millis(5));
    }
}
