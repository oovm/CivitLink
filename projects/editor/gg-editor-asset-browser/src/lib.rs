#![warn(missing_docs)]

//! GG 编辑器资源浏览器面板模块
//! 提供项目资源目录浏览、文件导入和引用查找功能

pub mod panel;

pub use panel::{
    AssetBatchOperationResult, AssetBrowserPanel, AssetContextMenu, AssetMetadata, AssetOperationResult, AssetPreviewData,
    AssetReference, AssetType, DirectoryNode, FileOperation, PendingImport, filter_tree,
};
