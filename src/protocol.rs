//Functions that maintain the MCP communication standards and provide functional access to the LLM
use crate::tools::{
    docker::{list_containers, list_docker_images},
    filesystem::{list_directory, read_file, write_file, ALLOWED_READABLE_ROOTS, ALLOWED_WRITABLE_ROOTS},
    git::{git_diff, git_diff_file, git_status},
    system::{active_processes, cpu_information, disk_usage},
    common::{allowed_roots_description},
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
            }
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
        _ => tool_error(&format!("unknown tool: {tool_name}")),
    }
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
