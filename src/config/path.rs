//Configuration of filepaths. User can specify environment variables for base path, readable_roots,
//and writable roots so the code can work out of the box.
use std::env;
use std::path::{Path, PathBuf};

pub struct PathConfig {
    pub base_path: PathBuf,
    pub readable_roots: Vec<PathBuf>,
    pub writable_roots: Vec<PathBuf>,
}

impl PathConfig {
    pub fn from_env() -> Self {
        let base_path = env::var("MCP_BASE_PATH").unwrap_or_else(|_| "/home/user".to_string());

        let readable_roots = parse_paths(
            &env::var("MCP_READABLE_ROOTS").unwrap_or_else(|_| format!("{}/readable", base_path)),
        );

        let writable_roots = parse_paths(
            &env::var("MCP_WRITABLE_ROOTS").unwrap_or_else(|_| format!("{}/writable", base_path)),
        );

        Self {
            base_path: PathBuf::from(base_path),
            readable_roots,
            writable_roots,
        }
    }

    pub fn is_readable(&self, path: &Path) -> Result<bool, String> {
        self.is_allowed(path, &self.readable_roots)
    }

    pub fn is_writable(&self, path: &Path) -> Result<bool, String> {
        self.is_allowed(path, &self.writable_roots)
    }

    fn is_allowed(&self, path: &Path, roots: &Vec<PathBuf>) -> Result<bool, String> {
        let parent = path
            .parent()
            .ok_or_else(|| "path must have a parent directory".to_string())?
            .canonicalize()
            .map_err(|e| format!("failed to canonicalize parent: {e}"))?;

        for root in roots {
            let root = root
                .canonicalize()
                .map_err(|e| format!("failed to canonicalize root {}: {e}", root.display()))?;

            if parent.starts_with(root) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

fn parse_paths(input: &str) -> Vec<PathBuf> {
    input
        .split(',')
        .map(|s| PathBuf::from(s.trim()))
        .filter(|p| !p.as_os_str().is_empty())
        .collect()
}
