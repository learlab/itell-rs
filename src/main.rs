use anyhow::Context;
use page::{clean_pages, get_pages_by_volume_id, write_page};
use std::{env, fs};
mod frontmatter;
mod page;

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
fn create_output_dir(output_dir: &str) -> anyhow::Result<()> {
    if fs::metadata(output_dir).is_ok() {
        fs::remove_dir_all(output_dir)?;
    }

    fs::create_dir(output_dir)?;
    Ok(())
}
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Usage: cargo run <volume_id> [output_dir]"));
    }

    let volume_id = args[1].parse::<i32>()?;
    let default_output_dir = String::from("output");
    let output_dir = args.get(2).map_or(default_output_dir, |s| s.to_owned());
    let pages = get_pages_by_volume_id(volume_id).context("failed to fetch volume")?;
    let pages_clean = clean_pages(pages);

    create_output_dir(&output_dir).context("failed to create output directory")?;

    for page in pages_clean.iter() {
        write_page(page, &output_dir)?;
    }
    println!(
        "created {BOLD}{}{RESET} pages in {BOLD}{}{RESET}",
        pages_clean.len(),
        &output_dir
    );

    Ok(())
}
