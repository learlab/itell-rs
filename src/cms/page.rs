use serde::Serialize;

use super::ChunkData;

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

    // quiz
    pub quiz: Option<Vec<QuizItem>>,

    // cloze test
    pub cloze_test: Option<ClozeTest>,

    /// content chunks
    pub chunks: Vec<ChunkData>,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct QuizItem {
    pub question: String,
    pub answers: Vec<QuizAnswerItem>,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct QuizAnswerItem {
    pub answer: String,
    pub correct: bool,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct ClozeGap {
    pub start: usize,
    pub end: usize,
    pub gapped_text: String,
    pub original_word: Option<String>,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct ClozeTest {
    pub original_text: String,
    pub gaps: Vec<ClozeGap>,
}
