//Git capabilities given to the agent

use crate::tools::common::{tool_error_result, tool_text_result};
use serde_json::Value;

pub fn git_status(arguments: &Value) -> Value {
    let repo_path = arguments["path"].as_str();

    let mut command = std::process::Command::new("git");

    command.arg("-C");

    command.arg(repo_path.unwrap());

    command.arg("status");

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!(
                "failed to check git status at specified path: {error}"
            ));
        }
    };
    tool_text_result(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn git_diff(arguments: &Value) -> Value {
    let repo_path = arguments["path"].as_str();

    let mut command = std::process::Command::new("git");

    command.arg("-C");

    command.arg(repo_path.unwrap());

    command.arg("diff");

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!("failed to check git diff at repository: {error}"));
        }
    };
    tool_text_result(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn git_diff_file(arguments: &Value) -> Value {
    let file_path = arguments["path"].as_str();

    let mut command = std::process::Command::new("git");

    command.arg("diff");

    command.arg("--");

    command.arg(file_path.unwrap());

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!("failed to check git diff at file path: {error}"));
        }
    };
    tool_text_result(String::from_utf8_lossy(&output.stdout).to_string())
}

//TODO -> Add the following commands
//git_log(repo_path, limit)
//git_branch(repo_path)
//git_list_conflicts(repo_path)
//git_show_file(repo_path, file_path, ref?)
//git_add(repo_path, paths)
//git_commit(repo_path, message)
//git_checkout_branch(repo_path, branch)
//git_create_branch(repo_path, branch)
//docker_ps(all)
//docker_images(all)
//docker_logs(container, tail)
//docker_inspect_container(container)
//docker_inspect_image(image)
//docker_compose_ps(project_path)
//docker_build(context_path, dockerfile?, tag)
//docker_run(image, name?, ports?, env?, detach)
//docker_stop(container)
//docker_rm(container)
//docker_rmi(image)
//docker_compose_up(project_path, detach)
//docker_compose_down(project_path)
