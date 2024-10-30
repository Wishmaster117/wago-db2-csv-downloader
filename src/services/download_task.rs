use crate::entities::Build;

#[derive(Clone)]
pub struct DownloadTask {
    pub table: String,
    pub build: Build,
    pub locale: String,
}

impl DownloadTask {
    pub fn new(table: String, build: Build, locale: String) -> Self {
        Self { table, build, locale }
    }
}