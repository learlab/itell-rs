use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ChunkData {
    pub title: String,
    pub slug: String,
    pub depth: usize,
    pub content: String,
    pub cri: Option<QuestionAnswer>,
    pub show_header: bool,
    pub chunk_type: ChunkType,
}

#[derive(Serialize, Debug)]
pub enum ChunkType {
    #[serde(rename = "regular")]
    Regular,
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "video")]
    Video,
}

#[derive(Serialize, Debug)]
pub struct QuestionAnswer {
    pub question: String,
    pub answer: String,
    pub slug: String,
}
