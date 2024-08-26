use std::io::Write;
use std::{
    env,
    fs::{self, OpenOptions},
};

use anyhow::Context;
use itell::cms::{serialize_page, PageData};

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const DEFAULT_OUTPUT_DIR: &str = "output";

pub struct Config {
    pub volume_id: i32,
    pub output_dir: String,
}

impl Config {
    pub fn new(volume_id: i32, output_dir: &str) -> Self {
        Self {
            volume_id,
            output_dir: output_dir.to_string(),
        }
    }
}

fn parse_config(mut args: impl Iterator<Item = String>) -> anyhow::Result<Config> {
    let volume_id: i32 = args.next().context("volume_id is required")?.parse()?;
    let output_dir = args.next().unwrap_or(DEFAULT_OUTPUT_DIR.to_string());

    Ok(Config::new(volume_id, &output_dir))
}

fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1);

    let config = match parse_config(args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Usage; cargo run <volume_id> [output_dir]");
            return Err(e);
        }
    };

    let volume =
        itell::cms::get_pages_by_volume_id(config.volume_id).context("failed to fetch volume")?;
    let pages = itell::cms::clean_pages(volume).context("failed to clean pages")?;

    create_output_dir(&config.output_dir).context("failed to create output directory")?;

    for page in pages.iter() {
        if let Err(e) = write_page(page, &config.output_dir) {
            eprintln!("Error writing page {}: {}", page.slug, e);
            return Err(e);
        }
    }

    println!(
        "created {BOLD}{}{RESET} pages in {BOLD}{}{RESET}",
        pages.len(),
        &config.output_dir
    );

    Ok(())
}

fn write_page(page: &PageData, output_dir: &str) -> anyhow::Result<()> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(format!("{}/{}.md", output_dir, page.slug))
        .context(format!("failed to open file for {}", page.slug))?;

    let content = serialize_page(page).context("failed to serialize page")?;
    write!(file, "{}", content).context("failed to write page")?;

    Ok(())
}

fn create_output_dir(output_dir: &str) -> anyhow::Result<()> {
    if fs::metadata(output_dir).is_ok() {
        fs::remove_dir_all(output_dir)?;
    }

    fs::create_dir(output_dir)?;
    Ok(())
}
