use std::collections::BTreeMap;
use std::io::Write;
use std::{
    env,
    fs::{self, OpenOptions},
    process,
};

use anyhow::Context;
use itell::cms::{
    get_embedding_slugs, perform_health_check, serialize_page, HealthCheckData, PageData,
    VolumeData,
};
use serde::Serialize;

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const DEFAULT_OUTPUT_DIR: &str = "output/textbook";

pub struct Config {
    pub volume_id: String,
    pub output_dir: String,
    pub embeddings_supabase_url: Option<String>,
    pub embeddings_supabase_api_key: Option<String>,
}

impl Config {
    pub fn new(
        volume_id: String,
        output_dir: &str,
        embeddings_supabase_url: Option<String>,
        embeddings_supabase_api_key: Option<String>,
    ) -> Self {
        Self {
            volume_id,
            output_dir: output_dir.to_string(),
            embeddings_supabase_url,
            embeddings_supabase_api_key,
        }
    }
}

fn parse_config(mut args: impl Iterator<Item = String>) -> anyhow::Result<Config> {
    let volume_id = args.next().context("volume_id is required, search for the 'documentId` field at https://itell-strapi-um5h.onrender.com/api/texts/")?;
    let output_dir = args.next().unwrap_or(DEFAULT_OUTPUT_DIR.to_string());


    // Get iTELL AI Supabase configuration from environment variables (optional)
    let embeddings_supabase_url = env::var("EMBEDDINGS_SUPABASE_URL").ok();
    let embeddings_supabase_api_key = env::var("EMBEDDINGS_SUPABASE_API_KEY").ok();

    Ok(Config::new(
        volume_id,
        &output_dir,
        embeddings_supabase_url,
        embeddings_supabase_api_key,
    ))
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
        itell::cms::get_volume_data(&config.volume_id).context(format!(
            "failed to fetch volume data with id {}, make sure you provide the correct `documentId` found at https://itell-strapi-um5h.onrender.com/api/texts/",
            config.volume_id.as_str()
        ))?;
    let pages = itell::cms::collect_pages(&volume).context("failed to collect pages")?;

    create_output_dir(&config.output_dir).context("failed to create output directory")?;

    // Create volume metadata
    let volume_str = create_volume_metadata(&volume, &config.output_dir)
        .context("failed to create volume metadata")?;

    // Sort and create pages
    let mut sorted_pages: Vec<&PageData> = pages.iter().collect();
    sorted_pages.sort_by_key(|page| page.order);

    for (idx, page) in sorted_pages.iter().enumerate() {
        let next_slug = if idx == sorted_pages.len() - 1 {
            None
        } else {
            Some(sorted_pages[idx + 1].slug.as_str())
        };
        if let Err(e) = create_page(page, &config.output_dir, next_slug) {
            eprintln!("{}Error writing page {}: {}{}", RED, page.slug, e, RESET);
            process::exit(1);
        }
    }

    println!("Volume: {} ({})", volume.title, volume.slug);
    println!("Created {} pages in {}", pages.len(), &config.output_dir);
    println!();

    // Perform health check only if Supabase credentials are provided
    match (&config.embeddings_supabase_url, &config.embeddings_supabase_api_key) {
        (Some(url), Some(api_key)) => {
            println!("{}🔍 Starting vector validation...{}", YELLOW, RESET);
            let embedding_slugs_array = get_embedding_slugs(
                url,
                api_key,
                &volume.slug.as_str(),
            )
            .context("Failed to get embedding slugs")?;

            let health_check = perform_health_check(
                &config.volume_id,
                &volume.slug,
                &volume.title,
                &pages,
                &embedding_slugs_array,
            )
            .context("Failed to perform health check")?;

            // Print health check results and determine exit code
            let validation_passed = print_health_check_summary(&health_check);

            // Exit with appropriate code for GitHub Actions
            if validation_passed {
                println!("{}✅ Vector validation passed!{}", GREEN, RESET);
                process::exit(0);
            } else {
                println!("{}❌ Vector validation failed!{}", RED, RESET);
                process::exit(1);
            }
        }
        _ => {
            println!("{}⚠️  Skipping vector validation (Supabase credentials not provided){}", YELLOW, RESET);
            println!("{}✅ Content fetched successfully{}", GREEN, RESET);
            process::exit(0);
        }
    }
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

fn print_health_check_summary(health_check: &HealthCheckData) -> bool {
    println!("--------------------");
    println!("\n{BOLD}HEALTH CHECK SUMMARY:{RESET}");
    println!("--------------------");
    println!(
        "Volume: {} (Slug: {})",
        health_check.volume_title, health_check.volume_slug
    );
    println!("Total chunks: {}", health_check.total_chunks);
    println!(
        "✓ Existing in Supabase: {}{}{}",
        BOLD, health_check.existing_chunks_count, RESET
    );

    let validation_passed = health_check.missing_chunks_count == 0;

    if validation_passed {
        println!("✓ All chunks found in Supabase! 🎉");
    } else {
        println!(
            "✗ Missing from Supabase: {BOLD}{}{RESET}",
            health_check.missing_chunks_count
        );
        println!("\nMissing chunks by page:");
        for page in &health_check.pages {
            if !page.missing_chunks.is_empty() {
                println!(
                    "  Page '{}': {} missing",
                    page.page_title,
                    page.missing_chunks.len()
                );
                for chunk in &page.missing_chunks {
                    println!("    - {}", chunk);
                }
            }
        }
        println!("💡Tip: Verify that all pages have been successfully published. If there was an issue, try publishing the page with missing chunks again.");
    }

    println!();
    validation_passed
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
