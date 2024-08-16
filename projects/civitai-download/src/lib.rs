pub use trauma::downloader::{Downloader, DownloaderBuilder};

pub use crate::{
    all_models_info::{AllModels, RequestAllModels},
    model_info::{ModelInfo, RequestModel},
    model_version::RequestModelVersion,
};

mod all_models_info;
mod model_info;
mod model_version;

pub type CivitResult<T> = Result<T, trauma::Error>;
