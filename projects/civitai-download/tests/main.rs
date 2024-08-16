use std::path::PathBuf;

use civitai::{RequestAllModels, RequestModel};
use trauma::{download::Download, downloader::DownloaderBuilder, Error};

#[test]
fn ready() {
    println!("it works!")
}

#[tokio::test]
async fn test_all() {
    let models = RequestAllModels::default().send().await.unwrap();
    println!("{:#?}", models.next_page());
}

#[tokio::test]
async fn test_model1() {
    let models = RequestModel::new(1).send().await.unwrap();
    println!("{:#?}", models);
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let reqwest_rs = "https://github.com/seanmonstar/reqwest/archive/refs/tags/v0.11.9.zip";
    let downloads = vec![Download::try_from(reqwest_rs).unwrap()];
    let downloader = DownloaderBuilder::new().directory(PathBuf::from("output")).build();
    downloader.download(&downloads).await;
    Ok(())
}
