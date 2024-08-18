use std::fs;

use page::{clean_pages, get_pages_by_volume_id, write_page};
mod frontmatter;
mod page;

fn create_output_dir(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if fs::metadata(output_dir).is_ok() {
        fs::remove_dir_all(output_dir).unwrap();
    }

    fs::create_dir(output_dir).unwrap();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pages = get_pages_by_volume_id(10).await?;

    let pages_clean = clean_pages(pages);

    let output_dir = "output";
    create_output_dir(output_dir)?;

    for page in pages_clean {
        write_page(&page, output_dir)?;
    }

    Ok(())
}
