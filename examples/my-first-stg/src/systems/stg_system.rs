use gg_ecs::prelude::*;
use crate::components::*;
use crate::resources::GameState;

pub struct PlayerSystem;

impl System for PlayerSystem {
    type Data<'a> = (
        Query<'a, (&mut Player, &Collider)>,
        Res<'a, GameState>,
    );

    fn run(&mut self, (mut players, game_state): Self::Data<'_>) {
        if game_state.game_over {
            return;
        }

        for (mut player, _) in &mut players {
            // 玩家移动逻辑
            // 玩家射击逻辑
        }
    }
}

pub struct EnemySystem;

impl System for EnemySystem {
    type Data<'a> = (
        Query<'a, (&mut Enemy, &Collider)>,
        Res<'a, GameState>,
    );

    fn run(&mut self, (mut enemies, game_state): Self::Data<'_>) {
        if game_state.game_over {
            return;
        }

        for (mut enemy, _) in &mut enemies {
            // 敌人 AI 逻辑
            // 敌人射击逻辑
        }
    }
}

pub struct ProjectileSystem;

impl System for ProjectileSystem {
    type Data<'a> = Query<'a, &mut Projectile>;

    fn run(&mut self, mut projectiles: Self::Data<'_>) {
        for mut projectile in &mut projectiles {
            // 子弹移动逻辑
            // 子弹生命周期管理
        }
    }
}

pub struct CollisionSystem;

impl System for CollisionSystem {
    type Data<'a> = (
        Query<'a, (&Player, &Collider)>,
        Query<'a, (&Enemy, &Collider)>,
        Query<'a, (&Projectile, &Collider)>,
        ResMut<'a, GameState>,
    );

    fn run(&mut self, (players, enemies, projectiles, mut game_state): Self::Data<'_>) {
        if game_state.game_over {
            return;
        }

        // 碰撞检测逻辑
        // 处理玩家与敌人的碰撞
        // 处理子弹与敌人的碰撞
        // 处理子弹与玩家的碰撞
    }
}

pub struct SpawnSystem;

impl System for SpawnSystem {
    type Data<'a> = (
        Res<'a, GameState>,
        Commands<'a>,
    );

    fn run(&mut self, (game_state, commands): Self::Data<'_>) {
        if game_state.game_over {
            return;
        }

        // 敌人生成逻辑
    }
}
