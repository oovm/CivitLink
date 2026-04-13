/// 二维变换
///
/// 描述一个对象在二维空间中的位置、缩放、旋转和层级。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// 位置 `[x, y]`
    pub position: [f32; 2],
    /// 缩放 `[sx, sy]`
    pub scale: [f32; 2],
    /// 旋转角度（弧度）
    pub rotation: f32,
    /// Z 轴层级，值越大越靠前
    pub z_index: f32,
}

impl Transform {
    /// 单位变换
    ///
    /// 位置为原点，缩放为 `(1.0, 1.0)`，无旋转，层级为 `0.0`。
    pub const IDENTITY: Self = Self { position: [0.0, 0.0], scale: [1.0, 1.0], rotation: 0.0, z_index: 0.0 };

    /// 创建单位变换
    pub fn identity() -> Self {
        Self::IDENTITY
    }

    /// 创建具有指定位置的变换
    ///
    /// # 参数
    ///
    /// - `pos` - 位置 `[x, y]`
    pub fn with_position(pos: [f32; 2]) -> Self {
        Self { position: pos, ..Self::IDENTITY }
    }

    /// 创建具有指定 Z 层级的变换
    ///
    /// # 参数
    ///
    /// - `z` - Z 层级值
    pub fn with_z_index(z: f32) -> Self {
        Self { z_index: z, ..Self::IDENTITY }
    }
}
