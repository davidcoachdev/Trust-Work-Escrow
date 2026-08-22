use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UploadResult {
    pub url: String,
    pub hash: String,
}

#[cfg(feature = "server")]
pub async fn upload_image(_bytes: Vec<u8>, _filename: &str) -> Result<UploadResult, String> {
    // TODO: Cloudinary upload (free 25GB) — por ahora mock
    Ok(UploadResult {
        url: "https://res.cloudinary.com/demo/image/upload/v1/mock.jpg".to_string(),
        hash: "a".repeat(64),
    })
}

#[cfg(feature = "server")]
pub async fn upload_doc(_bytes: Vec<u8>, _filename: &str) -> Result<UploadResult, String> {
    // TODO: R2 10GB free — por ahora mock
    Ok(UploadResult {
        url: "https://r2.cloudflarestorage.com/mock.pdf".to_string(),
        hash: "b".repeat(64),
    })
}
