// common/src/model/grid.rs

//! # 座標管理・範囲判定モジュール
//!
//! このモジュールは、position情報をもとに、現在地がから行動可能等の内容を提供します。
//!
//! ## 主な機能
//! - `Direction`: 列挙型
//!   * `North`: 北
//!   * `South`: 南
//!   * `East`: 東
//!   * `West`: 西
//! - `Position`: 構造体
//!   * `new`: positionを持つobjectの生成。player,enemy等
//!   * `translate`: 東西南北を基準とした移動。絶対方向
//!   * `move_forward`: 確認方向に応じて進む。ラジコンコントローラ
//!   * `is_inside`: 範囲内に収まっているかの判定

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
    pub x: f32, // 横軸
    pub y: f32, // 高さ軸
    pub z: f32, // 縦軸
    pub direction: Direction,
}

// playerだけじゃなくて、cameraの座標にも使えると思う。

impl Position {
    /// 概要を書く
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::grid::{Position,Direction};
    ///
    /// let player = Position::new(0.0,0.0,0.0,Direction::North);
    /// ```
    ///
    /// # Arguments
    /// * `x` - `f32` x軸
    /// * `y` - `f32` y軸
    /// * `z` - `f32` z軸
    /// * `direction` - `Direction` 向いている方向
    pub fn new(x: f32, y: f32, z: f32, direction: Direction) -> Self {
        Self { x, y, z, direction }
    }

    /// 十字キーを東西南北に見立て、入力した方角に移動する
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::grid::{Position,Direction};
    ///
    /// let mut player = Position::new(0.0,0.0,0.0,Direction::North);
    /// player.translate(Direction::North,1.0);
    /// ```
    ///
    /// # Arguments
    /// * `dir` - `Direction` 移動する方角
    /// * `distance` - `f32` 入力毎の移動距離
    pub fn translate(&mut self, dir: Direction, distance: f32) {
        match dir {
            Direction::North => self.z -= distance,
            Direction::South => self.z += distance,
            Direction::East => self.x += distance,
            Direction::West => self.x -= distance,
        }
    }

    /// 現在の向いている方向に移動する
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::grid::{Direction,Position};
    ///
    /// let mut player = Position::new(0.0,0.0,0.0,Direction::North);
    /// player.move_forward(1.0);
    /// ```
    ///
    /// # Arguments
    /// * `distance` - `f32` 入力毎の移動距離
    pub fn move_forward(&mut self, distance: f32) {
        let current_dir = self.direction;
        self.translate(current_dir, distance);
    }

    /// 範囲内にいるかどうかを判定して返す
    /// RangeInclusiveは`0.0..=100`のように書く。これで、0.0~100.0という意味
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::grid::{Direction,Position};
    ///
    /// let player = Position::new(0.0,0.0,0.0,Direction::North);
    /// let inside_bool = player.is_inside(0.0..=10.0,0.0..=10.0,0.0..=10.0);
    /// ```
    ///
    /// # Arguments
    /// * `x_range` - `RangeInclusive<f32>` x軸の範囲
    /// * `y_range` - `RangeInclusive<f32>` y軸の範囲
    /// * `z_range` - `RangeInclusive<f32>` z軸の範囲
    ///
    /// # Returns
    /// 範囲内に収まっている場合はTrueを、違うならFalseを返している。
    pub fn is_inside(
        &self,
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
