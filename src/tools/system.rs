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
