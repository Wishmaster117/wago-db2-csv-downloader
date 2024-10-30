#[derive(Debug, Clone)]
pub struct Build {
    version: String,
    build_number: u32,
}

impl Build {
    pub fn new(version: &str, build_number: u32) -> Self {
        Self {
            version: version.to_string(),
            build_number,
        }
    }

    pub fn format_full_version(&self) -> String {
        format!("{}.{}", self.version, self.build_number)
    }
}

impl std::fmt::Display for Build {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_full_version())
    }
}