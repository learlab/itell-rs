use std::collections::BTreeMap;
use std::io::Write;
use std::{
    env,
    fs::{self, OpenOptions},
};

use anyhow::Context;
use itell::cms::{serialize_page, PageData, VolumeData, get_embedding_slugs, perform_health_check, HealthCheckData, save_health_check_to_supabase};
use serde::Serialize;
use dotenv::dotenv;

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const DEFAULT_OUTPUT_DIR: &str = "output/textbook";

pub struct Config {
    pub volume_id: String,
    pub output_dir: String,
    pub embeddings_supabase_url: String, 
    pub embeddings_supabase_api_key: String,
    pub log_supabase_url: String,
    pub log_supabase_api_key: String,
}

impl Config {
    pub fn new(
        volume_id: String, 
        output_dir: &str, 
        embeddings_supabase_url: String, 
        embeddings_supabase_api_key: String,
        log_supabase_url: String,
        log_supabase_api_key: String,
    ) -> Self {
        Self {
            volume_id,
            output_dir: output_dir.to_string(),
            embeddings_supabase_url,
            embeddings_supabase_api_key,
            log_supabase_url,
            log_supabase_api_key,
        }
    }
}


fn parse_config(mut args: impl Iterator<Item = String>) -> anyhow::Result<Config> {
    let volume_id = args.next().context("volume_id is required, search for the 'documentId` field at https://itell-strapi-um5h.onrender.com/api/texts/")?;
    let output_dir = args.next().unwrap_or(DEFAULT_OUTPUT_DIR.to_string());
    
    let _ = dotenv();

    // Get iTELL AI Supabase configuration from environment variables
    let embeddings_supabase_url = env::var("EMBEDDINGS_SUPABASE_URL")
        .context("EMBEDDINGS_SUPABASE_URL environment variable is required. Please set it in your .env file.")?;
    let embeddings_supabase_api_key = env::var("EMBEDDINGS_SUPABASE_API_KEY")
        .context("EMBEDDINGS_SUPABASE_API_KEY environment variable is required. Please set it in your .env file.")?;
    
    // Get Log Supabase configuration from environment variables
    let log_supabase_url = env::var("LOG_SUPABASE_URL")
        .context("LOG_SUPABASE_URL environment variable is required. Please set it in your .env file.")?;
    let log_supabase_api_key = env::var("LOG_SUPABASE_API_KEY")
        .context("LOG_SUPABASE_API_KEY environment variable is required. Please set it in your .env file.")?;

    Ok(Config::new(volume_id, &output_dir, embeddings_supabase_url, embeddings_supabase_api_key, log_supabase_url, log_supabase_api_key))

}

fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1);

    let config = match parse_config(args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Usage: cargo run <volume_id> [output_dir]");
            return Err(e);
        }
    };

    let volume =
        itell::cms::get_volume_data(&config.volume_id).context(format!("failed to fetch volume data with id {}, make sure you provide the correct `documentId` found at https://itell-strapi-um5h.onrender.com/api/texts/", config.volume_id.as_str()))?;
    let pages = itell::cms::collect_pages(&volume).context("failed to collect pages")?;

    create_output_dir(&config.output_dir).context("failed to create output directory")?;

    let volume_str = create_volume_metadata(&volume, &config.output_dir)
        .context("failed to create volume metadata")?;

    let mut sorted_pages: Vec<&PageData> = pages.iter().collect();
    sorted_pages.sort_by_key(|page| page.order);

    for (idx, page) in sorted_pages.iter().enumerate() {
        let next_slug = if idx == sorted_pages.len() - 1 {
            None
        } else {
            Some(sorted_pages[idx + 1].slug.as_str())
        };
        if let Err(e) = create_page(page, &config.output_dir, next_slug) {
            eprintln!("Error writing page {}: {}", page.slug, e);
            return Err(e);
        }
    }

    println!("Fetched volume metadata\n");
    println!("---");
    println!("{}", volume_str);
    println!("---\n");

    println!(
        "created {BOLD}{}{RESET} pages in {BOLD}{}{RESET}",
        pages.len(),
        &config.output_dir
    );

    // Start health check
    let embedding_slugs_array = get_embedding_slugs(&config.embeddings_supabase_url, &config.embeddings_supabase_api_key, &volume.slug.as_str())
        .context("Failed to get embedding slugs")?;

    let health_check = perform_health_check(
        &config.volume_id,
        &volume.slug,
        &volume.title,
        &pages,
        &embedding_slugs_array,
    ).context("Failed to perform health check")?;

    save_health_check_to_supabase(
        &health_check,
        &config.log_supabase_url,
        &config.log_supabase_api_key,
    ).context("Failed to save health check to Supabase")?;

    print_health_check_summary(&health_check);

    Ok(())
}

fn create_volume_metadata(volume: &VolumeData, output_dir: &str) -> anyhow::Result<String> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(format!("{}/volume.yaml", output_dir))
        .context("failed to open file for volume.toml")?;

    let mut map = BTreeMap::<&str, VolumeFrontmatter>::new();
    map.insert("title", VolumeFrontmatter::Title(volume.title.as_str()));
    map.insert("slug", VolumeFrontmatter::Slug(volume.slug.as_str()));
    map.insert(
        "description",
        VolumeFrontmatter::Description(volume.description.as_str()),
    );
    map.insert(
        "free_pages",
        VolumeFrontmatter::FreePages(volume.free_pages.as_slice()),
    );

    map.insert(
        "summary",
        VolumeFrontmatter::Summary(volume.summary.as_deref()),
    );

    let content = serde_yaml_ng::to_string(&map).context("failed to serialize volume metadata")?;
    write!(file, "{}", content).context("failed to write volume metadata")?;

    Ok(content)
}

fn create_page(page: &PageData, output_dir: &str, next_slug: Option<&str>) -> anyhow::Result<()> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(format!("{}/{}.md", output_dir, page.slug))
        .context(format!("failed to open file for {}", page.slug))?;

    let content = serialize_page(page, next_slug).context("failed to serialize page")?;
    write!(file, "{}", content).context("failed to write page")?;

    Ok(())
}

fn create_output_dir(output_dir: &str) -> anyhow::Result<()> {
    if fs::metadata(output_dir).is_ok() {
        fs::remove_dir_all(output_dir)?;
    }

    fs::create_dir_all(output_dir)?;
    Ok(())
}

fn print_health_check_summary(health_check: &HealthCheckData) {
    println!("--------------------");
    println!("\n{BOLD}HEALTH CHECK SUMMARY:{RESET}");
    println!("--------------------");
    println!("Volume: {} (Slug: {})", health_check.volume_title, health_check.volume_slug);
    println!("Total chunks: {}", health_check.total_chunks);
    println!("✓ Existing in Supabase: {BOLD}{}{RESET}", health_check.existing_chunks_count);
    
    if health_check.missing_chunks_count == 0 {
        println!("✓ All chunks found in Supabase! 🎉");
    } else {
        println!("✗ Missing from Supabase: {BOLD}{}{RESET}", health_check.missing_chunks_count);
        println!("\nMissing chunks by page:");
        for page in &health_check.pages {
            if !page.missing_chunks.is_empty() {
                println!("  Page '{}': {} missing", page.page_title, page.missing_chunks.len());
                for chunk in &page.missing_chunks {
                    println!("    - {}", chunk);
                }
            }
        }
        println!("💡Tip: The chunk slug might exist in the embeddings table but under a different volume slug. Check the Supabase database to verify.");
    }
    
    println!("\n✓ Health check data saved to Supabase log table");
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum VolumeFrontmatter<'a> {
    Title(&'a str),
    Slug(&'a str),
    Description(&'a str),
    FreePages(&'a [String]),
    Summary(Option<&'a str>),
}
