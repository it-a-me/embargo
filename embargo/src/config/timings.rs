use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Timing {
    name: String,
    timing: RefreshType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RefreshType {
    Continous(Duration),
    Never,
}
