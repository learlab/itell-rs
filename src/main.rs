use std::fs::{self, OpenOptions};
use std::io::Write;

#[derive(Debug)]
struct PageData {
    title: String,
    slug: String,
    order: usize,
    summary: bool,
    chunks: Vec<ChunkData>,
}

#[derive(Debug)]
struct ChunkData {
    title: String,
    slug: String,
    depth: usize,
    text: String,
    show_header: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text_id = 10;
    let resp: serde_json::Value = reqwest::get(format!(
        "https://itell-strapi-um5h.onrender.com/api/texts/{}?populate%5BPages%5D%5Bpopulate%5D%5BContent%5D=true",
        text_id,
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

    let clean_pages = pages
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let attributes = page.get("attributes").unwrap();

            let id = page.get("id").unwrap().as_i64().unwrap() as i32;
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
            let summary = attributes.get("HasSummary").unwrap().as_bool().unwrap();
            let content = attributes.get("Content").unwrap().as_array().unwrap();

            let chunks = content
                .iter()
                .map(|chunk| {
                    let id = chunk.get("id").unwrap().as_i64().unwrap() as i32;
                    let slug = chunk.get("Slug").unwrap().as_str().unwrap().to_string();
                    let text = chunk.get("MDX").unwrap().as_str().unwrap().to_string();
                    let title = chunk.get("Header").unwrap().as_str().unwrap().to_string();
                    let show_header = chunk.get("ShowHeader").unwrap().as_bool().unwrap();

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
                    }
                })
                .collect::<Vec<ChunkData>>();

            PageData {
                title,
                slug,
                summary,
                chunks,
                order: index + 1, // chunks: content.split(" ").map(|s| s.to_string()).collect(),
            }
        })
        .collect::<Vec<PageData>>();

    let output_dir = "output";
    if fs::metadata(output_dir).is_ok() {
        fs::remove_dir_all(output_dir).unwrap();
    }

    fs::create_dir(output_dir).unwrap();

    for page in clean_pages {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(format!("{}/{}.md", output_dir, page.slug))
            .unwrap();

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
title: {}
slug: {}
summary: {}
order: {}
---

{}"#,
            page.title, page.slug, page.summary, page.order, content_string
        )
        .unwrap();
    }

    Ok(())
}
