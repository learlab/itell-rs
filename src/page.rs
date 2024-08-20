use crate::frontmatter::{Frontmatter, QuestionAnswer};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use ureq;

use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Error, Debug)]
enum RequestError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] ureq::Error),

    #[error("Failed to read response body: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Server returned an error: {status}")]
    ServerError { status: u16 },
}

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

const BASE_URL: &str = "https://itell-strapi-um5h.onrender.com/api/texts/";
const QUERY: &str = "?populate%5BPages%5D%5Bpopulate%5D%5BContent%5D=true";

pub struct VolumeResponse(Vec<serde_json::Value>);

pub fn get_pages_by_volume_id(volume_id: i32) -> Result<VolumeResponse> {
    let response = ureq::get(format!("{}{}{}", BASE_URL, volume_id, QUERY).as_str())
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => RequestError::ServerError { status: code },
            other => RequestError::HttpError(other),
        })
        .context("Failed to send request")?;

    let body: serde_json::Value = response
        .into_json()
        .context("Failed to read response body")?;

    let data = body.get("data").context("no data in volume response")?;
    let attributes = data.get("attributes").context("volume as no attributes")?;

    return Ok(VolumeResponse(
        attributes
            .get("Pages")
            .and_then(|p| p.get("data").and_then(|d| d.as_array()))
            .unwrap_or(&Vec::new())
            .to_owned(),
    ));
}

pub fn clean_pages(resp: VolumeResponse) -> Vec<PageData> {
    resp.0
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

                    let question = chunk.get("Question").and_then(|q| q.as_str());
                    let answer = chunk.get("ConstructedResponse").and_then(|a| a.as_str());

                    let cri = match (question, answer) {
                        (Some(question), Some(answer)) => Some(QuestionAnswer {
                            slug: slug.clone(),
                            question: question.to_string(),
                            answer: answer.to_string(),
                        }),
                        _ => None,
                    };

                    let depth = match chunk.get("HeaderLevel").and_then(|h| h.as_str()) {
                        Some("h3") => 3,
                        Some("h4") => 4,
                        _ => 2,
                    };

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

pub fn write_page(page: &PageData, output_dir: &str) -> Result<()> {
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
    .context(format!("failed to write to page {}", page.slug))?;

    Ok(())
}
