#![warn(missing_docs)]
#![doc = include_str!("readme.md")]

pub use wae_ai as ai;
pub use wae_config as config;
#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub use wae_database as database;
pub use wae_distributed as distributed;
pub use wae_effect as effect;
pub use wae_email as email;
pub use wae_event as event;
pub use wae_https as https;
#[cfg(feature = "observability")]
pub use wae_observability as observability;
pub use wae_resilience as resilience;
pub use wae_scheduler as scheduler;
pub use wae_service as service;
pub use wae_session as session;
pub use wae_storage as storage;
pub use wae_testing as testing;
#[cfg(feature = "tools")]
pub use wae_tools as tools;
pub use wae_types as types;
pub use wae_websocket as websocket;

pub use types::{WaeError, WaeResult};
