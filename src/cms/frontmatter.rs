use serde::Serialize;

use super::{page::QuizItem, ChunkType, CriItem, PageParent};

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Frontmatter<'a> {
    Title(&'a str),
    Slug(&'a str),
    NextSlug(Option<&'a str>),
    Assignments(&'a [String]),
    Parent(Option<&'a PageParent>),
    Order(usize),
    Chunks(Vec<ChunkMeta<'a>>),
    CRI(&'a [&'a CriItem]),
    Quiz(Option<&'a Vec<QuizItem>>),
}

#[derive(Serialize, Debug)]
pub struct ChunkMeta<'a> {
    title: &'a str,
    slug: &'a str,
    #[serde(rename = "type")]
    chunk_type: &'a ChunkType,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    headings: Vec<Heading>,
}

impl<'a> ChunkMeta<'a> {
    pub fn new(title: &'a str, slug: &'a str, chunk_type: &'a ChunkType) -> Self {
        Self {
            title,
            slug,
            chunk_type,
            headings: vec![],
        }
    }

    pub fn add_headings(&mut self, headings: Vec<Heading>) {
        self.headings = headings;
    }
}

#[derive(Serialize, Debug)]
pub struct Heading {
    pub level: usize,
    pub slug: String,
    pub title: String,
}
