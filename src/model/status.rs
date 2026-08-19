// common/src/model/status.rs

use std::time::Duration;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StatusError {
    MinLimitReached,
    MaxLimitReached,
}

#[derive(Clone, Copy, Debug)]
pub struct BoundedStatus {
    pub current: f32,
    pub min: f32,
    pub max: f32,
    pub weight: f32, // game内1秒の変化量
}

impl BoundedStatus {
    pub fn new(current: f32, min: f32, max: f32, weight: f32) -> Self {
        Self {
            current,
            min,
            max,
            weight,
        }
    }

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

    pub fn apply_action_subtract(&mut self, amount: f32) -> Result<(), StatusError> {
        self.current -= amount;

        if self.current <= self.min {
            self.current = self.min;
            Err(StatusError::MinLimitReached)
        } else {
            Ok(())
        }
    }

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
