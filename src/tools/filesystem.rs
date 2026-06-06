use serde_json::Value;
use crate::tools::common::{tool_error_result, tool_text_result};

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
    let path = arguments["path"].as_str();

    let file_contents = std::fs::read_to_string(path.unwrap());

    let output = match file_contents {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!("failed to read file: {error}"));
        }
    };

    tool_text_result(output)
}
