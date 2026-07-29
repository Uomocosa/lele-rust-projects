//! Remote control plugin for the clicker game.
//!
//! # Vision
//!
//! This plugin will enable remote control of the game via TCP/Unix socket,
//! allowing external tools (like MCP servers, automation scripts, or other
//! applications) to interact with the game without a GUI.
//!
//! ## Planned Features
//!
//! - **TCP Server**: Listen on a configurable port for incoming connections
//! - **JSON Protocol**: Accept commands in JSON format for easy integration
//! - **Command Types**:
//!   - `increment` - Increment the counter
//!   - `status` - Get current game state
//!   - `subscribe` - Subscribe to game events (notifications, updates)
//! - **Event Streaming**: Push events to connected clients in real-time
//! - **Multiple Clients**: Support multiple simultaneous connections
//!
//! ## Use Cases
//!
//! 1. **MCP Integration**: Connect to an MCP server for AI-driven game control
//! 2. **Automation**: Script repetitive actions or testing scenarios
//! 3. **Remote Monitoring**: Observe game state from another machine
//! 4. **Integration Testing**: Drive the game programmatically in tests
//!
//! ## Example Protocol (JSON)
//!
//! ```json
//! // Client sends:
//! {"command": "increment"}
//! {"command": "status"}
//!
//! // Server responds:
//! {"response": "ok", "count": 42}
//! {"response": "status", "count": 42, "contract_key": "..."}
//!
//! // Server pushes events:
//! {"event": "notification", "count": 43}
//! {"event": "update_response", "count": 44}
//! ```
//!
//! ## Implementation Notes
//!
//! - Use `tokio::net::TcpListener` for async TCP server
//! - Spawn a task per connection to handle multiple clients
//! - Use `tokio::sync::broadcast` for event distribution
//! - Consider adding authentication/authorization for security

#[path = "Plugin.rs"]
pub mod plugin;
pub use plugin::RemotePlugin;

pub mod PluginMethod;
