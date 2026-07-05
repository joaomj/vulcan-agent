use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub provider: ProviderConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
    #[serde(default)]
    pub data_dir: DataDirConfig,
    #[serde(default)]
    pub logs: LogsConfig,
    #[serde(default)]
    pub budgets: BudgetsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(default = "default_provider_kind")]
    pub kind: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_api_key_env")]
    pub api_key_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    #[serde(default = "default_timeout")]
    pub default_timeout_ms: u64,
    #[serde(default = "default_bash_timeout")]
    pub bash_timeout_ms: u64,
    #[serde(default = "default_delegate_timeout")]
    pub delegate_timeout_ms: u64,
    #[serde(default = "default_subagent_max_parallel")]
    pub subagent_max_parallel: u32,
    #[serde(default)]
    pub always_approve_hosts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDirConfig {
    #[serde(default = "default_data_root")]
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsConfig {
    #[serde(default = "default_logs_dir")]
    pub dir: String,
    #[serde(default = "default_log_level")]
    pub level: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BudgetsConfig {}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_provider_kind() -> String {
    "openai-compatible".to_string()
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_api_key_env() -> String {
    "OPENAI_API_KEY".to_string()
}

fn default_timeout() -> u64 {
    30_000
}

fn default_bash_timeout() -> u64 {
    120_000
}

fn default_delegate_timeout() -> u64 {
    600_000
}

fn default_subagent_max_parallel() -> u32 {
    4
}

fn default_data_root() -> String {
    ".vulcan/data".to_string()
}

fn default_logs_dir() -> String {
    ".vulcan/logs".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            provider: ProviderConfig::default(),
            tools: ToolsConfig::default(),
            data_dir: DataDirConfig::default(),
            logs: LogsConfig::default(),
            budgets: BudgetsConfig {},
        }
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            kind: default_provider_kind(),
            base_url: default_base_url(),
            model: default_model(),
            api_key_env: default_api_key_env(),
        }
    }
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: default_timeout(),
            bash_timeout_ms: default_bash_timeout(),
            delegate_timeout_ms: default_delegate_timeout(),
            subagent_max_parallel: default_subagent_max_parallel(),
            always_approve_hosts: Vec::new(),
        }
    }
}

impl Default for DataDirConfig {
    fn default() -> Self {
        Self {
            root: default_data_root(),
        }
    }
}

impl Default for LogsConfig {
    fn default() -> Self {
        Self {
            dir: default_logs_dir(),
            level: default_log_level(),
        }
    }
}

impl ServerConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(ServerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.provider.kind, "openai-compatible");
        assert_eq!(config.provider.model, "gpt-4o-mini");
        assert_eq!(config.provider.api_key_env, "OPENAI_API_KEY");
        assert_eq!(config.tools.default_timeout_ms, 30_000);
        assert_eq!(config.data_dir.root, ".vulcan/data");
        assert_eq!(config.logs.dir, ".vulcan/logs");
        assert_eq!(config.logs.level, "info");
    }
}
