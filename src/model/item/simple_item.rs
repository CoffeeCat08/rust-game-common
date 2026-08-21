// common/src/model/item/simple_item.rs

//! # シンプルアイテム管理モジュール
//!
//! このモジュールは、耐久度や鮮度による劣化のない、通常のRPGやアクションゲームに適した
//! 汎用的なアイテムスタックおよびコンテナ管理の仕組みを提供します。
//!
//! ## 主な機能
//! - `ItemError`: 列挙型（エラーハンドリング用）
//!   * `InsufficientAmountToSplit`: 分割時の数量不足
//!   * `InsufficientTotalAmount`: コンテナ内の総量不足
//! - `ItemStack`: 構造体
//!   * `new`: 初期化
//!   * `add_quantity`: 数量の追加
//!   * `subtract_quantity`: 数量の減算
//!   * `is_empty`: 数量ゼロの判定
//!   * `split`: スタックの分割
//! - `Container`: 構造体
//!   * `new`: コンテナの初期化
//!   * `item_total`: 指定アイテムの合計数量の取得
//!   * `item_delete`: 空スタックの自動削除
//!   * `item_use`: 古い（リストの先頭の）スタックから順に一括消費
//!   * `item_drop_by_stack`: 指定スタックを名指しで削除・廃棄

use crate::model::status::BoundedStatus;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItemError {
    InsufficientAmountToSplit, // 分割時に、現在のスタックの数量を超える要求がされた
    InsufficientTotalAmount,   // コンテナ内の合計数量が、必要消費量に満たない
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub item_id: u32,          // アイテムの一意なID
    pub amount: BoundedStatus, // 数量（min: 0.0, max: 最大スタック数, weight: 0.0）
}

impl ItemStack {
    /// 新しい `ItemStack` を作成します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::simple_item::ItemStack;
    /// # use rust_game_common::model::status::BoundedStatus;
    /// let herb = ItemStack::new(101, 10.0, 99.0);
    /// assert_eq!(herb.amount.current, 10.0);
    /// ```
    ///
    /// # Arguments
    /// * `item_id` - `u32` アイテムのID
    /// * `amount` - `f32` 初期数量
    /// * `max_stack` - `f32` このアイテムの最大スタック制限
    ///
    /// # Returns
    /// 初期化された `ItemStack` のインスタンス (Self)
    pub fn new(item_id: u32, amount: f32, max_stack: f32) -> Self {
        Self {
            item_id,
            amount: BoundedStatus::new(amount.min(max_stack), 0.0, max_stack, 0.0),
        }
    }

    /// スタックに数量を加算し、上限を超えて溢れた「余剰分」を返します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::simple_item::ItemStack;
    /// let mut herb = ItemStack::new(101, 90.0, 99.0);
    /// let overflow = herb.add_quantity(15.0);
    /// assert_eq!(herb.amount.current, 99.0);
    /// assert_eq!(overflow, 6.0);
    /// ```
    ///
    /// # Arguments
    /// * `qty` - `f32` 加算したい数量
    ///
    /// # Returns
    /// 追加できずに溢れた余剰分の数量 (`f32`)
    pub fn add_quantity(&mut self, qty: f32) -> f32 {
        let total = self.amount.current + qty;
        if total > self.amount.max {
            let overflow = total - self.amount.max;
            self.amount.current = self.amount.max;
            overflow
        } else {
            self.amount.current = total;
            0.0
        }
    }

    /// スタックから数量を減算し、足りずに回収しきれなかった「不足分」を返します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::simple_item::ItemStack;
    /// let mut herb = ItemStack::new(101, 10.0, 99.0);
    /// let shortage = herb.subtract_quantity(15.0);
    /// assert_eq!(herb.amount.current, 0.0);
    /// assert_eq!(shortage, 5.0);
    /// ```
    ///
    /// # Arguments
    /// * `qty` - `f32` 減算したい数量
    ///
    /// # Returns
    /// 数量が足りずに引くことができなかった不足分の数量 (`f32`)
    pub fn subtract_quantity(&mut self, qty: f32) -> f32 {
        let total = self.amount.current - qty;
        if total < self.amount.min {
            let shortage = self.amount.min - total;
            self.amount.current = self.amount.min;
            shortage
        } else {
            self.amount.current = total;
            0.0
        }
    }

    /// スタックが空（数量が最小値以下）になったかどうかを判定します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::simple_item::ItemStack;
    /// let herb = ItemStack::new(101, 0.0, 99.0);
    /// assert!(herb.is_empty());
    /// ```
    ///
    /// # Returns
    /// 数量が最小値以下であれば `true`、まだ残っていれば `false` (`bool`)
    pub fn is_empty(&self) -> bool {
        self.amount.current <= self.amount.min
    }

    /// 指定した数量をスタックから切り離し、同じアイテム設定を引き継いだ新しい `ItemStack` として返します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::simple_item::{ItemStack, ItemError};
    /// let mut herb = ItemStack::new(101, 10.0, 99.0);
    /// let new_stack = herb.split(3.0).unwrap();
    /// assert_eq!(herb.amount.current, 7.0);
    /// assert_eq!(new_stack.amount.current, 3.0);
    /// ```
    ///
    /// # Arguments
    /// * `qty` - `f32` 分割して引き抜きたい数量
    ///
    /// # Returns
    /// 分割されて新しく生成された `ItemStack` の結果 (Result<ItemStack, ItemError>)
    ///
    /// # Errors
    /// 要求された分割数が、現在のスタックの数量を超えている場合に `ItemError::InsufficientAmountToSplit` を返します。
    pub fn split(&mut self, qty: f32) -> Result<Self, ItemError> {
        if qty > self.amount.current {
            return Err(ItemError::InsufficientAmountToSplit);
        }
        self.amount.current -= qty;

        Ok(Self {
            item_id: self.item_id,
            amount: BoundedStatus::new(qty, self.amount.min, self.amount.max, self.amount.weight),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    pub container_id: u32,     // コンテナを一意に識別するID
    pub items: Vec<ItemStack>, // コンテナ内の全アイテムスタックのリスト
}

impl Container {
    /// 新しい空の `Container` を作成します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::simple_item::Container;
    /// let bag = Container::new(1);
    /// assert!(bag.items.is_empty());
    /// ```
    ///
    /// # Arguments
    /// * `container_id` - `u32` コンテナのID
    ///
    /// # Returns
    /// 初期化された空の `Container` インスタンス (Self)
    pub fn new(container_id: u32) -> Self {
        Self {
            container_id,
            items: Vec::new(),
        }
    }

    /// コンテナ内にある指定されたIDのアイテムの「合計数量」を取得します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::simple_item::{Container, ItemStack};
    /// let mut bag = Container::new(1);
    /// bag.items.push(ItemStack::new(101, 50.0, 99.0));
    /// bag.items.push(ItemStack::new(101, 20.0, 99.0));
    /// assert_eq!(bag.item_total(101), 70.0);
    /// ```
    ///
    /// # Arguments
    /// * `item_id` - `u32` 取得したいアイテムのID
    ///
    /// # Returns
    /// コンテナ内に存在する指定アイテムの合計数量 (`f32`)
    pub fn item_total(&self, item_id: u32) -> f32 {
        self.items
            .iter()
            .filter(|item| item.item_id == item_id)
            .map(|item| item.amount.current)
            .sum()
    }

    /// 数量が0になった、中身の空なスタックをコンテナから自動的に除去します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::simple_item::{Container, ItemStack};
    /// let mut bag = Container::new(1);
    /// bag.items.push(ItemStack::new(101, 0.0, 99.0));
    /// bag.item_delete();
    /// assert!(bag.items.is_empty());
    /// ```
    pub fn item_delete(&mut self) {
        self.items.retain(|item| !item.is_empty());
    }

    /// 格納されている古い順（リストの先頭）のスタックから優先して、指定した数量分を消費します。
    ///
    /// 数量を消費しきって空になったスタックはコンテナ内から自動的に除去されます。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::simple_item::{Container, ItemStack, ItemError};
    /// let mut bag = Container::new(1);
    /// bag.items.push(ItemStack::new(101, 10.0, 99.0)); // 古いスタック
    /// bag.items.push(ItemStack::new(101, 15.0, 99.0)); // 新しいスタック
    ///
    /// bag.item_use(101, 12.0).unwrap();
    /// assert_eq!(bag.item_total(101), 13.0);
    /// assert_eq!(bag.items[0].amount.current, 13.0); // 先頭の10.0が消去され、次のスタックから2.0消費された
    /// ```
    ///
    /// # Arguments
    /// * `item_id` - `u32` 消費したいアイテムのID
    /// * `req_amount` - `f32` 必要とする消費数量
    ///
    /// # Returns
    /// 消費処理が成功すれば `Ok(())` (Result<(), ItemError>)
    ///
    /// # Errors
    /// コンテナ内の指定アイテムの合計数が、必要とする数量に満たない場合に `ItemError::InsufficientTotalAmount` を返します。
    pub fn item_use(&mut self, item_id: u32, mut req_amount: f32) -> Result<(), ItemError> {
        let total = self.item_total(item_id);
        if req_amount > total {
            return Err(ItemError::InsufficientTotalAmount);
        }

        for item in self.items.iter_mut() {
            if item.item_id != item_id {
                continue;
            }
            if req_amount <= 0.0 {
                break;
            }

            let available = item.amount.current;
            if available >= req_amount {
                item.subtract_quantity(req_amount);
                req_amount = 0.0;
            } else {
                item.subtract_quantity(available);
                req_amount -= available;
            }
        }

        self.item_delete();
        Ok(())
    }

    /// 指定されたスタックと「完全に一致する」アイテムをコンテナ内から特定し、ピンポイントで取り出して廃棄（取得）します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::simple_item::{Container, ItemStack};
    /// let mut bag = Container::new(1);
    /// let item_a = ItemStack::new(101, 5.0, 99.0);
    /// bag.items.push(item_a.clone());
    ///
    /// let dropped = bag.item_drop_by_stack(&item_a).unwrap();
    /// assert_eq!(dropped, item_a);
    /// assert!(bag.items.is_empty());
    /// ```
    ///
    /// # Arguments
    /// * `target` - `&ItemStack` 取り出したい対象アイテムスタックへの参照
    ///
    /// # Returns
    /// 取り出したスタックが存在した場合は `Some(ItemStack)` を、存在しない場合は `None` を返します (Option<ItemStack>)
    pub fn item_drop_by_stack(&mut self, target: &ItemStack) -> Option<ItemStack> {
        if let Some(pos) = self.items.iter().position(|item| item == target) {
            Some(self.items.remove(pos))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_stack_new_and_bounds() {
        let item = ItemStack::new(101, 150.0, 100.0);
        assert_eq!(item.item_id, 101);
        assert_eq!(item.amount.current, 100.0); // 上限クランプのチェック
    }

    #[test]
    fn test_add_quantity() {
        let mut item = ItemStack::new(101, 80.0, 100.0);

        let overflow = item.add_quantity(15.0);
        assert_eq!(item.amount.current, 95.0);
        assert_eq!(overflow, 0.0);

        let overflow2 = item.add_quantity(15.0);
        assert_eq!(item.amount.current, 100.0);
        assert_eq!(overflow2, 10.0);
    }

    #[test]
    fn test_subtract_quantity() {
        let mut item = ItemStack::new(101, 30.0, 100.0);

        let shortage = item.subtract_quantity(20.0);
        assert_eq!(item.amount.current, 10.0);
        assert_eq!(shortage, 0.0);

        let shortage2 = item.subtract_quantity(15.0);
        assert_eq!(item.amount.current, 0.0);
        assert_eq!(shortage2, 5.0);
    }

    #[test]
    fn test_is_empty() {
        let empty_item = ItemStack::new(101, 0.0, 100.0);
        assert!(empty_item.is_empty());

        let normal_item = ItemStack::new(101, 10.0, 100.0);
        assert!(!normal_item.is_empty());
    }

    #[test]
    fn test_split_success_and_failure() {
        let mut item = ItemStack::new(101, 10.0, 99.0);

        let new_stack = item.split(4.0).unwrap();
        assert_eq!(item.amount.current, 6.0);
        assert_eq!(new_stack.amount.current, 4.0);
        assert_eq!(new_stack.item_id, item.item_id);

        let err = item.split(7.0);
        assert_eq!(err, Err(ItemError::InsufficientAmountToSplit));
        assert_eq!(item.amount.current, 6.0);
    }

    #[test]
    fn test_container_item_total() {
        let mut bag = Container::new(1);
        bag.items.push(ItemStack::new(101, 3.5, 10.0));
        bag.items.push(ItemStack::new(101, 4.2, 10.0));
        bag.items.push(ItemStack::new(301, 5.0, 10.0));

        assert_eq!(bag.item_total(101), 7.7);
        assert_eq!(bag.item_total(301), 5.0);
        assert_eq!(bag.item_total(999), 0.0);
    }

    #[test]
    fn test_container_item_delete() {
        let mut bag = Container::new(1);
        bag.items.push(ItemStack::new(101, 5.0, 10.0));
        bag.items.push(ItemStack::new(101, 0.0, 10.0));

        bag.item_delete();
        assert_eq!(bag.items.len(), 1);
        assert_eq!(bag.items[0].amount.current, 5.0); // itemsを使用
    }

    #[test]
    fn test_container_item_use_success() {
        let mut bag = Container::new(1);
        bag.items.push(ItemStack::new(101, 3.0, 10.0)); // 古いスタック
        bag.items.push(ItemStack::new(101, 5.0, 10.0)); // 新しいスタック

        // 4.0消費する（古い方の3.0が消滅し、新しい方の5.0から1.0引かれて4.0残る）
        let result = bag.item_use(101, 4.0);
        assert_eq!(result, Ok(()));

        assert_eq!(bag.item_total(101), 4.0);
        assert_eq!(bag.items.len(), 1);
        assert_eq!(bag.items[0].amount.current, 4.0); // itemsを使用
    }

    #[test]
    fn test_container_item_use_shortage_error() {
        let mut bag = Container::new(1);
        bag.items.push(ItemStack::new(101, 5.0, 10.0));

        let result = bag.item_use(101, 6.0);
        assert_eq!(result, Err(ItemError::InsufficientTotalAmount));
        assert_eq!(bag.item_total(101), 5.0);
    }

    #[test]
    fn test_container_item_drop_by_stack() {
        let mut bag = Container::new(1);
        let item_a = ItemStack::new(101, 1.0, 10.0);
        let item_b = ItemStack::new(101, 2.0, 10.0);

        bag.items.push(item_a.clone());
        bag.items.push(item_b.clone());

        let dropped = bag.item_drop_by_stack(&item_b).unwrap();
        assert_eq!(dropped, item_b);
        assert_eq!(bag.items.len(), 1);
        assert_eq!(bag.items, vec![item_a]);

        let non_existent = ItemStack::new(999, 1.0, 10.0);
        assert_eq!(bag.item_drop_by_stack(&non_existent), None);
    }
}
