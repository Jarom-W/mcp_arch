//Repeatedly used functions that aren't tool specific
use serde_json::Value;

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

pub fn allowed_roots_description(roots: &[&str]) -> String {
    roots.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn allowed_roots_description_joins_multiple_roots_with_comma_and_space() {
        let roots = &[
            "/home/jarom/Projects",
            "/home/jarom/Projects/sandbox",
            "/tmp/example",
        ];

        let description = allowed_roots_description(roots);

        assert_eq!(
            description,
            "/home/jarom/Projects, /home/jarom/Projects/sandbox, /tmp/example"
        );
    }

    #[test]
    fn allowed_roots_description_returns_single_root_without_separator() {
        let roots = &["/home/jarom/Projects"];

        let description = allowed_roots_description(roots);

        assert_eq!(description, "/home/jarom/Projects");
    }

    #[test]
    fn allowed_roots_description_returns_empty_string_when_no_roots_are_provided() {
        let roots: &[&str] = &[];

        let description = allowed_roots_description(roots);

        assert_eq!(description, "");
    }
}
