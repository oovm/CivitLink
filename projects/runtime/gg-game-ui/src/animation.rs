//! 游戏UI动画系统模块
//! 
//! 实现游戏UI的动画功能，包括属性动画、状态机动画等

use std::time::{Duration, Instant};
use gg_ecs::{World, Entity};

/// 动画曲线
#[derive(Debug, Clone)]
pub struct AnimationCurve {
    /// 关键帧
    pub keyframes: Vec<Keyframe>,
    /// 是否循环
    pub looped: bool,
    /// 播放速度
    pub speed: f32,
}

/// 关键帧
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// 时间
    pub time: f32,
    /// 值
    pub value: f32,
    /// 入切线
    pub in_tangent: f32,
    /// 出切线
    pub out_tangent: f32,
    /// 切线模式
    pub tangent_mode: TangentMode,
}

/// 切线模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TangentMode {
    /// 自动
    Auto,
    /// 线性
    Linear,
    /// 常量
    Constant,
    /// 自由
    Free,
}

impl Default for AnimationCurve {
    fn default() -> Self {
        Self {
            keyframes: Vec::new(),
            looped: false,
            speed: 1.0,
        }
    }
}

/// 动画状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    /// 播放中
    Playing,
    /// 暂停
    Paused,
    /// 停止
    Stopped,
    /// 完成
    Completed,
}

/// 动画组件
#[derive(Debug, Clone)]
pub struct Animation {
    /// 动画片段
    pub clips: Vec<AnimationClip>,
    /// 当前播放的动画
    pub current_clip: Option<String>,
    /// 动画状态
    pub state: AnimationState,
    /// 播放时间
    pub time: f32,
    /// 播放速度
    pub speed: f32,
    /// 是否循环
    pub looped: bool,
}

/// 动画片段
#[derive(Debug, Clone)]
pub struct AnimationClip {
    /// 名称
    pub name: String,
    /// 动画曲线
    pub curves: Vec<AnimationCurve>,
    /// 长度
    pub length: f32,
    /// 是否循环
    pub looped: bool,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            clips: Vec::new(),
            current_clip: None,
            state: AnimationState::Stopped,
            time: 0.0,
            speed: 1.0,
            looped: false,
        }
    }
}

/// 动画器组件
#[derive(Debug, Clone)]
pub struct Animator {
    /// 动画控制器
    pub controller: Option<Entity>,
    /// 当前状态
    pub current_state: String,
    /// 动画状态
    pub state: AnimationState,
    /// 播放时间
    pub time: f32,
    /// 播放速度
    pub speed: f32,
}

impl Default for Animator {
    fn default() -> Self {
        Self {
            controller: None,
            current_state: String::new(),
            state: AnimationState::Stopped,
            time: 0.0,
            speed: 1.0,
        }
    }
}

/// 动画系统
pub struct AnimationSystem {
    /// 最后更新时间
    last_update_time: Instant,
}

impl AnimationSystem {
    /// 创建新的动画系统
    pub fn new() -> Self {
        Self {
            last_update_time: Instant::now(),
        }
    }

    /// 更新动画
    fn update_animation(&self, entity: Entity, animation: &mut Animation, delta_time: f32, world: &mut World) {
        if animation.state != AnimationState::Playing {
            return;
        }

        if let Some(clip_name) = &animation.current_clip {
            // 查找动画片段
            for clip in &animation.clips {
                if clip.name == *clip_name {
                    // 更新播放时间
                    animation.time += delta_time * animation.speed * clip.length;

                    // 检查是否完成
                    if animation.time >= clip.length {
                        if clip.looped || animation.looped {
                            animation.time %= clip.length;
                        } else {
                            animation.time = clip.length;
                            animation.state = AnimationState::Completed;
                        }
                    }

                    // 应用动画
                    self.apply_animation(entity, clip, animation.time, world);
                    break;
                }
            }
        }
    }

    /// 应用动画
    fn apply_animation(&self, entity: Entity, clip: &AnimationClip, time: f32, world: &mut World) {
        // 应用动画曲线到组件属性
        // 这里只是一个占位实现
        // 实际需要根据动画曲线类型应用到不同的组件属性
    }

    /// 更新动画器
    fn update_animator(&self, entity: Entity, animator: &mut Animator, delta_time: f32, world: &mut World) {
        if animator.state != AnimationState::Playing {
            return;
        }

        // 更新播放时间
        animator.time += delta_time * animator.speed;

        // 这里应该根据动画控制器的状态机逻辑更新动画
        // 实际需要实现状态机逻辑
    }
}

impl gg_ecs::System for AnimationSystem {
    fn name(&self) -> &str {
        "AnimationSystem"
    }

    fn execute(&mut self, world: &mut World) -> gg_error::GResult<()> {
        // 计算 delta time
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update_time).as_secs_f32();
        self.last_update_time = now;

        // 更新所有动画组件
        for entity in world.entities() {
            if let Some(mut animation) = world.get_component_mut::<Animation>(entity) {
                self.update_animation(entity, &mut animation, delta_time, world);
            }
        }

        // 更新所有动画器组件
        for entity in world.entities() {
            if let Some(mut animator) = world.get_component_mut::<Animator>(entity) {
                self.update_animator(entity, &mut animator, delta_time, world);
            }
        }

        Ok(())
    }
}

/// 补间动画工具
pub mod tween {
    use super::*;

    /// 缓动函数
    pub enum Ease {
        /// 线性
        Linear,
        /// 缓入
        InQuad,
        /// 缓出
        OutQuad,
        /// 缓入缓出
        InOutQuad,
        /// 缓入
        InCubic,
        /// 缓出
        OutCubic,
        /// 缓入缓出
        InOutCubic,
        /// 缓入
        InQuart,
        /// 缓出
        OutQuart,
        /// 缓入缓出
        InOutQuart,
        /// 缓入
        InQuint,
        /// 缓出
        OutQuint,
        /// 缓入缓出
        InOutQuint,
        /// 缓入
        InSine,
        /// 缓出
        OutSine,
        /// 缓入缓出
        InOutSine,
        /// 缓入
        InExpo,
        /// 缓出
        OutExpo,
        /// 缓入缓出
        InOutExpo,
        /// 缓入
        InCirc,
        /// 缓出
        OutCirc,
        /// 缓入缓出
        InOutCirc,
        /// 缓入
        InElastic,
        /// 缓出
        OutElastic,
        /// 缓入缓出
        InOutElastic,
        /// 缓入
        InBack,
        /// 缓出
        OutBack,
        /// 缓入缓出
        InOutBack,
        /// 缓入
        InBounce,
        /// 缓出
        OutBounce,
        /// 缓入缓出
        InOutBounce,
    }

    /// 补间动画
    pub struct Tween {
        /// 目标实体
        pub target: Entity,
        /// 起始值
        pub start_value: f32,
        /// 结束值
        pub end_value: f32,
        /// 持续时间
        pub duration: f32,
        /// 已用时间
        pub elapsed: f32,
        /// 缓动函数
        pub ease: Ease,
        /// 是否循环
        pub looped: bool,
        /// 是否正在播放
        pub playing: bool,
    }

    impl Tween {
        /// 创建新的补间动画
        pub fn new(target: Entity, start_value: f32, end_value: f32, duration: f32, ease: Ease) -> Self {
            Self {
                target,
                start_value,
                end_value,
                duration,
                elapsed: 0.0,
                ease,
                looped: false,
                playing: true,
            }
        }

        /// 更新补间动画
        pub fn update(&mut self, delta_time: f32) -> bool {
            if !self.playing {
                return false;
            }

            self.elapsed += delta_time;

            if self.elapsed >= self.duration {
                if self.looped {
                    self.elapsed %= self.duration;
                } else {
                    self.playing = false;
                    return false;
                }
            }

            true
        }

        /// 获取当前值
        pub fn get_value(&self) -> f32 {
            let t = self.elapsed / self.duration;
            let t = self.apply_ease(t);
            self.start_value + (self.end_value - self.start_value) * t
        }

        /// 应用缓动函数
        fn apply_ease(&self, t: f32) -> f32 {
            match self.ease {
                Ease::Linear => t,
                Ease::InQuad => t * t,
                Ease::OutQuad => t * (2.0 - t),
                Ease::InOutQuad => if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t },
                Ease::InCubic => t * t * t,
                Ease::OutCubic => (t - 1.0) * (t - 1.0) * (t - 1.0) + 1.0,
                Ease::InOutCubic => if t < 0.5 { 4.0 * t * t * t } else { (t - 1.0) * (2.0 * t - 2.0) * (2.0 * t - 2.0) + 1.0 },
                _ => t, // 其他缓动函数的实现省略
            }
        }
    }

    /// 补间动画系统
    pub struct TweenSystem {
        /// 补间动画列表
        tweens: Vec<Tween>,
        /// 最后更新时间
        last_update_time: Instant,
    }

    impl TweenSystem {
        /// 创建新的补间动画系统
        pub fn new() -> Self {
            Self {
                tweens: Vec::new(),
                last_update_time: Instant::now(),
            }
        }

        /// 添加补间动画
        pub fn add_tween(&mut self, tween: Tween) {
            self.tweens.push(tween);
        }
    }

    impl gg_ecs::System for TweenSystem {
        fn name(&self) -> &str {
            "TweenSystem"
        }

        fn execute(&mut self, world: &mut World) -> gg_error::GResult<()> {
            // 计算 delta time
            let now = Instant::now();
            let delta_time = now.duration_since(self.last_update_time).as_secs_f32();
            self.last_update_time = now;

            // 更新所有补间动画
            self.tweens.retain(|mut tween| {
                if tween.update(delta_time) {
                    // 应用补间动画值
                    // 这里只是一个占位实现
                    // 实际需要根据补间动画的目标属性应用值
                    true
                } else {
                    false
                }
            });

            Ok(())
        }
    }
}
