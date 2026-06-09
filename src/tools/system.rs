//Functions exposed to the agent that check on system information

use crate::tools::common::{tool_error_result, tool_text_result};
use serde_json::Value;

pub fn disk_usage(arguments: &Value) -> Value {
    let human_readable = arguments["human_readable"].as_bool().unwrap_or(true); //Optional clean
    //format toggle
    let mut command = std::process::Command::new("df");

    if human_readable {
        command.arg("-h");
    }

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!(
                "failed to run command to determine disk usage: {error}"
            ));
        }
    };

    tool_text_result(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn cpu_information() -> Value {
    let output = match std::process::Command::new("lscpu").output() {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!("failed to gather cpu information: {error}"));
        }
    };

    tool_text_result(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn active_processes(arguments: &Value) -> Value {
    let limit = arguments["limit"].as_u64().unwrap_or(25);

    let output = match std::process::Command::new("ps").arg("aux").output() {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!("failed to determine active processes: {error}"));
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let limited_output = stdout
        .lines()
        .take(limit as usize)
        .collect::<Vec<_>>()
        .join("\n");

    tool_text_result(limited_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disk_usage_returns_text_content() {
        let result = disk_usage(&serde_json::json!({}));

        assert_eq!(result["content"][0]["type"], "text");
        assert!(result["content"][0]["text"].as_str().is_some());
    }

    #[test]
    fn disk_usage_defaults_to_human_readable_output() {
        let result = disk_usage(&serde_json::json!({}));

        let text = result["content"][0]["text"]
            .as_str()
            .expect("disk usage result should contain text");

        assert!(
            text.contains("Filesystem"),
            "expected df output header, got: {text}"
        );
    }

    #[test]
    fn disk_usage_accepts_human_readable_false() {
        let result = disk_usage(&serde_json::json!({
            "human_readable": false
        }));

        assert_eq!(result["content"][0]["type"], "text");
        assert!(result["content"][0]["text"].as_str().is_some());
    }

    #[test]
    fn cpu_information_returns_text_content() {
        let result = cpu_information();

        assert_eq!(result["content"][0]["type"], "text");
        assert!(result["content"][0]["text"].as_str().is_some());
    }

    #[test]
    fn cpu_information_includes_expected_lscpu_fields() {
        let result = cpu_information();

        let text = result["content"][0]["text"]
            .as_str()
            .expect("cpu information result should contain text");

        assert!(
            text.contains("Architecture") || text.contains("CPU"),
            "expected lscpu output fields, got: {text}"
        );
    }

    #[test]
    fn active_processes_returns_text_content() {
        let result = active_processes(&serde_json::json!({}));

        assert_eq!(result["content"][0]["type"], "text");
        assert!(result["content"][0]["text"].as_str().is_some());
    }

    #[test]
    fn active_processes_defaults_to_limited_output() {
        let result = active_processes(&serde_json::json!({}));

        let text = result["content"][0]["text"]
            .as_str()
            .expect("active processes result should contain text");

        assert!(
            text.lines().count() <= 25,
            "expected default process output to contain at most 25 lines, got {}",
            text.lines().count()
        );
    }

    #[test]
    fn active_processes_respects_limit_argument() {
        let result = active_processes(&serde_json::json!({
            "limit": 5
        }));

        let text = result["content"][0]["text"]
            .as_str()
            .expect("active processes result should contain text");

        assert!(
            text.lines().count() <= 5,
            "expected process output to contain at most 5 lines, got {}",
            text.lines().count()
        );
    }

    #[test]
    fn active_processes_with_limit_one_returns_only_header_line() {
        let result = active_processes(&serde_json::json!({
            "limit": 1
        }));

        let text = result["content"][0]["text"]
            .as_str()
            .expect("active processes result should contain text");

        assert_eq!(text.lines().count(), 1);
        assert!(
            text.contains("USER"),
            "expected ps aux header line, got: {text}"
        );
    }

    #[test]
    fn active_processes_with_limit_zero_returns_empty_text() {
        let result = active_processes(&serde_json::json!({
            "limit": 0
        }));

        assert_eq!(result["content"][0]["text"], "");
    }
}
