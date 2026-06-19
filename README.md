# MCP Arch Linux Integration for Agents
This project is an MCP Server binary written in Rust that can be plugged into any commercial LLM. The server provides JSON-RPC interface between
the LLM and the server to allow commands to be executed in your Arch Linux environment.

The commands have been restricted and scoped. For example, specific read/write roots must be provided so the model can't have free reign across your
entire filesystem. 

## Getting Started:

1. Clone the repository into an Arch Linux environment.
2. Ensure rustc is installed in said environment `rustc --version`
3. Compile the binary `cargo build`
4. Expose the linked binary to an LLM of choice via the desktop MCP integrations. I used Claude Desktop with the following claude_desktop_config.json.

```json
"mcpServers": {
    "arch-linux": {
      "command": "wsl.exe",
      "args": [
        "-d",
        "archlinux",
        "--",
        "/home/$USER/path_to_repo/mcp_arch/run_mcp.sh"
      ]
    }
  },
```
5. The shell script included in the root of the repository allows you to define your readable and writable roots. Replace the sample roots in run_mcp.sh with your desired roots. Also change the `exec` command in the same file to the path to your compiled binary.
```bash
#!/usr/bin/env bash

export MCP_READABLE_ROOTS="/home/jarom/Projects" #Sample readable root. Must be an absolute path.
export MCP_WRITABLE_ROOTS="/home/jarom/Projects/sandbox" #Sample writable root. Must also be an absolute path.

exec /home/jarom/Projects/mcp_arch/target/debug/mcp_arch #Must be an absolute path to the binary.
``````
6. Make the shell script executable:

```bash
chmod +x run_mcp.sh
```
7. Boot into your LLM of choice with these configurations in place.
