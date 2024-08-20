use crate::frontmatter::{ChunkMeta, ChunkType, Frontmatter, QuestionAnswer};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;
use ureq;

use anyhow::{Context, Ok};
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
    content: String,
    cri: Option<QuestionAnswer>,
    show_header: bool,
    chunk_type: ChunkType,
}

const BASE_URL: &str = "https://itell-strapi-um5h.onrender.com/api/texts/";
const QUERY: &str = "?populate%5BPages%5D%5Bpopulate%5D%5BContent%5D=true";

pub struct VolumeResponse(Vec<serde_json::Value>);

pub fn get_pages_by_volume_id(volume_id: i32) -> anyhow::Result<VolumeResponse> {
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
            .context("no pages in volume response")?
            .to_owned(),
    ));
}

fn get_attribute<T>(value: &Value, attribute: &str) -> Option<T>
where
    T: FromStr,
{
    value.get(attribute).and_then(|v| match v {
        Value::String(s) => T::from_str(s).ok(),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                T::from_str(&f.to_string()).ok()
            } else {
                None
            }
        }
        Value::Bool(b) => T::from_str(&b.to_string()).ok(),
        _ => None,
    })
}

pub fn clean_pages(resp: VolumeResponse) -> anyhow::Result<Vec<PageData>> {
    resp.0
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let attributes = page.get("attributes").context("page has no attributes")?;

            let title = get_attribute(attributes, "Title")
                .context(format!("page '{}' must set title", index))?;

            let slug = get_attribute(attributes, "Slug")
                .context(format!("page '{}' must set slug", &title))?;
            let has_summary: bool = get_attribute(attributes, "HasSummary")
                .context(format!("page '{}' must set HasSummary", &title))?;

            let assignments = if has_summary {
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
                    let chunk_title: String = get_attribute(chunk, "Header").context(format!(
                        "chunk '{}' in page '{}' must set Header",
                        index, &title
                    ))?;
                    let chunk_slug: String = get_attribute(chunk, "Slug").context(format!(
                        "chunk '{}' in page '{}' must set Slug",
                        index, &title
                    ))?;

                    let content: String = get_attribute(chunk, "MDX").context(format!(
                        "chunk '{}' in page '{}' must set MDX",
                        index, &title
                    ))?;

                    let show_header: bool = get_attribute(chunk, "ShowHeader").unwrap_or_default();

                    let question: Option<String> = get_attribute(chunk, "Question");
                    let answer: Option<String> = get_attribute(chunk, "ConstructedResponse");

                    let component: Option<String> = get_attribute(chunk, "__component");
                    let chunk_type = component.map_or(ChunkType::Regular, |c| {
                        if c == "page.plain-chunk" {
                            ChunkType::Plain
                        } else {
                            ChunkType::Regular
                        }
                    });

                    let cri = match (question, answer) {
                        (Some(question), Some(answer)) => Some(QuestionAnswer {
                            slug: chunk_slug.clone(),
                            question,
                            answer,
                        }),
                        _ => None,
                    };

                    let header_level: Option<String> = get_attribute(chunk, "HeaderLevel");

                    let depth = match header_level.as_deref() {
                        Some("h3") => 3,
                        Some("h4") => 4,
                        _ => 2,
                    };

                    Ok(ChunkData {
                        title: chunk_title,
                        slug: chunk_slug,
                        content,
                        depth,
                        show_header,
                        cri,
                        chunk_type,
                    })
                })
                .collect::<anyhow::Result<Vec<ChunkData>>>();

            Ok(PageData {
                title,
                slug,
                assignments,
                chunks: chunks.context("failed to parse chunk")?,
                order: index,
            })
        })
        .collect::<anyhow::Result<Vec<PageData>>>()
}

pub fn write_page(page: &PageData, output_dir: &str) -> anyhow::Result<()> {
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
                .map(|c| ChunkMeta::new(c.slug.as_str(), &c.chunk_type))
                .collect::<Vec<ChunkMeta>>(),
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
                chunk.content
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
