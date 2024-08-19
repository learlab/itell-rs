use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Frontmatter<'a> {
    Title(&'a str),
    Slug(&'a str),
    Assignments(&'a [String]),
    Order(usize),
    Chunks(Vec<&'a str>),
    CRI(&'a [&'a QuestionAnswer]),
}

#[derive(Serialize, Debug)]
pub struct QuestionAnswer {
    pub question: String,
    pub answer: String,
    pub slug: String,
}
