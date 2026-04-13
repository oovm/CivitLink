use crate::format::BytecodeValue;

/// 宿主接口，允许字节码解释器调用引擎 Rust API
pub trait Host {
    /// 创建实体，返回实体 ID
    fn spawn_entity(&mut self) -> u64;

    /// 销毁实体
    fn destroy_entity(&mut self, entity_id: u64);

    /// 添加组件
    fn add_component(&mut self, entity_id: u64, component_type: &str, value: BytecodeValue);

    /// 获取组件字段值
    fn get_component_field(&mut self, entity_id: u64, component_type: &str, field: &str) -> Option<BytecodeValue>;

    /// 设置组件字段值
    fn set_component_field(&mut self, entity_id: u64, component_type: &str, field: &str, value: BytecodeValue);

    /// 调用宿主函数
    fn call_host_function(&mut self, name: &str, args: Vec<BytecodeValue>) -> Option<BytecodeValue>;
}
