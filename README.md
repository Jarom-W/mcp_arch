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
        "/home/$USER/.../mcp_arch/target/debug/mcp_arch"
      ]
    }
  },
```
5. The LLM client can now run the binary upon initialization and request/run commands.
