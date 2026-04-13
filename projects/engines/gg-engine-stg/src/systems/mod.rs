//! STG 引擎系统定义
//! 定义游戏中使用的各种 ECS 系统

pub mod ai;
pub mod audio;
pub mod behavior_tree;
pub mod boss_ai;
pub mod boundary;
pub mod bullet_emitter;
pub mod bullet_lifetime;
pub mod collision;
pub mod input;
pub mod invulnerability;
pub mod item;
pub mod movement;
pub mod render;
pub mod types;
pub mod weapon;

pub use ai::AISystem;
pub use audio::AudioSystem;
pub use behavior_tree::BehaviorTreeSystem;
pub use boss_ai::BossAISystem;
pub use boundary::BoundarySystem;
pub use bullet_emitter::{BulletEmitterSystem, spawn_enemy_bullet};
pub use bullet_lifetime::BulletLifetimeSystem;
pub use collision::CollisionSystem;
pub use input::InputSystem;
pub use invulnerability::InvulnerabilitySystem;
pub use item::ItemSystem;
pub use movement::MovementSystem;
pub use render::RenderSystem;
pub use types::BehaviorStatus;
pub use weapon::WeaponSystem;
