#![warn(missing_docs)]

//! GG 引擎核心模块
//! 提供基础类型、错误处理和平台抽象接口

pub use gg_error::{GError, GErrorKind, GResult};

/// 平台抽象层
pub mod platform;

/// 插件系统
pub mod plugin {
    use gg_ecs::{Entity, Resource, System, World};
    use gg_error::{GError, GErrorKind, GResult};
    use std::{
        any::{Any, TypeId},
        collections::HashSet,
    };

    /// 类型擦除的资源条目
    struct ResourceEntry {
        /// 类型擦除的资源
        resource: Box<dyn Any + Send + Sync>,
        /// 资源类型 ID
        type_id: TypeId,
    }

    /// 类型擦除的组件条目
    struct ComponentEntry {
        /// 目标实体
        entity: Entity,
        /// 类型擦除的组件
        component: Box<dyn Any + Send + Sync>,
        /// 组件类型 ID
        type_id: TypeId,
    }

    /// 插件注册器
    ///
    /// 在插件的 build 阶段收集注册信息，包括系统、资源和依赖。
    /// 在 apply 阶段统一将注册信息应用到 World。
    pub struct PluginRegistrar {
        /// 待注册的系统列表
        systems: Vec<Box<dyn System>>,
        /// 待注册的全局资源列表
        resources: Vec<ResourceEntry>,
        /// 待注册的组件条目列表
        components: Vec<ComponentEntry>,
        /// 依赖插件名称列表
        dependencies: Vec<String>,
    }

    impl PluginRegistrar {
        /// 创建新的插件注册器
        pub fn new() -> Self {
            Self { systems: Vec::new(), resources: Vec::new(), components: Vec::new(), dependencies: Vec::new() }
        }

        /// 注册系统
        ///
        /// 将系统添加到待注册队列，在 apply 阶段注册到 World。
        pub fn register_system(&mut self, system: Box<dyn System>) {
            self.systems.push(system);
        }

        /// 插入全局资源
        ///
        /// 将资源添加到待注册队列，在 apply 阶段插入到 World 的全局资源。
        pub fn insert_resource<T: Resource + 'static>(&mut self, resource: T) {
            let type_id = TypeId::of::<T>();
            self.resources.push(ResourceEntry { resource: Box::new(resource), type_id });
        }

        /// 向实体插入组件
        ///
        /// 将组件添加到待注册队列，在 apply 阶段添加到指定实体。
        pub fn insert_component<T: gg_ecs::Component + 'static>(&mut self, entity: Entity, component: T) {
            let type_id = TypeId::of::<T>();
            self.components.push(ComponentEntry { entity, component: Box::new(component), type_id });
        }

        /// 添加依赖
        ///
        /// 声明当前插件依赖的另一个插件。
        pub fn add_dependency(&mut self, plugin_name: &str) {
            self.dependencies.push(plugin_name.to_string());
        }

        /// 将收集的注册信息应用到 World
        ///
        /// 依次执行：插入全局资源 → 插入组件 → 注册系统
        pub fn apply(self, world: &mut World) -> GResult<()> {
            for entry in self.resources {
                world.insert_resource_raw(entry.resource, entry.type_id);
            }
            for entry in self.components {
                world.add_component_raw(entry.entity, entry.component, entry.type_id)?;
            }
            for system in self.systems {
                world.register_system(system);
            }
            Ok(())
        }

        /// 获取依赖列表
        pub fn dependencies(&self) -> &[String] {
            &self.dependencies
        }
    }

    /// 插件 trait
    ///
    /// 所有引擎插件都需要实现此 trait。
    /// 通过 build 方法注册系统、资源和依赖，
    /// 通过 initialize/shutdown 管理插件生命周期。
    pub trait Plugin {
        /// 插件名称
        fn name(&self) -> &str;

        /// 构建插件
        ///
        /// 通过 PluginRegistrar 注册系统、资源和依赖。
        /// 此阶段仅收集注册信息，不修改 World。
        fn build(&self, registrar: &mut PluginRegistrar) {
            let _ = registrar;
        }

        /// 插件依赖列表
        ///
        /// 返回此插件依赖的其他插件名称列表。
        fn dependencies(&self) -> Vec<&str> {
            Vec::new()
        }

        /// 初始化插件
        fn initialize(&self) -> GResult<()>;

        /// 关闭插件
        fn shutdown(&self) -> GResult<()>;
    }

    /// 插件管理器
    ///
    /// 管理插件的加载、依赖检查和生命周期。
    pub struct PluginManager {
        /// 已注册的插件列表
        plugins: Vec<Box<dyn Plugin>>,
        /// 已注册的插件名称集合
        plugin_names: HashSet<String>,
    }

    impl PluginManager {
        /// 创建新的插件管理器
        pub fn new() -> Self {
            Self { plugins: Vec::new(), plugin_names: HashSet::new() }
        }

        /// 注册插件
        ///
        /// 将插件添加到管理器中，检查依赖是否满足。
        pub fn register(&mut self, plugin: Box<dyn Plugin>) -> GResult<()> {
            for dep in plugin.dependencies() {
                if !self.plugin_names.contains(dep) {
                    return Err(GError {
                        kind: GErrorKind::Plugin,
                        message: format!("Plugin '{}' depends on '{}' which is not registered", plugin.name(), dep),
                    });
                }
            }
            self.plugin_names.insert(plugin.name().to_string());
            self.plugins.push(plugin);
            Ok(())
        }

        /// 构建所有插件
        ///
        /// 依次调用每个插件的 build 方法，收集注册信息并应用到 World。
        pub fn build_all(&mut self, world: &mut World) -> GResult<()> {
            for plugin in &self.plugins {
                let mut registrar = PluginRegistrar::new();
                plugin.build(&mut registrar);
                registrar.apply(world)?;
            }
            Ok(())
        }

        /// 初始化所有插件
        pub fn initialize_all(&self) -> GResult<()> {
            for plugin in &self.plugins {
                plugin.initialize()?;
            }
            Ok(())
        }

        /// 关闭所有插件
        ///
        /// 按注册的逆序关闭插件。
        pub fn shutdown_all(&self) -> GResult<()> {
            for plugin in self.plugins.iter().rev() {
                plugin.shutdown()?;
            }
            Ok(())
        }
    }
}

pub use platform::{
    DirEntry, FileMetadata, FileSystem, FileType, Input, InputEvent, KeyCode, KeyState, PlatformServices, PointerAction,
    PointerButton, RuntimePlatform, RuntimeThread, Time, Window, WindowConfig, WindowEvent,
};
