use std::fs;
use std::path::Path;
use anyhow::Result;
use reqwest::Client;
use tokio::time::Duration;
use crate::utils::RateLimiter;
use crate::entities::Build;
use std::collections::HashSet;
use crate::utils::{file_exists_with_size, ensure_dir_exists};
use tokio::sync::Semaphore;
use std::sync::Arc;
use indicatif::{ProgressBar, ProgressStyle};
use futures::future::join_all; 

pub struct DownloadService {
    client: Client,
    base_url: String,
    rate_limiter: RateLimiter,
    max_retries: u32,
    retry_delay_secs: u64,
    max_concurrent_downloads: usize,
}

impl DownloadService {
    pub fn new(base_url: String) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            base_url,
            rate_limiter: RateLimiter::new(100),
            max_retries: 3,
            retry_delay_secs: 5,
            max_concurrent_downloads: 4,
        })
    }

    pub fn set_rate_limit(&mut self, requests_per_minute: u32) {
        self.rate_limiter = RateLimiter::new(requests_per_minute);
    }

    pub fn set_retry_params(&mut self, max_retries: u32, retry_delay_secs: u64) {
        self.max_retries = max_retries;
        self.retry_delay_secs = retry_delay_secs;
    }

    pub fn set_concurrent_downloads(&mut self, count: usize) {
        self.max_concurrent_downloads = count;
    }

    async fn download_csv(
        &mut self,
        table: &str,
        build: &Build,
        locale: &str,
    ) -> Result<()> {
        self.rate_limiter.wait().await;
    
        let url = format!(
            "{}/{}/csv?build={}&locale={}",
            self.base_url, table, build.format_full_version(), locale
        );
    
        let folder_path = Path::new(&build.format_full_version()).join(locale);
        let file_path = folder_path.join(format!("{}.csv", table));
    
        if file_exists_with_size(&file_path) {
            println!("Skipping an existing file: {}", file_path.display());
            return Ok(());
        }
    
        println!("Downloading: {}", url);
    
        ensure_dir_exists(&folder_path)?;
        
        let response = self.client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?;
        
        if response.status().is_success() {
            let content = response.bytes().await?;
            fs::write(&file_path, content)?;
            println!("✓ Downloaded {}", file_path.display());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Download error: {}: {}", table, response.status()))
        }
    }

    async fn download_with_retry(
        &mut self,
        table: &str,
        build: &Build,
        locale: &str,
    ) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        let mut retries = 0;

        while retries < MAX_RETRIES {
            match self.download_csv(table, build, locale).await {
                Ok(_) => return Ok(()),
                Err(e) if e.to_string().contains("timeout") => {
                    retries += 1;
                    println!("Attempt {} of {} for table {}", retries, MAX_RETRIES, table);
                    println!("Wait 5 seconds before trying again....");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                },
                Err(e) => return Err(e),
            }
        }
        
        Err(anyhow::anyhow!("The number of attempts for the table has been exceeded {}", table))
    }

    pub async fn download_all(
        &self,
        tables: &HashSet<String>,
        builds: &[Build],
        locales: &[String]
    ) -> Result<()> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_downloads));
        let client = self.client.clone();
        let rate_limiter = self.rate_limiter.clone();
    
        // Создаем прогресс-бар
        let total_files = tables.len() * builds.len() * locales.len();
        let progress = ProgressBar::new(total_files as u64);
        progress.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%) {msg}")?);
    
        let mut handles = Vec::new();
        
        for build in builds {
            for locale in locales {
                for table in tables {
                    let table = table.clone();
                    let build = build.clone();
                    let locale = locale.clone();
                    let semaphore = Arc::clone(&semaphore);
                    let client = client.clone();
                    let rate_limiter = rate_limiter.clone();
                    let progress = progress.clone();
                    let base_url = self.base_url.clone();
    
                    let handle = tokio::spawn(async move {
                        let _permit = semaphore.acquire().await.unwrap();
                        
                        let folder_path = Path::new(&build.format_full_version()).join(&locale);
                        let file_path = folder_path.join(format!("{}.csv", table));
    
                        if file_exists_with_size(&file_path) {
                            progress.inc(1);
                            progress.set_message(format!("Skipped: {}", file_path.display()));
                            return Ok(());
                        }
    
                        ensure_dir_exists(&folder_path)?;

                        let mut rate_limiter = rate_limiter.clone(); 

                        rate_limiter.wait().await;
    
                        let url = format!(
                            "{}/{}/csv?build={}&locale={}",
                            base_url, table, build.format_full_version(), locale
                        );
    
                        let response = client.get(&url)
                            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                            .send()
                            .await?;
    
                        if response.status().is_success() {
                            let content = response.bytes().await?;
                            fs::write(&file_path, content)?;
                            progress.inc(1);
                            progress.set_message(format!("Downloaded: {}", file_path.display()));
                            Ok(())
                        } else {
                            Err(anyhow::anyhow!("Download error: {}: {}", table, response.status()))
                        }
                    });
    
                    handles.push(handle);
                }
            }
        }
    
        let results = join_all(handles).await;
        
        let mut had_errors = false;
        for result in results {
            if let Err(e) = result {
                eprintln!("Task error: {}", e);
                had_errors = true;
            }
        }
    
        progress.finish_with_message("Download complete");
    
        if had_errors {
            Err(anyhow::anyhow!("Some downloads failed"))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Mock};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_build() -> Build {
        Build::new("11.0.5".to_string(), 57212)
    }

    fn create_mock_response(status: usize, body: &str) -> Mock {
        mock("GET", "/Achievement/csv")
            .match_query(mockito::Matcher::Any)
            .with_status(status)
            .with_header("content-type", "text/csv")
            .with_body(body)
            .create()
    }

    #[tokio::test]
    async fn test_service_creation() {
        let service = DownloadService::new("https://wago.tools/db2".to_string());
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_successful_download() {
        let mock_server = mockito::Server::new();
        let _m = create_mock_response(200, "id,name\n1,Test");
        
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let mut service = DownloadService::new(mock_server.url()).unwrap();
        let build = create_test_build();
        
        let result = service.download_csv("Achievement", &build, "ruRU").await;
        
        std::env::set_current_dir(original_dir).unwrap();
        assert!(result.is_ok());
        
        let file_path = temp_dir
            .path()
            .join("11.0.5.57212")
            .join("ruRU")
            .join("Achievement.csv");
        assert!(file_path.exists());
        
        let content = fs::read_to_string(file_path).unwrap();
        assert_eq!(content, "id,name\n1,Test");
    }

    #[tokio::test]
    async fn test_failed_download() {
        let mock_server = mockito::Server::new();
        let _m = create_mock_response(404, "Not Found");
        
        let mut service = DownloadService::new(mock_server.url()).unwrap();
        let build = create_test_build();
        
        let result = service.download_csv("Achievement", &build, "ruRU").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("404"));
    }

    #[tokio::test]
    async fn test_retry_on_timeout() {
        let mock_server = mockito::Server::new();
        
        let _m1 = mock("GET", "/Achievement/csv")
            .with_status(408)
            .create();
            
        let _m2 = create_mock_response(200, "id,name\n1,Test");
        
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let mut service = DownloadService::new(mock_server.url()).unwrap();
        let build = create_test_build();
        
        let result = service.download_with_retry("Achievement", &build, "ruRU").await;
        
        std::env::set_current_dir(original_dir).unwrap();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mock_server = mockito::Server::new();
        let _m = create_mock_response(200, "id,name\n1,Test");
        
        let mut service = DownloadService::new(mock_server.url()).unwrap();
        let build = create_test_build();
        
        let start = std::time::Instant::now();
        
        for _ in 0..3 {
            let _ = service.download_csv("Achievement", &build, "ruRU").await;
        }
        
        let duration = start.elapsed();
        
        assert!(duration.as_secs() > 0, "Requests should be limited in frequency");
    }

    #[tokio::test]
    async fn test_download_all() {
        let mock_server = mockito::Server::new();
        let _m = create_mock_response(200, "id,name\n1,Test");
        
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let mut service = DownloadService::new(mock_server.url()).unwrap();
        
        let tables = vec!["Achievement".to_string()];
        let builds = vec![create_test_build()];
        let locales = vec!["ruRU".to_string()];
        
        let result = service.download_all(&tables, &builds, &locales).await;
        
        std::env::set_current_dir(original_dir).unwrap();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_skip_existing_file() {
        let mock_server = mockito::Server::new();
        let _m = create_mock_response(200, "id,name\n1,Test");
        
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let folder_path = temp_dir.path().join("11.0.5.57212").join("ruRU");
        fs::create_dir_all(&folder_path).unwrap();
        let file_path = folder_path.join("Achievement.csv");
        fs::write(&file_path, "existing content").unwrap();

        let mut service = DownloadService::new(mock_server.url()).unwrap();
        let build = create_test_build();
        
        let result = service.download_csv("Achievement", &build, "ruRU").await;
        
        std::env::set_current_dir(original_dir).unwrap();
        assert!(result.is_ok());
        
        let content = fs::read_to_string(file_path).unwrap();
        assert_eq!(content, "existing content");
    }

    #[tokio::test]
    async fn test_parallel_downloads() {
        let mock_server = mockito::Server::new();
        
        let _m1 = mock("GET", "/Achievement/csv")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body("data1")
            .create();
            
        let _m2 = mock("GET", "/Achievement_Category/csv")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body("data2")
            .create();

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let mut service = DownloadService::new(mock_server.url()).unwrap();
        service.set_concurrent_downloads(2); 
        
        let tables = HashSet::from([
            "Achievement".to_string(), 
            "Achievement_Category".to_string()
        ]);
        let builds = vec![create_test_build()];
        let locales = vec!["ruRU".to_string()];
        
        let start = std::time::Instant::now();
        let result = service.download_all(&tables, &builds, &locales).await;
        let duration = start.elapsed();
        
        std::env::set_current_dir(original_dir).unwrap();
        
        assert!(result.is_ok());
        assert!(temp_dir.path()
            .join("11.0.5.57212")
            .join("ruRU")
            .join("Achievement.csv")
            .exists());
        assert!(temp_dir.path()
            .join("11.0.5.57212")
            .join("ruRU")
            .join("Achievement_Category.csv")
            .exists());
        assert!(duration.as_secs() < 2, "Parallel downloads should be faster");
    }

    #[tokio::test]
    async fn test_progress_bar() {
        let mock_server = mockito::Server::new();
        let _m = create_mock_response(200, "test data");
        
        let mut service = DownloadService::new(mock_server.url()).unwrap();
        
        let tables = HashSet::from(["Achievement".to_string()]);
        let builds = vec![create_test_build()];
        let locales = vec!["ruRU".to_string()];
        
        let result = service.download_all(&tables, &builds, &locales).await;
        assert!(result.is_ok());
    }
}