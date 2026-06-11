pub mod claude;
pub mod codex;
pub mod cursor;
pub mod opencode;
pub mod provider;
pub mod registry;

pub use claude::{claude_available, claude_install_hint, ClaudeProvider, DEFAULT_TIMEOUT_SECS};
pub use codex::{codex_available, codex_install_hint, CodexProvider};
pub use cursor::{cursor_available, cursor_install_hint, CursorProvider};
pub use opencode::{opencode_available, opencode_install_hint, OpenCodeProvider};
pub use provider::Provider;
pub use registry::{all_providers, available_providers, find_provider, list_available_names, resolve_provider};
