use anyhow::Result;

pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn available(&self) -> bool;
    fn suggest_sync(&self, query: &str) -> Result<String>;
    fn query_sync(&self, query: &str) -> Result<String>;
    fn install_hint(&self) -> &'static str;
}
