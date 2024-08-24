use serde::Serialize;

use crate::page::PageParent;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Frontmatter<'a> {
    Title(&'a str),
    Slug(&'a str),
    Assignments(&'a [String]),
    Parent(Option<&'a PageParent>),
    Order(usize),
    Chunks(Vec<ChunkMeta<'a>>),
    CRI(&'a [&'a QuestionAnswer]),
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
