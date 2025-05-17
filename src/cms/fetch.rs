use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;
use std::{collections::BTreeMap, str::FromStr};
use thiserror::Error;

use super::{
    chunk::{ChunkData, ChunkType, CriItem},
    frontmatter::{ChunkMeta, Frontmatter, Heading},
    page::{PageData, PageParent, QuizAnswerItem, QuizItem},
};

const BASE_URL: &str = "https://itell-strapi-um5h.onrender.com/api/texts/";
const QUERY: &str = "?populate%5BPages%5D%5Bfields%5D%5B0%5D=%2A&populate%5BPages%5D%5Bsort%5D=createdAt&populate%5BPages%5D%5Bpopulate%5D%5BContent%5D=true&populate%5BPages%5D%5Bpopulate%5D%5BChapter%5D%5Bfields%5D%5B0%5D=Title&populate%5BPages%5D%5Bpopulate%5D%5BChapter%5D%5Bfields%5D%5B1%5D=Slug&populate%5BPages%5D%5Bpopulate%5D%5BQuiz%5D%5Bpopulate%5D%5BQuestions%5D%5Bpopulate%5D=%2A";

pub struct VolumeData {
    pub title: String,
    pub description: String,
    pub slug: String,
    pub free_pages: Vec<String>,
    pub summary: Option<String>,
    pages: Vec<serde_json::Value>,
}

#[derive(Error, Debug)]
enum RequestError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] ureq::Error),

    #[error("Failed to read response body: {0}")]
    IO(#[from] std::io::Error),

    #[error("Server returned an error: {status}")]
    StrapiServer { status: u16 },
}

/// Extracts a typed attribute from a JSON value
fn get_attribute<T>(value: &Value, attribute: &str) -> Option<T>
where
    T: FromStr,
{
    value.get(attribute).and_then(|v| match v {
        Value::String(s) => T::from_str(s).ok(),
        Value::Number(n) => n.as_f64().and_then(|f| T::from_str(&f.to_string()).ok()),
        Value::Bool(b) => T::from_str(&b.to_string()).ok(),
        _ => None,
    })
}

/// Fetches volume data from the API
pub fn get_volume_data(volume_id: &str) -> Result<VolumeData> {
    let url = format!("{}{}{}", BASE_URL, volume_id, QUERY);
    let response = ureq::get(&url)
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => RequestError::StrapiServer { status: code },
            other => RequestError::Http(other),
        })
        .context("Connecting to Strapi API")?;

    let body: Value = response.into_json().context("Response body is not JSON")?;
    let data = body.get("data").context("No data in volume response")?;

    let free_pages = get_attribute::<String>(data, "FreePages")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let pages = data
        .get("Pages")
        .and_then(|p| p.as_array())
        .context("No pages in volume response")?
        .to_owned();

    Ok(VolumeData {
        title: get_attribute(data, "Title").context("Volume must set title")?,
        description: get_attribute(data, "Description").context("Volume must set description")?,
        summary: get_attribute(data, "VolumeSummary"),
        slug: get_attribute(data, "Slug").context("Volume must set slug")?,
        pages,
        free_pages,
    })
}

/// Transforms markdown content by adding IDs to H3 headings
fn transform_headings(content: &str, headings: &mut Vec<Heading>) -> String {
    let heading_regex = Regex::new(r"(?m)^### (.+)$").unwrap();
    let mut slugger = github_slugger::Slugger::default();

    heading_regex
        .replace_all(content, |caps: &regex::Captures| {
            let heading_title = &caps[1];
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

/// Parses a question-answer pair from a chunk
fn parse_cri(chunk: &Value, chunk_slug: &str) -> Option<CriItem> {
    let question = get_attribute::<String>(chunk, "Question")?;
    let answer = get_attribute::<String>(chunk, "ConstructedResponse")?;

    Some(CriItem {
        slug: chunk_slug.to_string(),
        question,
        answer,
    })
}

/// Parses a video chunk
fn parse_video(chunk: &Value, page_title: &str) -> Result<ChunkData> {
    let title = get_attribute::<String>(chunk, "Header").context(format!(
        "Video chunk in page '{}' must set Header",
        page_title
    ))?;

    let video_url = get_attribute::<String>(chunk, "URL")
        .context(format!("Video chunk in page '{}' must set URL", page_title))?;

    let chunk_slug = get_attribute::<String>(chunk, "Slug").context(format!(
        "Video chunk '{}' in page '{}' must set Slug",
        &title, page_title
    ))?;

    let cri = parse_cri(chunk, &chunk_slug);

    let video_id = video_url
        .split("v=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .unwrap_or_default();

    let description = get_attribute::<String>(chunk, "Description").unwrap_or_default();
    let content = format!(
        "{description}\n\n<i-youtube videoid=\"{}\" height={{400}} width=\"100%\" >\n\n</i-youtube>\n\n",
        video_id
    );

    Ok(ChunkData {
        title,
        slug: chunk_slug,
        depth: 2,
        content,
        cri,
        show_header: true,
        chunk_type: ChunkType::Video,
    })
}

/// Parses a regular content chunk
fn parse_regular_chunk(
    chunk: &Value,
    index: usize,
    page_title: &str,
    chunk_type: ChunkType,
) -> Result<ChunkData> {
    let chunk_title = get_attribute::<String>(chunk, "Header").context(format!(
        "Chunk '{}' in page '{}' must set Header",
        index, page_title
    ))?;

    let chunk_slug = get_attribute::<String>(chunk, "Slug").context(format!(
        "Chunk '{}' in page '{}' must set Slug",
        index, page_title
    ))?;

    let content = get_attribute::<String>(chunk, "MD").context(format!(
        "Chunk '{}' in page '{}' must set MD",
        index, page_title
    ))?;

    let show_header = get_attribute::<bool>(chunk, "ShowHeader").unwrap_or_default();
    let cri = parse_cri(chunk, &chunk_slug);

    let header_level = get_attribute::<String>(chunk, "HeaderLevel");
    let depth = match header_level.as_deref() {
        Some("h3") => 3,
        Some("H3") => 3,
        Some("h4") => 4,
        Some("H4") => 4,
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
}

/// Unified quiz parsing function for all quiz types
fn parse_quiz(page: &Value) -> Result<Option<Vec<QuizItem>>> {
    let questions = match page
        .get("Quiz")
        .and_then(|a| a.get("Questions"))
        .and_then(|q| q.as_array())
    {
        Some(questions) => questions,
        None => return Ok(None),
    };

    let quiz_items = questions
        .iter()
        .map(|q| parse_quiz_item(q))
        .collect::<Result<Vec<_>>>()?;

    if quiz_items.is_empty() {
        Ok(None)
    } else {
        Ok(Some(quiz_items))
    }
}

/// Parse a single quiz item from different formats
fn parse_quiz_item(question: &Value) -> Result<QuizItem> {
    let id = get_attribute::<String>(question, "id").context("Quiz question has no id")?;

    // Check if it's a multiple-choice question
    if question.get("__component").and_then(|c| c.as_str())
        == Some("quizzes.multiple-choice-question")
    {
        parse_multiple_choice_question(question, &id)
    }
    // Check if it's a generated question (YAML format)
    else if let Some(text) = question.get("GeneratedQuestion").and_then(|q| q.as_str()) {
        parse_generated_question(text)
    } else {
        Err(anyhow::anyhow!(
            "Quiz item is missing a valid '__component' or 'GeneratedQuestion' field"
        ))
    }
}

/// Parse a multiple-choice question
fn parse_multiple_choice_question(question: &Value, id: &str) -> Result<QuizItem> {
    let question_text = get_attribute::<String>(question, "Question")
        .context(format!("Quiz question '{}' has no question", id))?;

    let answers = question
        .get("Answers")
        .and_then(|a| a.as_array())
        .context(format!("Quiz question '{}' has no answers", id))?;

    let quiz_answers = answers
        .iter()
        .map(|a| parse_quiz_answer(a, id))
        .collect::<Result<Vec<_>>>()?;

    Ok(QuizItem {
        question: question_text,
        answers: quiz_answers,
    })
}

/// Parse a quiz answer item
fn parse_quiz_answer(answer: &Value, question_id: &str) -> Result<QuizAnswerItem> {
    let answer_id = get_attribute::<String>(answer, "id").context(format!(
        "In quiz question '{}', one answer has no id",
        question_id
    ))?;

    let answer_text = get_attribute::<String>(answer, "Text").context(format!(
        "In quiz question '{}', answer '{}' has no text",
        question_id, answer_id
    ))?;

    let correct = get_attribute::<bool>(answer, "IsCorrect").context(format!(
        "In quiz question '{}', answer {} has no IsCorrect flag",
        question_id, answer_id
    ))?;

    Ok(QuizAnswerItem {
        answer: answer_text,
        correct,
    })
}

/// Parse a generated question in YAML format
fn parse_generated_question(yaml_text: &str) -> Result<QuizItem> {
    let quiz_items: Vec<QuizItem> =
        serde_yaml_ng::from_str(yaml_text).context("Quiz format is invalid")?;

    quiz_items
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Generated question YAML contains no items"))
}

/// Collects and processes pages from volume data
pub fn collect_pages(resp: &VolumeData) -> Result<Vec<PageData>> {
    resp.pages
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let title = get_attribute::<String>(page, "Title")
                .context(format!("Page '{}' must set title", index))?;

            let slug = get_attribute::<String>(page, "Slug")
                .context(format!("Page '{}' must set slug", &title))?;

            let mut assignments = match get_attribute::<bool>(page, "HasSummary")
                .context(format!("Page '{}' must set HasSummary", &title))?
            {
                true => vec![String::from("summary")],
                false => Vec::new(),
            };

            // Parse parent chapter
            let parent = page
                .get("Chapter")
                .filter(|c| !c.is_null())
                .map(|c| -> Result<PageParent> {
                    Ok(PageParent::new(
                        get_attribute(c, "Title")
                            .context(format!("Chapter for page '{}' must set title", &title))?,
                        get_attribute(c, "Slug")
                            .context(format!("Chapter for page '{}' must set slug", &title))?,
                    ))
                })
                .transpose()?;

            // Parse quiz
            let quiz =
                parse_quiz(page).context(format!("Failed to parse quiz for page '{}'", &title))?;

            if quiz.is_some() {
                assignments.push("quiz".to_string());
            }

            // Parse content chunks
            let default_content = Vec::new();
            let content = page
                .get("Content")
                .and_then(|v| v.as_array())
                .unwrap_or(&default_content);

            let chunks = content
                .iter()
                .enumerate()
                .map(|(index, chunk)| parse_chunk(chunk, index, &title))
                .collect::<Result<Vec<_>>>()?;

            let order = get_attribute::<usize>(page, "Order")
                .context(format!("Page '{}' must set Order", &title))?;

            Ok(PageData {
                title: title.to_string(),
                chunks,
                slug,
                parent,
                order,
                assignments,
                quiz,
            })
        })
        .collect()
}

/// Parse a content chunk based on its type
fn parse_chunk(chunk: &Value, index: usize, page_title: &str) -> Result<ChunkData> {
    // Determine chunk type
    let component = get_attribute::<String>(chunk, "__component");
    let chunk_type = component.map_or(ChunkType::Regular, |c| match c.as_str() {
        "page.plain-chunk" => ChunkType::Plain,
        "page.video" => ChunkType::Video,
        _ => ChunkType::Regular,
    });

    // Parse based on chunk type
    if matches!(chunk_type, ChunkType::Video) {
        parse_video(chunk, page_title)
    } else {
        parse_regular_chunk(chunk, index, page_title, chunk_type)
    }
}

/// Serializes a page to a string with YAML frontmatter
pub fn serialize_page(page: &PageData, next_slug: Option<&str>) -> Result<String> {
    let mut fm: BTreeMap<&str, Frontmatter> = BTreeMap::new();
    fm.insert("title", Frontmatter::Title(page.title.as_str()));
    fm.insert("slug", Frontmatter::Slug(page.slug.as_str()));
    fm.insert("next_slug", Frontmatter::NextSlug(next_slug));
    fm.insert("order", Frontmatter::Order(page.order));
    fm.insert("assignments", Frontmatter::Assignments(&page.assignments));
    fm.insert("parent", Frontmatter::Parent(page.parent.as_ref()));
    fm.insert("quiz", Frontmatter::Quiz(page.quiz.as_ref()));

    let mut cri = Vec::<&CriItem>::new();
    let mut chunks = Vec::<ChunkMeta>::new();
    let mut page_body = String::with_capacity(800 * page.chunks.len());

    // Process each chunk
    page.chunks.iter().for_each(|chunk| {
        let mut chunk_headings = Vec::new();
        let mut chunk_meta =
            ChunkMeta::new(chunk.title.as_str(), chunk.slug.as_str(), &chunk.chunk_type);

        // Add to CRI if present
        if let Some(ref item) = chunk.cri {
            cri.push(item);
        }

        // Generate page body
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
