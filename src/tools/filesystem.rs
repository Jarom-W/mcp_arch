use crate::config::path::PathConfig;
use crate::tools::common::{tool_error_result, tool_text_result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn canonicalize_existing_parent(path: &Path) -> Result<PathBuf, String> {
    let Some(parent) = path.parent() else {
        return Err("path must have a parent directory".to_string());
    };

    parent
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize the parent directory: {error}"))
}

fn get_config() -> PathConfig {
    PathConfig::from_env()
}

pub fn list_directory(arguments: &Value) -> Value {
    let Some(path) = arguments["path"].as_str() else {
        return tool_error_result("path is required".to_string());
    };

    let path = Path::new(path);

    let directory_contents = match fs::read_dir(path) {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!("failed to list directory: {error}"));
        }
    };

    let mut dir_list = String::new();

    for entry in directory_contents.flatten() {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        dir_list.push_str(&file_name);
        dir_list.push('\n');
    }

    tool_text_result(dir_list)
}

pub fn read_file(arguments: &Value) -> Value {
    let config = get_config();

    let Some(path_str) = arguments["path"].as_str() else {
        return tool_error_result("path is required".to_string());
    };

    let path = Path::new(path_str);

    if !path.is_absolute() {
        return tool_error_result("Path must be absolute".to_string());
    }

    if let Err(err) = config.is_readable(path) {
        return tool_error_result(err);
    }

    match config.is_readable(path) {
        Ok(true) => {}
        Ok(false) => {
            return tool_error_result(format!(
                "refusing to read outside allowed roots: {}",
                path.display()
            ));
        }
        Err(err) => {
            return tool_error_result(err);
        }
    }
    if path.is_dir() {
        return tool_error_result("refusing to read because path is a directory".to_string());
    }

    match fs::read_to_string(path) {
        Ok(output) => tool_text_result(output),
        Err(error) => tool_error_result(format!("failed to read file: {error}")),
    }
}

pub fn write_file(arguments: &Value) -> Value {
    let config = get_config();

    let Some(path_str) = arguments["path"].as_str() else {
        return tool_error_result("path is required".to_string());
    };

    let Some(contents) = arguments["contents"].as_str() else {
        return tool_error_result("contents is required".to_string());
    };

    let path = Path::new(path_str);

    if !path.is_absolute() {
        return tool_error_result("Path must be absolute".to_string());
    }

    if let Err(err) = config.is_writable(path) {
        return tool_error_result(err);
    }

    match config.is_writable(path) {
        Ok(true) => {}
        Ok(false) => {
            return tool_error_result(format!(
                "refusing to write outside allowed roots: {}",
                path.display()
            ));
        }
        Err(err) => {
            return tool_error_result(err);
        }
    }
    if path.is_dir() {
        return tool_error_result("refusing to write because path is a directory".to_string());
    }

    match fs::write(path, contents) {
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
    use std::env;
    use std::fs;

    fn tool_text(response: &Value) -> &str {
        response["content"][0]["text"]
            .as_str()
            .expect("tool response should contain text")
    }

    #[test]
    fn canonicalize_existing_parent_returns_real_parent_path() {
        let path = Path::new("/tmp/example_file_that_does_not_exist.txt");

        let parent = canonicalize_existing_parent(path).expect("parent /tmp should canonicalize");

        assert_eq!(parent, PathBuf::from("/tmp"));
    }

    #[test]
    fn read_file_returns_error_when_path_missing() {
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
    fn write_file_returns_error_when_missing_args() {
        let arguments = json!({
            "path": "/tmp/test.txt"
        });

        let response = write_file(&arguments);

        assert_eq!(response["isError"], true);
        assert_eq!(tool_text(&response), "contents is required");
    }

    #[test]
    fn list_directory_returns_entries() {
        let test_dir = "/tmp/mcp_test_list_dir";
        let test_file = format!("{test_dir}/hello.txt");

        fs::create_dir_all(test_dir).unwrap();
        fs::write(&test_file, "hello").unwrap();

        let arguments = json!({
            "path": test_dir
        });

        let response = list_directory(&arguments);
        let text = tool_text(&response);

        assert!(text.contains("hello.txt"));

        fs::remove_file(test_file).ok();
        fs::remove_dir(test_dir).ok();
    }

    fn setup_env(read: &str, write: &str) {
    unsafe {
        env::set_var("MCP_READABLE_ROOTS", read);
        env::set_var("MCP_WRITABLE_ROOTS", write);
        }
    }
}
