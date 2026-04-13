//! 渲染系统
//! 处理渲染逻辑

use crate::boss::Boss;
use crate::components::*;
use crate::item::Item;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};
use gg_render::{Color, DrawCommand, Rect};

/// 渲染系统
/// 处理渲染逻辑
pub struct RenderSystem;

impl System for RenderSystem {
    fn name(&self) -> &str {
        "render_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        if let Some(buffer) = world.get_resource_mut::<DrawCommandBuffer>() {
            buffer.clear();
        }
        else {
            world.insert_resource(DrawCommandBuffer::new());
        }

        let entities: Vec<Entity> = world.query::<Sprite>().filter(|(_, sprite)| sprite.visible).map(|(e, _)| e).collect();

        for entity in entities {
            let (x, y, width, height) = {
                let sprite = match world.get_component::<Sprite>(entity) {
                    Some(s) => s,
                    None => continue,
                };
                if !sprite.visible {
                    continue;
                }
                let (w, h) = (sprite.width, sprite.height);
                if let Some(transform) = world.get_component::<Transform>(entity) {
                    (transform.x, transform.y, w, h)
                }
                else {
                    (0.0, 0.0, w, h)
                }
            };

            let is_player = world.get_component::<Player>(entity).is_some();
            let is_enemy = world.get_component::<Enemy>(entity).is_some();
            let is_bullet = world.get_component::<Bullet>(entity).is_some();
            let is_item = world.get_component::<Item>(entity).is_some();
            let is_boss = world.get_component::<Boss>(entity).is_some();

            let color = if is_player {
                Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 }
            }
            else if is_boss {
                Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 }
            }
            else if is_enemy {
                Color::RED
            }
            else if is_item {
                Color::GREEN
            }
            else if is_bullet {
                let is_player_bullet = world
                    .get_component::<Bullet>(entity)
                    .map(|b| matches!(b.shooter_type, ShooterType::Player))
                    .unwrap_or(true);
                if is_player_bullet { Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 } } else { Color::RED }
            }
            else {
                Color::WHITE
            };

            let command = DrawCommand::Rect {
                rect: Rect::new(x - width / 2.0, y - height / 2.0, width, height),
                color,
                corner_radius: 0.0,
            };

            if let Some(buffer) = world.get_resource_mut::<DrawCommandBuffer>() {
                buffer.push(command);
            }
        }
        Ok(())
    }
}
