use anyhow::{Context, Ok};
use regex::Regex;
use serde_json::Value;
use std::{collections::BTreeMap, str::FromStr};
use thiserror::Error;

use super::{
    chunk::{ChunkData, ChunkType, QuestionAnswer},
    frontmatter::{ChunkMeta, Frontmatter, Heading},
    page::{PageData, PageParent},
};

const BASE_URL: &str = "https://itell-strapi-um5h.onrender.com/api/texts/";
const QUERY: &str = "?populate[Pages][fields][0]=*&populate[Pages][populate][Content]=true&populate[Pages][populate][Chapter][fields][0]=Title&populate[Pages][populate][Chapter][fields][1]=Slug";

pub struct VolumeData(Vec<serde_json::Value>);

pub fn get_pages_by_volume_id(volume_id: i32) -> anyhow::Result<VolumeData> {
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

    return Ok(VolumeData(
        attributes
            .get("Pages")
            .and_then(|p| p.get("data").and_then(|d| d.as_array()))
            .context("no pages in volume response")?
            .to_owned(),
    ));
}

pub fn clean_pages(resp: VolumeData) -> anyhow::Result<Vec<PageData>> {
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

            let parent_attributes = attributes
                .get("Chapter")
                .and_then(|v| v.as_object())
                .and_then(|v| v.get("data"))
                .and_then(|v| v.get("attributes"));

            let parent = match parent_attributes {
                Some(p) => Some(PageParent::new(
                    get_attribute(p, "Title")
                        .context(format!("chapter '{}' must set title", &title))?,
                    get_attribute(p, "Slug")
                        .context(format!("chapter '{}' must set slug", &title))?,
                )),
                None => None,
            };

            let default_content = Vec::new();
            let content = attributes
                .get("Content")
                .and_then(|v| v.as_array())
                .unwrap_or(&default_content);

            let chunks: anyhow::Result<Vec<ChunkData>> = content
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
                        depth,
                        content,
                        cri,
                        show_header,
                        chunk_type,
                    })
                })
                .collect();

            Ok(PageData {
                title,
                slug,
                parent,
                order: index,
                assignments,
                chunks: chunks.context("failed to parse chunk")?,
            })
        })
        .collect()
}

pub fn serialize_page(page: &PageData) -> anyhow::Result<String> {
    let mut fm: BTreeMap<&str, Frontmatter> = BTreeMap::new();
    fm.insert("title", Frontmatter::Title(page.title.as_str()));
    fm.insert("slug", Frontmatter::Slug(page.slug.as_str()));
    fm.insert("order", Frontmatter::Order(page.order));
    fm.insert("assignments", Frontmatter::Assignments(&page.assignments));
    fm.insert("parent", Frontmatter::Parent(page.parent.as_ref()));

    let mut cri = Vec::<&QuestionAnswer>::new();
    let mut chunks = Vec::<ChunkMeta>::new();
    let mut page_body = String::new();

    page.chunks.iter().for_each(|chunk| {
        let mut chunk_headings: Vec<Heading> = vec![];
        let mut chunk_meta =
            ChunkMeta::new(chunk.title.as_str(), chunk.slug.as_str(), &chunk.chunk_type);

        // Populate cri
        if let Some(ref item) = chunk.cri {
            cri.push(item);
        }

        // Generate page_body
        let header_class = if chunk.show_header { "" } else { " .sr-only" };
        let content = transform_headings(&chunk.content, &mut chunk_headings);

        chunk_meta.add_headings(chunk_headings);
        chunks.push(chunk_meta);

        page_body.push_str(&format!(
            "{} {} {{#{}{}}} \n\n{}\n\n",
            "#".repeat(chunk.depth),
            chunk.title,
            chunk.slug,
            header_class,
            content
        ));
    });

    fm.insert("cri", Frontmatter::CRI(&cri));
    fm.insert("chunks", Frontmatter::Chunks(chunks));

    Ok(format!(
        r#"---
{}---

{}"#,
        serde_yaml_ng::to_string(&fm)?,
        page_body
    ))
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
// add custom ids to h3 headings (h2 headings are chunk headers with ids already, and we ignore lower level headings in the page toc)
fn transform_headings(content: &str, headings: &mut Vec<Heading>) -> String {
    let re = Regex::new(r"(?m)^### (.+)$").unwrap();
    let mut slugger = github_slugger::Slugger::default();

    re.replace_all(content, |caps: &regex::Captures| {
        let heading_title = &caps[1];
        // Here you would use github-slugger to generate the ID
        let id = slugger.slug(heading_title);
        headings.push(Heading {
            slug: id.clone(),
            title: heading_title.to_string(),
            level: 3,
        });
        format!("### {} {{#{}}}", heading_title, id)
    })
    .to_string()
}

#[derive(Error, Debug)]
enum RequestError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] ureq::Error),

    #[error("Failed to read response body: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Server returned an error: {status}")]
    ServerError { status: u16 },
}
