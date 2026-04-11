//! STG 引擎道具系统定义
//! 定义道具类型、道具组件和道具生成器资源

use gg_ecs::{Component, Resource};

/// 道具类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    /// 增强火力
    PowerUp,
    /// 额外分数
    ScoreBonus,
    /// 增加炸弹
    Bomb,
    /// 增加生命
    Life,
    /// 临时护盾
    Shield,
}

/// 道具组件
#[derive(Debug, Component)]
pub struct Item {
    /// 道具类型
    pub item_type: ItemType,
    /// 道具效果数值
    pub value: u32,
    /// 下落速度
    pub fall_speed: f32,
}

impl Default for Item {
    fn default() -> Self {
        Self { item_type: ItemType::PowerUp, value: 1, fall_speed: 1.0 }
    }
}

/// 道具生成器资源
#[derive(Debug, Clone, Resource)]
pub struct ItemSpawner {
    /// 道具掉落概率（0.0 ~ 1.0）
    pub drop_rate: f32,
    /// 各道具类型的权重
    pub weights: Vec<(ItemType, f32)>,
}

impl Default for ItemSpawner {
    fn default() -> Self {
        Self {
            drop_rate: 0.3,
            weights: vec![
                (ItemType::PowerUp, 40.0),
                (ItemType::ScoreBonus, 30.0),
                (ItemType::Bomb, 15.0),
                (ItemType::Life, 5.0),
                (ItemType::Shield, 10.0),
            ],
        }
    }
}

impl ItemSpawner {
    /// 根据权重随机选择道具类型
    pub fn random_item_type(&self) -> ItemType {
        let total: f32 = self.weights.iter().map(|(_, w)| w).sum();
        let mut roll = total * (rand_simple() as f32);
        for (item_type, weight) in &self.weights {
            roll -= weight;
            if roll <= 0.0 {
                return *item_type;
            }
        }
        ItemType::PowerUp
    }
}

/// 简单伪随机数生成（0.0 ~ 1.0）
fn rand_simple() -> f64 {
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u64> = Cell::new(12345);
    }
    SEED.with(|s| {
        let mut seed = s.get();
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        s.set(seed);
        (seed >> 33) as f64 / (1u64 << 31) as f64
    })
}
