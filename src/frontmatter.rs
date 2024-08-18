use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Frontmatter<'a> {
    Title(String),
    Slug(String),
    Assignments(&'a [String]),
    Order(usize),
    CRI(&'a [&'a QuestionAnswer]),
}

#[derive(Serialize, Debug)]
pub struct QuestionAnswer {
    pub question: String,
    pub answer: String,
    pub slug: String,
}
