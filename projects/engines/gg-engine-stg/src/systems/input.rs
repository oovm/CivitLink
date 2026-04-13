//! 输入系统
//! 处理玩家输入，从 World 的 InputState 资源读取按键状态

use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};

/// 输入系统
/// 处理玩家输入，从 World 的 InputState 资源读取按键状态
pub struct InputSystem {
    /// 移动速度
    move_speed: f32,
}

impl InputSystem {
    /// 创建新的输入系统
    pub fn new(move_speed: f32) -> Self {
        Self { move_speed }
    }
}

impl System for InputSystem {
    fn name(&self) -> &str {
        "input_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let (up, down, left, right, shoot, special) = world
            .get_resource::<InputState>()
            .map(|s| (s.up_pressed, s.down_pressed, s.left_pressed, s.right_pressed, s.shoot_pressed, s.special_pressed))
            .unwrap_or((false, false, false, false, false, false));

        let move_speed = self.move_speed;
        let entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
        for entity in entities {
            if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                velocity.dx = 0.0;
                velocity.dy = 0.0;
                if left {
                    velocity.dx -= move_speed;
                }
                if right {
                    velocity.dx += move_speed;
                }
                if up {
                    velocity.dy -= move_speed;
                }
                if down {
                    velocity.dy += move_speed;
                }
            }
        }

        let _ = (shoot, special);
        Ok(())
    }
}
