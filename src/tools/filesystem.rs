//Functions that allow the LLM to explore the environment and read/write with scoped access.

use crate::tools::common::{tool_error_result, tool_text_result};
use serde_json::Value;
use std::path::{Path, PathBuf};

const ALLOWED_READABLE_ROOTS: &[&str] = &[
    //TODO Make read and write roots in fn is_allowed_path
    "/home/jarom/Projects",
    //Can add more later if it's decided
];
const ALLOWED_WRITABLE_ROOTS: &[&str] = &["/home/jarom/Projects/sandbox"];

fn canonicalize_existing_parent(path: &Path) -> Result<PathBuf, String> {
    let Some(parent) = path.parent() else {
        return Err("path must have a parent directory".to_string());
    };

    parent
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize the parent directory: {error}"))
}

fn is_allowed_path(path: &Path, read: bool) -> Result<bool, String> {
    let parent = canonicalize_existing_parent(path)?;

    if read {
        for root in ALLOWED_READABLE_ROOTS {
            let root_path = Path::new(root)
                .canonicalize()
                .map_err(|error| format!("failed to canonicalize allowed root {root}: {error}"))?;

            if parent.starts_with(root_path) {
                return Ok(true);
            }
        }
        Ok(false)
    } else {
        for root in ALLOWED_WRITABLE_ROOTS {
            let root_path = Path::new(root)
                .canonicalize()
                .map_err(|error| format!("failed to canonicalize allowed root {root}: {error}"))?;

            if parent.starts_with(root_path) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

pub fn list_directory(arguments: &Value) -> Value {
    let path = arguments["path"].as_str();

    let directory_contents = std::fs::read_dir(path.unwrap());

    let output = match directory_contents {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!("failed to list directory: {error}"));
        }
    };

    let entries = output;

    let mut dir_list = String::new();
    for entry in entries {
        if let Ok(entry) = entry {
            let file_name = entry.file_name().to_string_lossy().into_owned();
            dir_list.push_str(&file_name);
            dir_list.push('\n');
        }
    }

    tool_text_result(dir_list)
}

pub fn read_file(arguments: &Value) -> Value {
    let Some(path) = arguments["path"].as_str() else {
        return tool_error_result("path is required".to_string());
    };

    let path = Path::new(path);

    if !path.is_absolute() {
        return tool_error_result("Path must be absolute".to_string());
    }

    match is_allowed_path(path, true) {
        Ok(true) => {}
        Ok(false) => {
            return tool_error_result(format!(
                "refusing to read outside allowed roots: {}",
                path.display()
            ));
        }
        Err(error) => {
            return tool_error_result(error);
        }
    }

    if path.is_dir() {
        return tool_error_result("refusing to read because path is a directory".to_string());
    }

    let output = match std::fs::read_to_string(path) {
        Ok(output) => output,
        Err(error) => return tool_error_result(format!("failed to read file: {error}")),
    };

    tool_text_result(output)
}

pub fn write_file(arguments: &Value) -> Value {
    let Some(path) = arguments["path"].as_str() else {
        return tool_error_result("path is required".to_string());
    };

    let Some(contents) = arguments["contents"].as_str() else {
        return tool_error_result("contents is required".to_string());
    };

    let path = Path::new(path);

    if !path.is_absolute() {
        return tool_error_result("Path must be absolute".to_string());
    }

    match is_allowed_path(path, false) {
        Ok(true) => {}
        Ok(false) => {
            return tool_error_result(format!(
                "refusing to write outside allowed roots: {}",
                path.display()
            ));
        }
        Err(error) => {
            return tool_error_result(error);
        }
    }

    if path.is_dir() {
        return tool_error_result("refusing to write because path is a directory".to_string());
    }

    match std::fs::write(path, contents) {
        Ok(_) => tool_text_result(format!(
            "wrote {} bytes to {}",
            contents.len(),
            path.display()
        )),
        Err(error) => tool_error_result(format!("failed to write file: {error}")),
    }
}
