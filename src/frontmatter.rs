use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Frontmatter<'a> {
    Title(&'a str),
    Slug(&'a str),
    Assignments(&'a [String]),
    Order(usize),
    Chunks(Vec<ChunkMeta<'a>>),
    CRI(&'a [&'a QuestionAnswer]),
}

#[derive(Serialize, Debug)]
pub struct ChunkMeta<'a> {
    slug: &'a str,
    #[serde(rename = "type")]
    chunk_type: &'a ChunkType,
}

impl<'a> ChunkMeta<'a> {
    pub fn new(slug: &'a str, chunk_type: &'a ChunkType) -> Self {
        Self { slug, chunk_type }
    }
}

#[derive(Serialize, Debug)]
pub enum ChunkType {
    #[serde(rename = "regular")]
    Regular,
    #[serde(rename = "plain")]
    Plain,
}

#[derive(Serialize, Debug)]
pub struct QuestionAnswer {
    pub question: String,
    pub answer: String,
    pub slug: String,
}
