use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use layer_platform::Anchor;
use tracing::Level;
mod timings;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Script {
    update: timings::Refresh,
    script: String,
}
impl Script {
    fn example() -> Self {
        Self {
            script: "date".to_string(),
            update: timings::Refresh::Continous(std::time::Duration::from_secs(10)),
        }
    }
}
#[derive(Debug)]
pub struct Config {
    //    slint_file: PathBuf,
    pub anchor: Anchor,
    pub config_path: PathBuf,
    pub slint_entrypoint: PathBuf,
    pub layer_name: String,
    pub scripts: HashMap<String, Script>,
}
impl Config {
    pub fn parse(override_path: Option<&Path>) -> anyhow::Result<Self> {
        let (config_dir, is_default_path) = if let Some(path) = override_path {
            (path.to_path_buf(), false)
        } else {
            (Self::default_config_dir()?, true)
        };
        let config_path = config_dir.join("config.toml");
        let config_file = match (config_path.exists(), is_default_path) {
            (true, _) => toml::from_str(&std::fs::read_to_string(&config_path)?)?,
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
            slint_entrypoint: config_file
                .slint_entrypoint
                .unwrap_or_else(|| config_dir.join("slint").join("main.slint")),
            layer_name: config_file.layer_name,
            scripts: config_file.scripts,
            anchor: config_file.anchor.into(),
            config_path,
        })
    }
    fn default_config_dir() -> anyhow::Result<PathBuf> {
        let os_config_dir = dirs::config_dir().ok_or(anyhow::anyhow!(
            "failed to get config dir.  Are you running Linux?"
        ))?;
        let embargo_config_dir = os_config_dir.join(format!("{}_bar", clap::crate_name!()));
        std::fs::create_dir_all(&embargo_config_dir)?;
        Ok(embargo_config_dir)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct ConfigFile {
    anchor: SimpleAnchor,
    layer_name: String,
    scripts: HashMap<String, Script>,
    slint_entrypoint: Option<PathBuf>,
}

impl ConfigFile {
    pub fn generate_default(path: &Path) -> anyhow::Result<()> {
        let config = toml::to_string_pretty(&Self::default())?;
        std::fs::write(path, config.as_bytes())?;
        Ok(())
    }
}
impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            layer_name: clap::crate_name!().to_string(),
            anchor: SimpleAnchor::Top,
            scripts: vec![("example_date".to_string(), Script::example())]
                .into_iter()
                .collect::<HashMap<_, _>>(),
            slint_entrypoint: None,
        }
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum SimpleAnchor {
    Top,
    Bottom,
}
impl Default for SimpleAnchor {
    fn default() -> Self {
        Self::Top
    }
}
impl From<SimpleAnchor> for Anchor {
    fn from(val: SimpleAnchor) -> Self {
        match val {
            SimpleAnchor::Top => Anchor::TOP,
            SimpleAnchor::Bottom => Anchor::BOTTOM,
        }
    }
}
