//Functions that allow the LLM to explore the environment and read/write with scoped access.

use crate::tools::common::{tool_error_result, tool_text_result};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub const ALLOWED_READABLE_ROOTS: &[&str] = &[
    "/home/jarom/Projects",
    //Can add more later if it's decided
];
pub const ALLOWED_WRITABLE_ROOTS: &[&str] = &[
    "/home/jarom/Projects/sandbox",
    //Can also add more here too
];

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
        //Boolean parameter to determine if you're looking at allowed read/write roots so they
        //can be different
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

    let path = Path::new(path); //Convert str path into Path type to run checks

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn tool_text(response: &Value) -> &str {
        response["content"][0]["text"]
            .as_str()
            .expect("tool response should contain text")
    }

    #[test]
    fn allowed_readable_roots_are_configured() {
        assert!(
            !ALLOWED_READABLE_ROOTS.is_empty(),
            "expected at least one readable root"
        );
    }

    #[test]
    fn allowed_writable_roots_are_configured() {
        assert!(
            !ALLOWED_WRITABLE_ROOTS.is_empty(),
            "expected at least one writable root"
        );
    }

    #[test]
    fn canonicalize_existing_parent_returns_real_parent_path_for_existing_parent() {
        let path = Path::new("/tmp/example_file_that_does_not_need_to_exist.txt");

        let parent = canonicalize_existing_parent(path).expect("parent /tmp should canonicalize");

        assert_eq!(parent, PathBuf::from("/tmp"));
    }

    #[test]
    fn canonicalize_existing_parent_returns_error_when_parent_does_not_exist() {
        let path = Path::new("/tmp/definitely_missing_parent_for_mcp_test/file.txt");

        let result = canonicalize_existing_parent(path);

        assert!(
            result.is_err(),
            "expected error when parent directory does not exist"
        );
    }

    #[test]
    fn read_file_returns_error_when_path_is_missing() {
        let arguments = json!({});

        let response = read_file(&arguments);

        assert_eq!(response["isError"], true);
        assert_eq!(tool_text(&response), "path is required");
    }

    #[test]
    fn read_file_returns_error_when_path_is_relative() {
        let arguments = json!({
            "path": "relative/path.txt"
        });

        let response = read_file(&arguments);

        assert_eq!(response["isError"], true);
        assert_eq!(tool_text(&response), "Path must be absolute");
    }

    #[test]
    fn write_file_returns_error_when_path_is_missing() {
        let arguments = json!({
            "contents": "hello"
        });

        let response = write_file(&arguments);

        assert_eq!(response["isError"], true);
        assert_eq!(tool_text(&response), "path is required");
    }

    #[test]
    fn write_file_returns_error_when_contents_are_missing() {
        let arguments = json!({
            "path": "/tmp/test.txt"
        });

        let response = write_file(&arguments);

        assert_eq!(response["isError"], true);
        assert_eq!(tool_text(&response), "contents is required");
    }

    #[test]
    fn write_file_returns_error_when_path_is_relative() {
        let arguments = json!({
            "path": "relative/path.txt",
            "contents": "hello"
        });

        let response = write_file(&arguments);

        assert_eq!(response["isError"], true);
        assert_eq!(tool_text(&response), "Path must be absolute");
    }

    #[test]
    fn list_directory_returns_directory_entries_for_valid_path() {
        let test_dir = "/tmp/mcp_arch_filesystem_test_list_directory";
        let test_file = format!("{test_dir}/hello.txt");

        fs::create_dir_all(test_dir).expect("should create test directory");
        fs::write(&test_file, "hello").expect("should write test file");

        let arguments = json!({
            "path": test_dir
        });

        let response = list_directory(&arguments);
        let text = tool_text(&response);

        assert!(
            text.contains("hello.txt"),
            "directory listing should include test file"
        );

        fs::remove_file(test_file).ok();
        fs::remove_dir(test_dir).ok();
    }

    #[test]
    fn read_file_refuses_path_outside_allowed_roots() {
        let test_file = "/tmp/mcp_arch_read_denied_test.txt";
        fs::write(test_file, "secret").expect("should write test file");

        let arguments = json!({
            "path": test_file
        });

        let response = read_file(&arguments);

        assert_eq!(response["isError"], true);
        assert!(
            tool_text(&response).contains("refusing to read outside allowed roots"),
            "expected refusal for path outside allowed roots"
        );

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn write_file_refuses_path_outside_allowed_roots() {
        let arguments = json!({
            "path": "/tmp/mcp_arch_write_denied_test.txt",
            "contents": "should not be written"
        });

        let response = write_file(&arguments);

        assert_eq!(response["isError"], true);
        assert!(
            tool_text(&response).contains("refusing to write outside allowed roots"),
            "expected refusal for path outside allowed roots"
        );

        fs::remove_file("/tmp/mcp_arch_write_denied_test.txt").ok();
    }
}
