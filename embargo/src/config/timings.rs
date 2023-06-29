use std::time::Duration;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Refresh {
    Continous(Duration),
    Never,
}
