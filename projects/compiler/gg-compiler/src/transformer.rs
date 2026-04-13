//! 编译转换器模块
//! 定义编译流水线中每个阶段的标准接口

use gg_core::GResult;

use crate::{
    artifact::{ArtifactKey, ArtifactSet},
    context::BuildContext,
};

/// 编译转换器 trait，定义编译流水线中每个阶段的标准接口
pub trait Transformer {
    /// 获取转换器名称
    fn name(&self) -> &str;

    /// 获取此转换器依赖的产物键列表
    fn input_keys(&self) -> Vec<ArtifactKey>;

    /// 获取此转换器产出的产物键列表
    fn output_keys(&self) -> Vec<ArtifactKey>;

    /// 执行转换
    fn transform(&self, inputs: &ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet>;
}
