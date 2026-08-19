// common/src/model/grid.rs

use std::ops::RangeInclusive;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    x: f32, // 横軸
    y: f32, // 高さ軸
    z: f32, // 縦軸
    direction: Direction,
}

// playerだけじゃなくて、cameraの座標にも使えると思う。

impl Position {
    pub fn new(x: f32, y: f32, z: f32, direction: Direction) -> Self {
        Self { x, y, z, direction }
    }

    pub fn translate(&mut self, dir: Direction, distance: f32) {
        match dir {
            Direction::North => self.z -= distance,
            Direction::South => self.z += distance,
            Direction::East => self.x += distance,
            Direction::West => self.x -= distance,
        }
    }

    pub fn move_forward(&mut self, distance: f32) {
        let current_dir = self.direction;
        self.translate(current_dir, distance);
    }

    /// RangeInclusiveは,-100.0..=100.0 のような書き方。
    pub fn is_inside(
        self,
        x_range: RangeInclusive<f32>,
        y_range: RangeInclusive<f32>,
        z_range: RangeInclusive<f32>,
    ) -> bool {
        x_range.contains(&self.x) && y_range.contains(&self.y) && z_range.contains(&self.z)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::grid::Direction::{East, North, South, West};

    use super::*;

    #[test]
    fn test_translate() {
        let mut test = Position::new(0.0, 0.0, 0.0, North);

        test.translate(North, 1.0);
        assert_eq!((test.x, test.y, test.z), (0.0, 0.0, -1.0));
        test.translate(South, 1.0);
        assert_eq!((test.x, test.y, test.z), (0.0, 0.0, 0.0));
        test.translate(East, 1.0);
        assert_eq!((test.x, test.y, test.z), (1.0, 0.0, 0.0));
        test.translate(West, 1.0);
        assert_eq!((test.x, test.y, test.z), (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_is_inside() {
        let mut test = Position::new(2.0, 2.0, 2.0, North);

        let range_x = 0.0..=5.0;
        let range_y = 0.0..=5.0;
        let range_z = 0.0..=5.0;

        assert!(test.is_inside(range_x.clone(), range_y.clone(), range_z.clone()));
        test.z = 7.0;
        assert!(!test.is_inside(range_x.clone(), range_y.clone(), range_z.clone()));
    }
}
