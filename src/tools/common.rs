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
