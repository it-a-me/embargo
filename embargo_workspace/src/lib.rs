use std::sync::OnceLock;
pub trait Workspace {
    fn state(&self) -> WorkspaceState;
    fn position(&self) -> u32;
    fn display_name(&self) -> &str {
        static NAME: OnceLock<String> = OnceLock::new();
        NAME.get_or_init(|| self.position().to_string())
    }
}
#[derive(Debug, Clone, Copy)]
pub enum WorkspaceState {
    Active,
    Used,
    Unused,
}
