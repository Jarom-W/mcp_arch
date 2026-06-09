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

pub fn git_log(arguments: &Value) -> Value {
    let repo_path = arguments["path"].as_str().unwrap_or("/home/jarom/GitHub");

    let limit = arguments["limit"].as_u64().unwrap_or(5);

    let output = match std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("log")
        .arg("-n")
        .arg(limit.to_string())
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!("failed to display git commit logs: {error}"));
        }
    };

    tool_text_result(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn git_branch(arguments: &Value) -> Value {
    let repo_path = arguments["path"].as_str().unwrap_or("/home/jarom/GitHub");

    let output = match std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("branch")
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return tool_error_result(format!("failed to display current git branch: {error}"));
        }
    };

    tool_text_result(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_repo_path(test_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("rust_mcp_git_test_{test_name}_{timestamp}"))
    }

    fn run_git_command(repo_path: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(args)
            .output()
            .expect("git command should execute");

        assert!(
            output.status.success(),
            "git command failed: git -C {} {}\nstderr: {}",
            repo_path.display(),
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn create_test_repo(test_name: &str) -> PathBuf {
        let repo_path = unique_test_repo_path(test_name);

        fs::create_dir_all(&repo_path).expect("test repo directory should be created");

        let init_output = Command::new("git")
            .arg("init")
            .arg(&repo_path)
            .output()
            .expect("git init should execute");

        assert!(
            init_output.status.success(),
            "git init failed: {}",
            String::from_utf8_lossy(&init_output.stderr)
        );

        run_git_command(&repo_path, &["config", "user.email", "test@example.com"]);
        run_git_command(&repo_path, &["config", "user.name", "Rust MCP Test"]);

        let readme_path = repo_path.join("README.md");
        fs::write(&readme_path, "initial contents\n").expect("README should be written");

        run_git_command(&repo_path, &["add", "README.md"]);
        run_git_command(&repo_path, &["commit", "-m", "Initial commit"]);

        repo_path
    }

    #[test]
    fn git_status_returns_text_content_for_valid_repository() {
        let repo_path = create_test_repo("status_returns_text_content");

        let result = git_status(&serde_json::json!({
            "path": repo_path
        }));

        assert_eq!(result["content"][0]["type"], "text");
        assert!(result["content"][0]["text"].as_str().is_some());

        fs::remove_dir_all(repo_path).expect("test repo should be cleaned up");
    }

    #[test]
    fn git_status_reports_clean_working_tree_for_new_test_repository() {
        let repo_path = create_test_repo("status_reports_clean_working_tree");

        let result = git_status(&serde_json::json!({
            "path": repo_path
        }));

        let text = result["content"][0]["text"]
            .as_str()
            .expect("git status result should contain text");

        assert!(
            text.contains("working tree clean") || text.contains("nothing to commit"),
            "expected clean working tree message, got: {text}"
        );

        fs::remove_dir_all(repo_path).expect("test repo should be cleaned up");
    }

    #[test]
    fn git_diff_returns_text_content_for_valid_repository() {
        let repo_path = create_test_repo("diff_returns_text_content");

        fs::write(repo_path.join("README.md"), "changed contents\n")
            .expect("README should be modified");

        let result = git_diff(&serde_json::json!({
            "path": repo_path
        }));

        assert_eq!(result["content"][0]["type"], "text");
        assert!(result["content"][0]["text"].as_str().is_some());

        fs::remove_dir_all(repo_path).expect("test repo should be cleaned up");
    }

    #[test]
    fn git_diff_includes_modified_file_contents() {
        let repo_path = create_test_repo("diff_includes_modified_file_contents");

        fs::write(repo_path.join("README.md"), "changed contents\n")
            .expect("README should be modified");

        let result = git_diff(&serde_json::json!({
            "path": repo_path
        }));

        let text = result["content"][0]["text"]
            .as_str()
            .expect("git diff result should contain text");

        assert!(text.contains("-initial contents"));
        assert!(text.contains("+changed contents"));

        fs::remove_dir_all(repo_path).expect("test repo should be cleaned up");
    }

    #[test]
    fn git_log_respects_limit_argument() {
        let repo_path = create_test_repo("log_respects_limit_argument");

        fs::write(repo_path.join("second.txt"), "second commit\n")
            .expect("second file should be written");
        run_git_command(&repo_path, &["add", "second.txt"]);
        run_git_command(&repo_path, &["commit", "-m", "Second commit"]);

        let result = git_log(&serde_json::json!({
            "path": repo_path,
            "limit": 1
        }));

        let text = result["content"][0]["text"]
            .as_str()
            .expect("git log result should contain text");

        assert!(text.contains("Second commit"));
        assert!(!text.contains("Initial commit"));

        fs::remove_dir_all(repo_path).expect("test repo should be cleaned up");
    }

    #[test]
    fn git_branch_returns_current_branch_information() {
        let repo_path = create_test_repo("branch_returns_current_branch_information");

        let result = git_branch(&serde_json::json!({
            "path": repo_path
        }));

        let text = result["content"][0]["text"]
            .as_str()
            .expect("git branch result should contain text");

        assert!(
            text.contains("*"),
            "expected current branch marker in git branch output, got: {text}"
        );

        fs::remove_dir_all(repo_path).expect("test repo should be cleaned up");
    }

    #[test]
    #[should_panic]
    fn git_status_panics_when_path_argument_is_missing() {
        let _ = git_status(&serde_json::json!({}));
    }

    #[test]
    #[should_panic]
    fn git_diff_panics_when_path_argument_is_missing() {
        let _ = git_diff(&serde_json::json!({}));
    }

    #[test]
    #[should_panic]
    fn git_diff_file_panics_when_path_argument_is_missing() {
        let _ = git_diff_file(&serde_json::json!({}));
    }
}

//TODO -> Add the following commands
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
