/// FileInfo stores the parsed result of a file retrieved from downloaded JSON
#[derive(Deserialize, Clone, Debug)]
pub struct FileInfo {
    pub url: String,
    pub name: String,
    pub size: u64,
}
