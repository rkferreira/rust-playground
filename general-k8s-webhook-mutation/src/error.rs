use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    Kube(#[from] kube::Error),

    #[error("Serialization / Deserialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML serialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON Patch error: {0}")]
    JsonPatch(#[from] json_patch::PatchError),

    #[error("TLS Generation error: {0}")]
    Rcgen(#[from] rcgen::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Invalid rule configuration: {0}")]
    InvalidRule(String),

    #[error("Admission request error: {0}")]
    Admission(String),
}

pub type Result<T> = std::result::Result<T, Error>;
