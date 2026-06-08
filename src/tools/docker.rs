//Functions that give the LLM scoped access to the Docker environment

use crate::tools::common::{tool_error_result, tool_text_result};
use serde_json::Value;

pub fn list_containers(arguments: &Value) -> Value {
    let include_all = arguments["all"].as_bool().unwrap_or(false); //Includes all containers;
                                                                   //running, stopped, and exited
    let mut command = std::process::Command::new("docker");
    command.arg("ps");

    if include_all {
        command.arg("-a");
    }

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!("failed to list docker containers: {error}"));
        }
    };

    tool_text_result(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn list_docker_images(arguments: &Value) -> Value {
    let include_all = arguments["all"].as_bool().unwrap_or(false); //Similar idea; Includes all
                                                                   //images.
    let mut command = std::process::Command::new("docker");
    command.arg("images");

    if include_all {
        command.arg("-a");
    }

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!("failed to list Docker images: {error}"));
        }
    };

    tool_text_result(String::from_utf8_lossy(&output.stdout).to_string())
}
