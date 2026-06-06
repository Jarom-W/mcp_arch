mod protocol;
mod tools;

use std::io;

fn main() {
    protocol::run_mcp_server(io::stdin(), io::stdout());
}
