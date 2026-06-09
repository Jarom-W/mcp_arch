//Functions that maintain the MCP communication standards and provide functional access to the LLM
use crate::tools::{
    common::allowed_roots_description,
    docker::{list_containers, list_docker_images},
    filesystem::{
        ALLOWED_READABLE_ROOTS, ALLOWED_WRITABLE_ROOTS, list_directory, read_file, write_file,
    },
    git::{git_branch, git_diff, git_diff_file, git_log, git_status},
    system::{active_processes, cpu_information, disk_usage},
};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

fn validate_request_id_type(msg: &Value) -> Option<Value> {
    match msg.get("id") {
        Some(Value::String(_)) | Some(Value::Number(_)) => msg.get("id").cloned(),
        _ => None,
    }
}

pub fn run_mcp_server(stdin: io::Stdin, mut stdout: io::Stdout) {
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("failed to read line: {error}");
                continue;
            }
        };

        let msg: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("invalid json: {error}");
                continue;
            }
        };

        let method = msg["method"].as_str().unwrap_or("");

        let Some(id) = validate_request_id_type(&msg) else {
            eprintln!("received notification or invalid request without usable id: {method}");
            continue;
        };

        let response = match method {
            "initialize" => json_rpc_result(id, handle_initialize()),
            "tools/list" => json_rpc_result(id, handle_tools_list()),
            "tools/call" => json_rpc_result(id, handle_tools_call(&msg)),
            _ => json_rpc_error(id, -32601, &format!("unknown method: {method}")),
        };

        writeln!(stdout, "{response}").unwrap();
        stdout.flush().unwrap();
    }
}

//Helper functions to standardize results and errors in JSON-RPC
//
fn json_rpc_result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn json_rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn tool_error(message: &str) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": message
            }
        ],
        "isError": true
    })
}

fn handle_initialize() -> Value {
    json!({
        "protocolVersion": "2025-06-18",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "mcp-arch-linux",
            "version": "0.1.0"
        }
    })
}

fn handle_tools_list() -> Value {
    let read_roots = allowed_roots_description(ALLOWED_READABLE_ROOTS);
    let write_roots = allowed_roots_description(ALLOWED_WRITABLE_ROOTS);

    json!({
        "tools": [
            {
                "name": "disk_usage",
                "description": "Shows available space for filesystems the user has access to read.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "human_readable": {
                            "type": "boolean",
                            "description": "Whether to show disk sizes in human-readable format."
                        }
                    },
                    "required": []
                }
            },
            {
                "name": "cpu_information",
                "description": "Shows host machine CPU architectural information.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            },
            {
                "name": "active_processes",
                "description": "Lists running processes.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of process rows to return."
                        }
                    },
                    "required": []
                }
            },
            {
                "name": "list_containers",
                "description": "Lists Docker containers currently running on the machine.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "all": {
                            "type": "boolean",
                            "description": "Whether to include stopped containers."
                        }
                    },
                    "required": []
                }
            },
            {
                "name": "list_docker_images",
                "description": "Lists Docker images available to run on the machine.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "all": {
                            "type": "boolean",
                            "description": "Whether to include all images, tagged or untagged."
                        }
                    },
                    "required": []
                }
            },

            {
                "name": "list_directory",
                "description": "Lists items in a directory at a given path.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to inspect, such as /home/jarom/Projects"
                        },
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "read_file",
                "description": format!("Read contents of a file at an absolute path. Allowed readable roots: {read_roots}"),
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to inspect, such as /home/jarom/Projects/test.txt"
                        },
                    },
                    "required": ["path"]
                }
            },

            {
                "name": "write_file",
                "description": format!("Writes contents to a file under an allowed project directory. Allowed writable roots: {write_roots}"),
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the file for writing."
                        },
                        "contents": {
                            "type": "string",
                            "description": "Full replacement contents for the file in question."
                        }
                    },
                    "required": ["path"]
                }
            },

            {
                "name": "git_status",
                "description": "Check status of git repository.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to repo to inspect status, such as /home/jarom/GitHub/Repo_Name"
                        },
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "git_diff",
                "description": "Check difference of versions in git repository.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to repo to inspect difference, such as /home/jarom/GitHub/Serenity"
                        },
                    },
                    "required": ["path"]
                }
            },

            {
                "name": "git_diff_file",
                "description": "Check difference of versions in git per file.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute filepath to file in quesiton to test difference in version."
                        }
                    },
                    "required": ["path"]
                }
            },

            {
                "name": "git_log",
                "description": "Displays recent git commits. Can be limited using the limit property.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the git repo."
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Quantity of commits to display in order from most recent."
                        }
                    },
                    "required": ["path"]
                }
            },

            {
                "name": "git_branch",
                "description": "Displays git branches in a repository.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute path to the git repo."
                        },
                    },
                    "required": ["path"]
                }
            },

        ]
    })
}

fn handle_tools_call(msg: &Value) -> Value {
    let Some(tool_name) = msg["params"]["name"].as_str() else {
        return tool_error("tools/call requires params.name");
    };

    let arguments = &msg["params"]["arguments"];

    match tool_name {
        "disk_usage" => disk_usage(arguments),
        "cpu_information" => cpu_information(),
        "active_processes" => active_processes(arguments),
        "list_containers" => list_containers(arguments),
        "list_docker_images" => list_docker_images(arguments),
        "list_directory" => list_directory(arguments),
        "read_file" => read_file(arguments),
        "git_status" => git_status(arguments),
        "git_diff" => git_diff(arguments),
        "git_diff_file" => git_diff_file(arguments),
        "write_file" => write_file(arguments),
        "git_log" => git_log(arguments),
        "git_branch" => git_branch(arguments),
        _ => tool_error(&format!("unknown tool: {tool_name}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool_names(tools_list: &Value) -> Vec<&str> {
        tools_list["tools"]
            .as_array()
            .expect("tools should be an array")
            .iter()
            .map(|tool| {
                tool["name"]
                    .as_str()
                    .expect("each tool should have a string name")
            })
            .collect()
    }

    fn find_tool<'a>(tools_list: &'a Value, name: &str) -> &'a Value {
        tools_list["tools"]
            .as_array()
            .expect("tools should be an array")
            .iter()
            .find(|tool| tool["name"] == name)
            .unwrap_or_else(|| panic!("expected tool named {name}"))
    }

    #[test]
    fn validate_request_id_accepts_string_id() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": "abc-123",
            "method": "tools/list"
        });

        assert_eq!(validate_request_id_type(&msg), Some(json!("abc-123")));
    }

    #[test]
    fn validate_request_id_accepts_number_id() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "tools/list"
        });

        assert_eq!(validate_request_id_type(&msg), Some(json!(42)));
    }

    #[test]
    fn validate_request_id_rejects_missing_id() {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        assert_eq!(validate_request_id_type(&msg), None);
    }

    #[test]
    fn validate_request_id_rejects_null_id() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": null,
            "method": "tools/list"
        });

        assert_eq!(validate_request_id_type(&msg), None);
    }

    #[test]
    fn json_rpc_result_wraps_result_with_protocol_fields() {
        let response = json_rpc_result(
            json!(7),
            json!({
                "ok": true
            }),
        );

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 7);
        assert_eq!(response["result"]["ok"], true);
        assert!(response.get("error").is_none());
    }

    #[test]
    fn json_rpc_error_wraps_error_with_code_and_message() {
        let response = json_rpc_error(json!("request-1"), -32601, "unknown method");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "request-1");
        assert_eq!(response["error"]["code"], -32601);
        assert_eq!(response["error"]["message"], "unknown method");
        assert!(response.get("result").is_none());
    }

    #[test]
    fn tool_error_returns_mcp_tool_error_shape() {
        let response = tool_error("something went wrong");

        assert_eq!(response["isError"], true);
        assert_eq!(response["content"][0]["type"], "text");
        assert_eq!(response["content"][0]["text"], "something went wrong");
    }

    #[test]
    fn initialize_returns_expected_server_metadata() {
        let response = handle_initialize();

        assert_eq!(response["protocolVersion"], "2025-06-18");
        assert!(response["capabilities"]["tools"].is_object());
        assert_eq!(response["serverInfo"]["name"], "mcp-arch-linux");
        assert_eq!(response["serverInfo"]["version"], "0.1.0");
    }

    #[test]
    fn tools_list_contains_expected_tools() {
        let response = handle_tools_list();
        let names = tool_names(&response);

        let expected = [
            "disk_usage",
            "cpu_information",
            "active_processes",
            "list_containers",
            "list_docker_images",
            "list_directory",
            "read_file",
            "write_file",
            "git_status",
            "git_diff",
            "git_diff_file",
            "git_log",
            "git_branch",
        ];

        for expected_name in expected {
            assert!(
                names.contains(&expected_name),
                "missing expected tool: {expected_name}"
            );
        }
    }

    #[test]
    fn every_tool_has_name_description_and_input_schema() {
        let response = handle_tools_list();
        let tools = response["tools"]
            .as_array()
            .expect("tools should be an array");

        for tool in tools {
            assert!(
                tool["name"].is_string(),
                "tool missing string name: {tool:?}"
            );
            assert!(
                tool["description"].is_string(),
                "tool missing string description: {tool:?}"
            );
            assert!(
                tool["inputSchema"].is_object(),
                "tool missing inputSchema object: {tool:?}"
            );
            assert_eq!(
                tool["inputSchema"]["type"], "object",
                "tool inputSchema should be object: {tool:?}"
            );
        }
    }

    #[test]
    fn read_file_schema_requires_path() {
        let response = handle_tools_list();
        let tool = find_tool(&response, "read_file");

        let required = tool["inputSchema"]["required"]
            .as_array()
            .expect("required should be an array");

        assert!(required.iter().any(|field| field == "path"));
    }

    #[test]
    fn write_file_schema_requires_path() {
        let response = handle_tools_list();
        let tool = find_tool(&response, "write_file");

        let required = tool["inputSchema"]["required"]
            .as_array()
            .expect("required should be an array");

        assert!(required.iter().any(|field| field == "path"));
    }

    #[test]
    fn write_file_schema_exposes_contents_property() {
        let response = handle_tools_list();
        let tool = find_tool(&response, "write_file");

        assert!(
            tool["inputSchema"]["properties"]["contents"].is_object(),
            "write_file should expose contents property"
        );
    }

    #[test]
    fn git_tools_require_path() {
        let response = handle_tools_list();

        for tool_name in [
            "git_status",
            "git_diff",
            "git_diff_file",
            "git_log",
            "git_branch",
        ] {
            let tool = find_tool(&response, tool_name);

            let required = tool["inputSchema"]["required"]
                .as_array()
                .unwrap_or_else(|| panic!("{tool_name} required should be an array"));

            assert!(
                required.iter().any(|field| field == "path"),
                "{tool_name} should require path"
            );
        }
    }

    #[test]
    fn read_and_write_descriptions_include_allowed_roots_context() {
        let response = handle_tools_list();

        let read_file = find_tool(&response, "read_file");
        let write_file = find_tool(&response, "write_file");

        let read_description = read_file["description"]
            .as_str()
            .expect("read_file description should be a string");

        let write_description = write_file["description"]
            .as_str()
            .expect("write_file description should be a string");

        assert!(
            read_description.contains("Allowed readable roots"),
            "read_file description should mention readable roots"
        );

        assert!(
            write_description.contains("Allowed writable roots"),
            "write_file description should mention writable roots"
        );
    }

    #[test]
    fn tools_call_without_name_returns_tool_error() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "arguments": {}
            }
        });

        let response = handle_tools_call(&msg);

        assert_eq!(response["isError"], true);
        assert_eq!(response["content"][0]["type"], "text");
        assert_eq!(
            response["content"][0]["text"],
            "tools/call requires params.name"
        );
    }

    #[test]
    fn tools_call_unknown_tool_returns_tool_error() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "fake_tool",
                "arguments": {}
            }
        });

        let response = handle_tools_call(&msg);

        assert_eq!(response["isError"], true);
        assert_eq!(response["content"][0]["type"], "text");
        assert_eq!(response["content"][0]["text"], "unknown tool: fake_tool");
    }
}
