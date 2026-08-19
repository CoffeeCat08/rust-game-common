// common/src/utility/time.rs

use std::time::{Duration, Instant};

pub struct GameTime {
    pub start_time: Instant,
    pub last_update_time: Instant,
}

/// Gameを開始した際の初期化
impl GameTime {
    pub fn init() -> Self {
        let now = Instant::now();

        Self {
            start_time: now,
            last_update_time: now,
        }
    }

    pub fn update(&mut self) -> Duration {
        let now = Instant::now();

        let current_duration = now.duration_since(self.last_update_time);
        self.last_update_time = now;

        current_duration
    }
    pub fn reset(&mut self) {
        self.last_update_time = Instant::now();
    }
}

/// real_durationを元にtime_weightに応じて、game時間に換算する関数
/// ex: in_game_world : real_world = 24h : 24h 同じなら、time_weightは"1.0"
/// ex: in_game_world : real_world = 24h : 3h 同じなら、time_weightは"8.0"
/// `time_weight`は,各projectでconst定義ほうが良い。
///
/// # Argument:
///   `real_duration` - 現実世界で経過した時間
///   `time_weight` - game内で経過した時間にする負価値
/// # Returns:
///   game内で経過した時間(Duration)
pub fn convert_to_game_duration(real_duration: Duration, time_weight: f64) -> Duration {
    real_duration.mul_f64(time_weight)
}

/// Game内での24時間を返す関数
///
/// # Argument:
///   'start_time` - 前回の時間
/// # Returns:
///   Game内での24:00を返す。
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
