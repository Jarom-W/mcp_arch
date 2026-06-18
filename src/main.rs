mod config;
mod protocol;
mod tools;

use std::io;

fn main() {
    dotenvy::dotenv().ok();
    protocol::run_mcp_server(io::stdin(), io::stdout());
}

//Project overview:

//This project is an MCP (model context protocol) server which exposes curated tools to an LLM. This
//allows the LLM to manipulate files and run commands within a specified scope. The compiled binary
//is loaded into a commercial LLMs MCP server slot and the LLM client runs the binary when started.

//Key components:
//main.rs -> Entrypoint of program.

//protocol.rs -> Defines various Json standards to ensure the correct format for MCP. Contains
//initialization commands such as tools/list and tools/call to provide the LLM with the levers it
//has access to.

// ./tools -> This directory contains files grouped by system that each contain commands for the LLM
// to use. Each command is written as a function and wired into protocol.rs handle_tools_call() to
// execute each.
