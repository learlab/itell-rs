use super::chunk::ChunkData;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PageParent {
    title: String,
    slug: String,
}

impl PageParent {
    pub fn new(title: String, slug: String) -> Self {
        Self { title, slug }
    }
}

#[derive(Debug)]
pub struct PageData {
    /// page title
    pub title: String,
    /// page slug, must be unique within the volume
    pub slug: String,

    /// parent page, if any
    pub parent: Option<PageParent>,

    /// relative order in the volume
    pub order: usize,

    /// evaluation assignments, ["summary", "quiz", ...]
    pub assignments: Vec<String>,

    /// content chunks
    pub chunks: Vec<ChunkData>,
}
