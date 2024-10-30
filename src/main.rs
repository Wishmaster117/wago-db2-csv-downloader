mod config;
mod data;
mod entities;
mod handlers;
mod services;
mod utils;

use anyhow::Result;
use dialoguer::Confirm;
use services::downloader::DownloadService;

async fn run() -> Result<()> {
    println!("wago.tools DB2 csv exporter by notwonderful");

    let config = config::AppConfig::new();
    
    let available_builds = data::builds::AVAILABLE_BUILDS;
    let available_locales = data::locales::AVAILABLE_LOCALES;
    let tables = data::tables::get_available_tables();

    let selected_builds = handlers::build::handle_build_selection(available_builds)?;
    if selected_builds.is_empty() {
        return Ok(());
    }

    let selected_locales = handlers::locale::handle_locale_selection(available_locales)?;
    if selected_locales.is_empty() {
        return Ok(());
    }

    println!("\n📥 Let's start downloading:");
    println!("Builds: {}", selected_builds.iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(", "));
    println!("Locales: {}", selected_locales.join(", "));
    println!("Total Tables: {}", tables.len());

    if Confirm::new()
        .with_prompt("Start downloading?")
        .interact()? 
    {
        let mut downloader = DownloadService::new(config.base_url)?;
        downloader.set_rate_limit(config.requests_per_minute);
        downloader.set_retry_params(config.max_retries, config.retry_delay_secs);
        
        downloader.download_all(&tables, &selected_builds, &selected_locales).await?;
        println!("Download completed!");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}