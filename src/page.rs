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

    let data = resp.get("data").expect("no data in volume response");
    let attributes = data.get("attributes").expect("volume as no attributes");

    return Ok(attributes
        .get("Pages")
        .and_then(|p| p.get("data").and_then(|d| d.as_array()))
        .unwrap_or(&Vec::new())
        .to_owned());
}

pub fn clean_pages(pages: Vec<Value>) -> Vec<PageData> {
    pages
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let attributes = page.get("attributes").unwrap();

            let title = attributes
                .get("Title")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let slug = attributes
                .get("Slug")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let assignments = if attributes
                .get("HasSummary")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                vec![String::from("summary")]
            } else {
                Vec::new()
            };

            let default_content = Vec::new();
            let content = attributes
                .get("Content")
                .and_then(|v| v.as_array())
                .unwrap_or(&default_content);

            let chunks = content
                .iter()
                .map(|chunk| {
                    let slug = chunk.get("Slug").unwrap().as_str().unwrap().to_string();
                    let text = chunk.get("MDX").unwrap().as_str().unwrap().to_string();
                    let title = chunk.get("Header").unwrap().as_str().unwrap().to_string();
                    let show_header = chunk.get("ShowHeader").unwrap().as_bool().unwrap();
                    let mut cri: Option<QuestionAnswer> = None;
                    if let (Some(question), Some(answer)) = (
                        chunk.get("Question").and_then(|q| q.as_str()),
                        chunk.get("ConstructedResponse").and_then(|a| a.as_str()),
                    ) {
                        cri = Some(QuestionAnswer {
                            slug: slug.clone(),
                            question: question.to_string(),
                            answer: answer.to_string(),
                        });
                    }

                    let mut depth: usize = 2;

                    if let Some(header_level) = chunk.get("HeaderLevel").and_then(|h| h.as_str()) {
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
                order: index,
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
    fm.insert("title", Frontmatter::Title(page.title.as_str()));
    fm.insert("slug", Frontmatter::Slug(page.slug.as_str()));
    fm.insert("order", Frontmatter::Order(page.order));
    fm.insert("assignments", Frontmatter::Assignments(&page.assignments));
    fm.insert(
        "chunks",
        Frontmatter::Chunks(
            page.chunks
                .iter()
                .map(|c| c.slug.as_str())
                .collect::<Vec<&str>>(),
        ),
    );

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
