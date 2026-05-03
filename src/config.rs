use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

#[derive(Debug, Clone, Default)]
pub struct EngineConfig {
    pub name: String,
    pub command: String,
    pub args: String,
}

#[derive(Debug, Clone, Default)]
pub struct UserConfig {
    pub language: Option<String>,
    pub active_engine: Option<usize>,
    pub engines: Vec<EngineConfig>,
}

impl UserConfig {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Ok(parse_config(&text))
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, self.to_toml())
            .with_context(|| format!("failed to write {}", path.display()))
    }

    fn to_toml(&self) -> String {
        let mut out = String::new();
        if let Some(language) = &self.language {
            out.push_str(&format!("language = \"{}\"\n", escape_toml(language)));
        }
        if let Some(active_engine) = self.active_engine {
            out.push_str(&format!("active_engine = {active_engine}\n"));
        }
        for engine in &self.engines {
            out.push_str("\n[[engines]]\n");
            out.push_str(&format!("name = \"{}\"\n", escape_toml(&engine.name)));
            out.push_str(&format!("command = \"{}\"\n", escape_toml(&engine.command)));
            out.push_str(&format!("args = \"{}\"\n", escape_toml(&engine.args)));
        }
        out
    }
}

pub fn config_path() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join(".qchess.toml")
}

fn parse_config(text: &str) -> UserConfig {
    let mut config = UserConfig::default();
    let mut current_engine: Option<EngineConfig> = None;

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line == "[[engines]]" {
            if let Some(engine) = current_engine.take() {
                config.engines.push(engine);
            }
            current_engine = Some(EngineConfig::default());
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = parse_value(value.trim());

        if let Some(engine) = current_engine.as_mut() {
            match key {
                "name" => engine.name = value,
                "command" => engine.command = value,
                "args" => engine.args = value,
                _ => {}
            }
        } else {
            match key {
                "language" => config.language = Some(value),
                "active_engine" => config.active_engine = value.parse().ok(),
                _ => {}
            }
        }
    }

    if let Some(engine) = current_engine {
        config.engines.push(engine);
    }
    if config
        .active_engine
        .is_some_and(|idx| idx >= config.engines.len())
    {
        config.active_engine = None;
    }
    config
}

fn parse_value(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(value)
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

fn escape_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
