//Repeatedly used functions that aren't tool specific
use serde_json::Value;
use std::path::PathBuf;

pub fn tool_text_result(text: String) -> Value {
    serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ]
    })
}

pub fn tool_error_result(message: String) -> Value {
    serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": message
            }
        ],
        "isError": true
    })
}

//Used in protocol.rs to concatinate allowed read/write roots to feed to the agent via the
//'tools/list' function

pub fn allowed_roots_description(roots: &[PathBuf]) -> String {
    if roots.is_empty() {
        return "none".to_string();
    }

    roots
        .iter()
        .map(|root| normalize_root(root))
        .collect::<Vec<_>>()
        .join(", ")
}

fn normalize_root(path: &PathBuf) -> String {
    // Try to make it stable and clean for LLM output
    match path.canonicalize() {
        Ok(canon) => canon.to_string_lossy().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn tool_text_result_returns_text_content_array() {
        let result = tool_text_result("hello from tool".to_string());

        assert_eq!(
            result,
            serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": "hello from tool"
                    }
                ]
            })
        );
    }

    #[test]
    fn tool_text_result_does_not_mark_result_as_error() {
        let result = tool_text_result("successful result".to_string());

        assert!(result.get("isError").is_none());
    }

    #[test]
    fn tool_error_result_returns_text_content_array_with_error_message() {
        let result = tool_error_result("something went wrong".to_string());

        assert_eq!(
            result,
            serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": "something went wrong"
                    }
                ],
                "isError": true
            })
        );
    }

    #[test]
    fn tool_error_result_marks_result_as_error() {
        let result = tool_error_result("permission denied".to_string());

        assert_eq!(result["isError"], true);
    }

    #[test]
    fn allowed_roots_description_joins_multiple_roots() {
        let roots = vec![
            PathBuf::from("/home/jarom/Projects"),
            PathBuf::from("/home/jarom/Projects/sandbox"),
            PathBuf::from("/tmp/example"),
        ];

        let description = allowed_roots_description(&roots);

        assert!(
            description.contains("/home/jarom/Projects"),
            "missing first root"
        );

        assert!(
            description.contains("/home/jarom/Projects/sandbox"),
            "missing second root"
        );

        assert!(
            description.contains("/tmp/example"),
            "missing third root"
        );

        assert!(
            description.contains(", "),
            "expected roots to be comma separated"
        );
    }

    #[test]
    fn allowed_roots_description_returns_single_root() {
        let roots = vec![
            PathBuf::from("/home/jarom/Projects")
        ];

        let description = allowed_roots_description(&roots);

        assert_eq!(
            description,
            "/home/jarom/Projects"
        );
    }

    #[test]
    fn allowed_roots_description_returns_none_when_empty() {
        let roots: Vec<PathBuf> = vec![];

        let description = allowed_roots_description(&roots);

        assert_eq!(description, "none");
    }

    #[test]
    fn normalize_root_returns_original_path_when_canonicalize_fails() {
        let path = PathBuf::from("/definitely/not/a/real/path");

        let normalized = normalize_root(&path);

        assert_eq!(
            normalized,
            "/definitely/not/a/real/path"
        );
    }

    #[test]
    fn normalize_root_returns_canonical_path_when_path_exists() {
        let path = PathBuf::from("/tmp");

        let normalized = normalize_root(&path);

        assert!(
            normalized.ends_with("/tmp"),
            "expected canonicalized tmp path"
        );
    }
}
