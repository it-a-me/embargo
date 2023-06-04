use std::path::{Path, PathBuf};

use smithay_client_toolkit::shell::wlr_layer::Anchor;
use tracing::Level;
//mod timings;
#[derive(Debug)]
pub struct Config {
    //    slint_file: PathBuf,
    pub anchor: Anchor,
    pub config_path: PathBuf,
    pub layer_name: String,
}
impl Config {
    pub fn parse(override_path: Option<&Path>) -> anyhow::Result<Self> {
        let (config_path, is_default_path) = if let Some(path) = override_path {
            (path.to_path_buf(), false)
        } else {
            (Self::default_config_path()?, true)
        };
        let config_file = match (config_path.exists(), is_default_path) {
            (true, _) => serde_yaml::from_str(&std::fs::read_to_string(&config_path)?)?,
            (false, false) => {
                anyhow::bail!(
                    "config does not exist at '{}'",
                    config_path.to_string_lossy()
                )
            }
            (false, true) => {
                tracing::event!(
                    Level::INFO,
                    "config does not exist.  Creating default at '{}'",
                    config_path.to_string_lossy()
                );
                ConfigFile::generate_default(&config_path)?;
                ConfigFile::default()
            }
        };
        Ok(Self {
            config_path,
            layer_name: config_file.layer_name,
            anchor: config_file.anchor.into(),
            // timings: config_file.timings,
        })
    }
    fn default_config_path() -> anyhow::Result<PathBuf> {
        let os_config_dir = dirs::config_dir().ok_or(anyhow::anyhow!(
            "failed to get config dir.  Are you running Linux?"
        ))?;
        let embargo_config_dir = os_config_dir.join(format!("{}_bar", clap::crate_name!()));
        std::fs::create_dir_all(&embargo_config_dir)?;
        Ok(embargo_config_dir.join("config.yaml"))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct ConfigFile {
    anchor: SimpleAnchor,
    layer_name: String,
    //timings: Vec<Timing>,
}

impl ConfigFile {
    pub fn generate_default(path: &Path) -> anyhow::Result<()> {
        let config = serde_yaml::to_string(&Self::default())?;
        std::fs::write(path, config.as_bytes())?;
        Ok(())
    }
}
impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            layer_name: clap::crate_name!().to_string(),
            anchor: SimpleAnchor::Top,
            //            timings: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum SimpleAnchor {
    Top,
    Bottom,
}
impl From<SimpleAnchor> for Anchor {
    fn from(val: SimpleAnchor) -> Self {
        match val {
            SimpleAnchor::Top => Anchor::TOP,
            SimpleAnchor::Bottom => Anchor::BOTTOM,
        }
    }
}
