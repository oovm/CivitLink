//! GWG Engine 核心框架
//!
//! 提供引擎核心功能的统一入口，重新导出所有子模块。

pub use gwg_ecs as ecs;
pub use gwg_asset as asset;
pub use gwg_reflection as reflection;
pub use gwg_schedule as schedule;
pub use gwg_world as world;

pub mod prelude {
    //! 常用类型和 trait 的预导入模块

    pub use gwg_ecs::prelude::*;
    pub use gwg_asset::prelude::*;
    pub use gwg_reflection::prelude::*;
    pub use gwg_schedule::prelude::*;
    pub use gwg_world::prelude::*;
}
