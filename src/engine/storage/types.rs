use serde::Deserialize;
use validator::Validate;

#[derive(Clone, Debug, Deserialize, Validate)]
pub struct StorageConfig {
  #[validate(length(min = 1))]
  pub storage_path: String,
  #[serde(default = "default_snapshots_path")]
  #[validate(length(min = 1))]
  pub snapshots_path: String,
  #[validate(length(min = 1))]
  #[serde(default)]
  pub temp_path: Option<String>,
  #[serde(default = "default_on_disk_payload")]
  pub on_disk_payload: bool,
}

fn default_snapshots_path() -> String {
  "./snapshots".to_string()
}

fn default_on_disk_payload() -> bool {
  false
}
