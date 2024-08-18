use crate::frontmatter::{Frontmatter, QuestionAnswer};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug)]
pub struct PageData {
    title: String,
    slug: String,
    order: usize,
    assignments: Vec<String>,
    chunks: Vec<ChunkData>,
}

#[derive(Debug)]
struct ChunkData {
    title: String,
    slug: String,
    depth: usize,
    text: String,
    cri: Option<QuestionAnswer>,
    show_header: bool,
}

pub async fn get_pages_by_volume_id(volume_id: i32) -> Result<Vec<Value>, reqwest::Error> {
    let resp: serde_json::Value = reqwest::get(format!(
        "https://itell-strapi-um5h.onrender.com/api/texts/{}?populate%5BPages%5D%5Bpopulate%5D%5BContent%5D=true",
        volume_id,
    ))
    .await?
    .json()
    .await?;

    let data = resp.get("data").unwrap();
    let attributes = data.get("attributes").unwrap();
    let pages = attributes
        .get("Pages")
        .unwrap()
        .get("data")
        .unwrap()
        .as_array()
        .unwrap();

    return Ok(pages.clone());
}

pub fn clean_pages(pages: Vec<Value>) -> Vec<PageData> {
    pages
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let attributes = page.get("attributes").unwrap();

            let title = attributes
                .get("Title")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let slug = attributes
                .get("Slug")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let mut assignments: Vec<String> = Vec::new();
            if attributes.get("HasSummary").unwrap().as_bool().unwrap() {
                assignments.push(String::from("summary"));
            }

            let content = attributes.get("Content").unwrap().as_array().unwrap();

            let chunks = content
                .iter()
                .map(|chunk| {
                    let slug = chunk.get("Slug").unwrap().as_str().unwrap().to_string();
                    let text = chunk.get("MDX").unwrap().as_str().unwrap().to_string();
                    let title = chunk.get("Header").unwrap().as_str().unwrap().to_string();
                    let show_header = chunk.get("ShowHeader").unwrap().as_bool().unwrap();

                    let mut cri: Option<QuestionAnswer> = None;
                    if chunk.get("Question").is_some() && chunk.get("ConstructedResponse").is_some()
                    {
                        cri = Some(QuestionAnswer {
                            slug: slug.clone(),
                            question: chunk.get("Question").unwrap().as_str().unwrap().to_string(),
                            answer: chunk
                                .get("ConstructedResponse")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_string(),
                        });
                    }

                    let mut depth: usize = 2;

                    if let Some(header_level) = chunk.get("HeaderLevel").unwrap().as_str() {
                        if header_level == "h3" {
                            depth = 3;
                        }
                        if header_level == "h4" {
                            depth = 4;
                        }
                    }

                    ChunkData {
                        slug,
                        text,
                        depth,
                        title,
                        show_header,
                        cri,
                    }
                })
                .collect::<Vec<ChunkData>>();

            PageData {
                title,
                slug,
                assignments,
                chunks,
                order: index, // chunks: content.split(" ").map(|s| s.to_string()).collect(),
            }
        })
        .collect::<Vec<PageData>>()
}

pub fn write_page(page: &PageData, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(format!("{}/{}.md", output_dir, page.slug))
        .unwrap();

    let mut fm: BTreeMap<&str, Frontmatter> = BTreeMap::new();
    fm.insert("title", Frontmatter::Title(page.title.clone()));
    fm.insert("slug", Frontmatter::Slug(page.slug.clone()));
    fm.insert("order", Frontmatter::Order(page.order));
    fm.insert("assignments", Frontmatter::Assignments(&page.assignments));

    let mut cri = Vec::<&QuestionAnswer>::new();

    page.chunks.iter().for_each(|chunk| {
        if let Some(ref item) = chunk.cri {
            cri.push(item);
        }
    });

    fm.insert("cri", Frontmatter::CRI(&cri));

    let content_string = page
        .chunks
        .iter()
        .map(|chunk| {
            let header_class = if chunk.show_header { "" } else { " .sr-only" };
            format!(
                "{} {} {{#{}{}}} \n\n{}\n",
                "#".repeat(chunk.depth),
                chunk.title,
                chunk.slug,
                header_class,
                chunk.text
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    writeln!(
        file,
        r#"---
{}---

{}"#,
        serde_yaml_ng::to_string(&fm)?,
        content_string
    )
    .unwrap();

    Ok(())
}
