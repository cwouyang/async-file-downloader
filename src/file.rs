#[derive(Deserialize, Clone, Debug)]
pub struct FileInfo {
    pub url: String,
    pub name: String,
    pub size: u64,
}
