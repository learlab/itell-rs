use serde::{Deserialize, Serialize};
use super::page::PageData;
use std::collections::HashSet;

use anyhow::{Context, Result};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckData {
    pub volume_id: String,
    pub volume_slug: String,
    pub volume_title: String,
    pub total_chunks: usize,
    pub existing_chunks_count: usize,
    pub missing_chunks_count: usize,
    pub pages: Vec<PageHealthCheck>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageHealthCheck {
    pub page_slug: String,
    pub page_title: String,
    pub existing_chunks: Vec<String>,
    pub missing_chunks: Vec<String>,
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] ureq::Error),

    #[error("Failed to read response body: {0}")]
    IO(#[from] std::io::Error),

    #[error("Supabase returned an error: {status}")]
    SupabaseError { status: u16 },
}

// Fetches embedding slugs for volume from Supabase
pub fn get_embedding_slugs(supabase_url: &str, api_key: &str, volume_slug: &str) -> Result<Vec<String>> {
    let url = format!("{}/rest/v1/embeddings?select=chunk,text&text=eq.{}", supabase_url, volume_slug);
    
    let response = ureq::get(&url)
        .set("apikey", api_key)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => RequestError::SupabaseError { status: code },
            other => RequestError::Http(other),
        })
        .context("Connecting to Supabase API")?;

    let body: Value = response.into_json().context("Response body is not JSON")?;
    
    let embedding_slugs = body
        .as_array()
        .context("Supabase response cannot be converted into an array")?
        .iter()
        .filter_map(|item| {
            item.get("chunk")
                .and_then(|chunk| chunk.as_str())
                .map(|s| s.to_string())
        })
        .collect();

    Ok(embedding_slugs)
}

pub fn perform_health_check(
    volume_id: &str,
    volume_slug: &str,
    volume_title: &str,
    pages: &[PageData],
    embedding_slugs: &[String]
) -> Result<HealthCheckData> {
    let embedding_slugs_set: HashSet<&String> = embedding_slugs.iter().collect();

    let mut page_health_checks = Vec::new();
    let mut total_existing = 0;
    let mut total_missing = 0;
    
    for page in pages {
        let mut existing_chunks = Vec::new();
        let mut missing_chunks = Vec::new();
        
        for chunk in &page.chunks {
            if embedding_slugs_set.contains(&chunk.slug) {
                existing_chunks.push(chunk.slug.clone());
                total_existing += 1;
            } else {
                missing_chunks.push(chunk.slug.clone());
                total_missing += 1;
            }
        }
        
        page_health_checks.push(PageHealthCheck {
            page_slug: page.slug.clone(),
            page_title: page.title.clone(),
            existing_chunks,
            missing_chunks,
        });
    }
    
    Ok(HealthCheckData {
        volume_id: volume_id.to_string(),
        volume_slug: volume_slug.to_string(),
        volume_title: volume_title.to_string(),
        total_chunks: total_existing + total_missing,
        existing_chunks_count: total_existing,
        missing_chunks_count: total_missing,
        pages: page_health_checks,
    })
}

/// Saves health check data to Supabase log table
pub fn save_health_check_to_supabase(
    health_check: &HealthCheckData,
    log_supabase_url: &str,
    log_api_key: &str,
) -> Result<()> {
    let url = format!("{}/rest/v1/log_rs", log_supabase_url);
    
    let payload = json!({
        "data": health_check
    });
    
    let response = ureq::post(&url)
        .set("apikey", log_api_key)
        .set("Authorization", &format!("Bearer {}", log_api_key))
        .set("Content-Type", "application/json")
        .send_json(&payload)
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => RequestError::SupabaseError { status: code },
            other => RequestError::Http(other),
        })
        .context("Failed to save health check to Supabase log table")?;

    if response.status() >= 200 && response.status() < 300 {
        println!("✓ Health check data saved to Supabase log table");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to save to Supabase: HTTP {}", response.status()))
    }
}

