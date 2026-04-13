//! 移动系统
//! 处理实体移动和边界约束

use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};

/// 移动系统
/// 处理实体移动和边界约束
pub struct MovementSystem;

impl System for MovementSystem {
    fn name(&self) -> &str {
        "movement_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let (screen_w, screen_h) = world.get_resource::<ScreenBounds>().map(|b| (b.width, b.height)).unwrap_or((800.0, 600.0));

        let entities: Vec<Entity> = world.query::<Velocity>().map(|(e, _)| e).collect();
        for entity in entities {
            let (dx, dy) = world.get_component::<Velocity>(entity).map(|v| (v.dx, v.dy)).unwrap_or((0.0, 0.0));
            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                transform.x += dx;
                transform.y += dy;
                if transform.x < 0.0 {
                    transform.x = 0.0;
                }
                if transform.x > screen_w {
                    transform.x = screen_w;
                }
                if transform.y < 0.0 {
                    transform.y = 0.0;
                }
                if transform.y > screen_h {
                    transform.y = screen_h;
                }
            }
        }
        Ok(())
    }
}
