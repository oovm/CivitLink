#[derive(Debug)]
pub enum GErrorKind {
    Io,
    Asset,
    Platform,
    Ecs,
    Plugin,
    Runtime,
    Other,
}

#[derive(Debug)]
pub struct GError {
    pub kind: GErrorKind,
    pub message: String,
}

pub type GResult<T> = std::result::Result<T, GError>;
