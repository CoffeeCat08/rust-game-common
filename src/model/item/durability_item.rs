// common/src/model/item/durability_item.rs

//! # 耐久度・鮮度管理機能付きアイテムモジュール
//!
//! このモジュールは、サバイバルゲームなどに適した「数量/重量」と「耐久度/鮮度」の
//! 両方の状態を保つアイテムスタックおよびコンテナ管理の仕組みを提供します。
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
//!   * `is_broken`: 耐久度ゼロの判定
//!   * `split`: スタックの分割
//! - `Container`: 構造体
//!   * `new`: コンテナの初期化
//!   * `item_total`: 指定アイテムの合計数量の取得
//!   * `item_delete`: 空スタックの自動削除
//!   * `item_use`: 鮮度の悪い順に一括消費
//!   * `item_drop_by_stack`: 指定スタックを名指しで削除・廃棄
//!   * `item_delete_including_broken`: 空または大破したスタックの自動削除
//!   * `purge_broken_items`: 大破したスタックのみを一括取り出し

use crate::model::status::BoundedStatus;

/// アイテム操作時に発生し得るエラー。
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItemError {
    /// 分割時に、現在のスタックの数量を超える要求がされた
    InsufficientAmountToSplit,
    /// コンテナ内の合計数量が、必要消費量に満たない
    InsufficientTotalAmount,
}

/// 数量と耐久度（鮮度）の両方を管理するアイテムのスタック（山の単位）。
#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    /// アイテムの一意なID
    pub item_id: u32,
    /// 数量、または肉などの重量（min: 0.0, max: 最大スタック数, weight: 0.0）
    pub amount: BoundedStatus,
    /// 耐久度、または新鮮さ（min: 0.0, max: 最大値, weight: 時間劣化率）
    pub durability: BoundedStatus,
}

impl ItemStack {
    /// 新しい `ItemStack`（耐久値あり）を作成します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::durability_item::ItemStack;
    /// # use rust_game_common::model::status::BoundedStatus;
    /// let meat = ItemStack::new(201, 4.1, 10.0, 100.0, 100.0, -0.05);
    /// assert_eq!(meat.amount.current, 4.1);
    /// assert_eq!(meat.durability.current, 100.0);
    /// ```
    ///
    /// # Arguments
    /// * `item_id` - `u32` アイテムのID
    /// * `amount` - `f32` 初期数量（または初期重量）
    /// * `max_stack` - `f32` このアイテムの最大スタック制限
    /// * `durability` - `f32` 初期耐久度
    /// * `max_durability` - `f32` 最大耐久値
    /// * `durability_decay_weight` - `f32` 時間経過による耐久度の変化量（通常、劣化はマイナス値）
    ///
    /// # Returns
    /// 初期化された `ItemStack` のインスタンス (Self)
    pub fn new(
        item_id: u32,
        amount: f32,
        max_stack: f32,
        durability: f32,
        max_durability: f32,
        durability_decay_weight: f32,
    ) -> Self {
        Self {
            item_id,
            amount: BoundedStatus::new(amount.min(max_stack), 0.0, max_stack, 0.0),
            durability: BoundedStatus::new(
                durability.min(max_durability),
                0.0,
                max_durability,
                durability_decay_weight,
            ),
        }
    }

    /// スタックに数量（または重量）を加算し、上限を超えて溢れた「余剰分」を返します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::durability_item::ItemStack;
    /// let mut item = ItemStack::new(101, 80.0, 100.0, 100.0, 100.0, 0.0);
    /// let overflow = item.add_quantity(35.0);
    /// assert_eq!(item.amount.current, 100.0);
    /// assert_eq!(overflow, 15.0);
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

    /// スタックから数量（または重量）を減算し、足りずに回収しきれなかった「不足分」を返します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::durability_item::ItemStack;
    /// let mut item = ItemStack::new(101, 30.0, 100.0, 100.0, 100.0, 0.0);
    /// let shortage = item.subtract_quantity(45.0);
    /// assert_eq!(item.amount.current, 0.0);
    /// assert_eq!(shortage, 15.0);
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
    /// # use rust_game_common::model::item::durability_item::ItemStack;
    /// let empty_item = ItemStack::new(101, 0.0, 100.0, 100.0, 100.0, 0.0);
    /// assert!(empty_item.is_empty());
    /// ```
    ///
    /// # Returns
    /// 数量が最小値以下であれば `true`、まだ残っていれば `false` (`bool`)
    pub fn is_empty(&self) -> bool {
        self.amount.current <= self.amount.min
    }

    /// アイテムの耐久度・品質・新鮮さが完全に底を突いた（大破・腐敗した）かを判定します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::durability_item::ItemStack;
    /// let broken_item = ItemStack::new(101, 10.0, 100.0, 0.0, 100.0, 0.0);
    /// assert!(broken_item.is_broken());
    /// ```
    ///
    /// # Returns
    /// 耐久度が最小値以下に破壊・腐敗していれば `true`、まだ機能が残っていれば `false` (`bool`)
    pub fn is_broken(&self) -> bool {
        self.durability.current <= self.durability.min
    }

    /// 指定した数量をスタックから切り離し、同じ耐久度・品質を引き継いだ新しい `ItemStack` として返します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::durability_item::{ItemStack, ItemError};
    /// let mut meat = ItemStack::new(201, 5.0, 10.0, 80.0, 100.0, -0.05);
    /// let new_stack = meat.split(2.0).unwrap();
    /// assert_eq!(meat.amount.current, 3.0);
    /// assert_eq!(new_stack.amount.current, 2.0);
    /// ```
    ///
    /// # Arguments
    /// * `qty` - `f32` 分割して引きぬきたい数量
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
            durability: self.durability,
        })
    }
}

/// 複数の `ItemStack`（耐久値あり）をまとめて収容するコンテナ（カバン、箱など）。
#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    /// コンテナを一意に識別するID
    pub container_id: u32,
    /// コンテナ内の全アイテムスタックのリスト
    pub items: Vec<ItemStack>,
}

impl Container {
    /// 新しい空の `Container` を作成します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::durability_item::Container;
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

    /// コンテナ内にある指定されたIDのアイテムの「合計数量（または合計重量）」を取得します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::durability_item::{Container, ItemStack};
    /// let mut bag = Container::new(1);
    /// bag.items.push(ItemStack::new(201, 3.5, 10.0, 100.0, 100.0, 0.0));
    /// bag.items.push(ItemStack::new(201, 4.2, 10.0, 80.0, 100.0, 0.0));
    /// assert_eq!(bag.item_total(201), 7.7);
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
    /// # use rust_game_common::model::item::durability_item::{Container, ItemStack};
    /// let mut bag = Container::new(1);
    /// bag.items.push(ItemStack::new(201, 0.0, 10.0, 100.0, 100.0, 0.0));
    /// bag.item_delete();
    /// assert!(bag.items.is_empty());
    /// ```
    pub fn item_delete(&mut self) {
        self.items.retain(|item| !item.is_empty());
    }

    /// 鮮度が低い（耐久値の現在値が低い）スタックから優先して、指定した数量分を消費します。
    ///
    /// 内部でアイテムスタックを自動的に「耐久度/鮮度の昇順（悪い順）」にソートしてから消費を実行します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::durability_item::{Container, ItemStack, ItemError};
    /// let mut bag = Container::new(1);
    /// bag.items.push(ItemStack::new(201, 3.0, 10.0, 100.0, 100.0, 0.0));
    /// bag.items.push(ItemStack::new(201, 2.0, 10.0, 50.0, 100.0, 0.0));
    ///
    /// bag.item_use(201, 3.0).unwrap();
    /// assert_eq!(bag.item_total(201), 2.0);
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
    /// コンテナ内の指定アイテムの合計数が足りない場合に `ItemError::InsufficientTotalAmount` を返します。
    pub fn item_use(&mut self, item_id: u32, mut req_amount: f32) -> Result<(), ItemError> {
        let total = self.item_total(item_id);
        if req_amount > total {
            return Err(ItemError::InsufficientTotalAmount);
        }

        // 鮮度が悪い順に並び替え
        self.items.sort_by(|a, b| {
            a.durability
                .current
                .partial_cmp(&b.durability.current)
                .unwrap()
        });

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
    /// # use rust_game_common::model::item::durability_item::{Container, ItemStack};
    /// let mut bag = Container::new(1);
    /// let item_a = ItemStack::new(201, 1.0, 10.0, 90.0, 100.0, 0.0);
    /// bag.items.push(item_a.clone());
    /// let dropped = bag.item_drop_by_stack(&item_a).unwrap();
    /// assert_eq!(dropped, item_a);
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

    /// 数量が0になったスタックに加え、「完全に大破・品質が0になったアイテム」も自動的にコンテナから除去して一掃します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::durability_item::{Container, ItemStack};
    /// let mut bag = Container::new(1);
    /// bag.items.push(ItemStack::new(201, 5.0, 10.0, 0.0, 100.0, 0.0)); // 耐久0
    /// bag.item_delete_including_broken();
    /// assert!(bag.items.is_empty());
    /// ```
    pub fn item_delete_including_broken(&mut self) {
        self.items
            .retain(|item| !item.is_empty() && !item.is_broken());
    }

    /// コンテナ内から、大破した（品質・鮮度が0の）スタックだけをすべて取り出して、リストとして返します。
    ///
    /// # Examples
    /// ```rust
    /// # use rust_game_common::model::item::durability_item::{Container, ItemStack};
    /// let mut bag = Container::new(1);
    /// let rotten_meat = ItemStack::new(201, 1.5, 10.0, 0.0, 100.0, 0.0);
    /// bag.items.push(rotten_meat.clone());
    /// let trashes = bag.purge_broken_items();
    /// assert_eq!(trashes, vec![rotten_meat]);
    /// ```
    ///
    /// # Returns
    /// コンテナ内から分離・抽出された、大破したアイテムスタックのリスト (`Vec<ItemStack>`)
    pub fn purge_broken_items(&mut self) -> Vec<ItemStack> {
        // self.items を一時的に空にして所有権を取り出す (&mut を忘れない)
        let (broken, kept): (Vec<ItemStack>, Vec<ItemStack>) = std::mem::take(&mut self.items)
            .into_iter()
            .partition(|item| item.is_broken());

        self.items = kept;
        self.item_delete();
        broken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_stack_new_and_bounds() {
        let item = ItemStack::new(101, 150.0, 100.0, 120.0, 100.0, -0.1);
        assert_eq!(item.item_id, 101);
        assert_eq!(item.amount.current, 100.0);
        assert_eq!(item.durability.current, 100.0);
    }

    #[test]
    fn test_add_quantity() {
        let mut item = ItemStack::new(101, 80.0, 100.0, 100.0, 100.0, 0.0);

        let overflow = item.add_quantity(15.0);
        assert_eq!(item.amount.current, 95.0);
        assert_eq!(overflow, 0.0);

        let overflow2 = item.add_quantity(15.0);
        assert_eq!(item.amount.current, 100.0);
        assert_eq!(overflow2, 10.0);
    }

    #[test]
    fn test_subtract_quantity() {
        let mut item = ItemStack::new(101, 30.0, 100.0, 100.0, 100.0, 0.0);

        let shortage = item.subtract_quantity(20.0);
        assert_eq!(item.amount.current, 10.0);
        assert_eq!(shortage, 0.0);

        let shortage2 = item.subtract_quantity(15.0);
        assert_eq!(item.amount.current, 0.0);
        assert_eq!(shortage2, 5.0);
    }

    #[test]
    fn test_is_empty_and_is_broken() {
        let empty_item = ItemStack::new(101, 0.0, 100.0, 100.0, 100.0, 0.0);
        assert!(empty_item.is_empty());
        assert!(!empty_item.is_broken());

        let broken_item = ItemStack::new(101, 10.0, 100.0, 0.0, 100.0, 0.0);
        assert!(!broken_item.is_empty());
        assert!(broken_item.is_broken());
    }

    #[test]
    fn test_split_success_and_failure() {
        let mut item = ItemStack::new(101, 10.0, 99.0, 85.0, 100.0, -0.5);

        let new_stack = item.split(4.0).unwrap();
        assert_eq!(item.amount.current, 6.0);
        assert_eq!(new_stack.amount.current, 4.0);
        assert_eq!(new_stack.item_id, item.item_id);
        assert_eq!(new_stack.durability.current, 85.0);
        assert_eq!(new_stack.durability.weight, -0.5);

        let err = item.split(7.0);
        assert_eq!(err, Err(ItemError::InsufficientAmountToSplit));
        assert_eq!(item.amount.current, 6.0);
    }

    #[test]
    fn test_container_item_total() {
        let mut bag = Container::new(1);
        bag.items
            .push(ItemStack::new(201, 3.5, 10.0, 100.0, 100.0, 0.0));
        bag.items
            .push(ItemStack::new(201, 4.2, 10.0, 80.0, 100.0, 0.0));
        bag.items
            .push(ItemStack::new(301, 5.0, 10.0, 100.0, 100.0, 0.0));

        assert_eq!(bag.item_total(201), 7.7);
        assert_eq!(bag.item_total(301), 5.0);
        assert_eq!(bag.item_total(999), 0.0);
    }

    #[test]
    fn test_container_item_delete() {
        let mut bag = Container::new(1);
        bag.items
            .push(ItemStack::new(201, 5.0, 10.0, 100.0, 100.0, 0.0));
        bag.items
            .push(ItemStack::new(201, 0.0, 10.0, 100.0, 100.0, 0.0));

        bag.item_delete();
        assert_eq!(bag.items.len(), 1);
        //  を確実に指定
        assert_eq!(bag.items[0].amount.current, 5.0);
    }

    #[test]
    fn test_container_item_use_sorting_and_success() {
        let mut bag = Container::new(1);

        let high_quality = ItemStack::new(201, 4.0, 10.0, 100.0, 100.0, 0.0);
        let low_quality = ItemStack::new(201, 3.0, 10.0, 30.0, 100.0, 0.0);
        let mid_quality = ItemStack::new(201, 2.0, 10.0, 75.0, 100.0, 0.0);

        bag.items.push(high_quality);
        bag.items.push(low_quality);
        bag.items.push(mid_quality);

        let result = bag.item_use(201, 6.0);
        assert_eq!(result, Ok(()));

        assert_eq!(bag.item_total(201), 3.0);
        assert_eq!(bag.items.len(), 1);
        //  を確実に指定
        assert_eq!(bag.items[0].durability.current, 100.0);
        assert_eq!(bag.items[0].amount.current, 3.0);
    }

    #[test]
    fn test_container_item_use_shortage_error() {
        let mut bag = Container::new(1);
        bag.items
            .push(ItemStack::new(201, 5.0, 10.0, 100.0, 100.0, 0.0));

        let result = bag.item_use(201, 6.0);
        assert_eq!(result, Err(ItemError::InsufficientTotalAmount));
        assert_eq!(bag.item_total(201), 5.0);
    }

    #[test]
    fn test_container_item_drop_by_stack() {
        let mut bag = Container::new(1);
        let item_a = ItemStack::new(201, 1.0, 10.0, 90.0, 100.0, 0.0);
        let item_b = ItemStack::new(201, 1.0, 10.0, 40.0, 100.0, 0.0);

        bag.items.push(item_a.clone());
        bag.items.push(item_b.clone());

        let dropped = bag.item_drop_by_stack(&item_b).unwrap();
        assert_eq!(dropped, item_b);
        assert_eq!(bag.items.len(), 1);
        assert_eq!(bag.items, vec![item_a]);

        let non_existent = ItemStack::new(999, 1.0, 10.0, 100.0, 100.0, 0.0);
        assert_eq!(bag.item_drop_by_stack(&non_existent), None);
    }

    #[test]
    fn test_item_delete_including_broken() {
        let mut bag = Container::new(1);
        bag.items
            .push(ItemStack::new(201, 5.0, 10.0, 100.0, 100.0, 0.0));
        bag.items
            .push(ItemStack::new(201, 3.0, 10.0, 0.0, 100.0, 0.0));
        bag.items
            .push(ItemStack::new(201, 0.0, 10.0, 100.0, 100.0, 0.0));

        bag.item_delete_including_broken();
        assert_eq!(bag.items.len(), 1);
        //  を確実に指定
        assert_eq!(bag.items[0].amount.current, 5.0);
    }

    #[test]
    fn test_purge_broken_items() {
        let mut bag = Container::new(1);
        let good_meat = ItemStack::new(201, 4.0, 10.0, 100.0, 100.0, 0.0);
        let rotten_meat = ItemStack::new(201, 1.5, 10.0, 0.0, 100.0, 0.0);

        bag.items.push(good_meat.clone());
        bag.items.push(rotten_meat.clone());

        let trashes = bag.purge_broken_items();
        assert_eq!(trashes.len(), 1);
        assert_eq!(trashes, vec![rotten_meat]);

        assert_eq!(bag.items.len(), 1);
        assert_eq!(bag.items, vec![good_meat]);
    }
}
