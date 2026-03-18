fn main() {
    if cfg!(feature = "pulsar-backend") {
        if let Ok(protoc_path) = protoc_bin_vendored::protoc_bin_path() {
            println!("cargo:rustc-env=PROTOC={}", protoc_path.display());
        }
    }
}
