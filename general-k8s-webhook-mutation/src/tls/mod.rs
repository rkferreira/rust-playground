pub mod bootstrap;

pub use bootstrap::{generate_self_signed_certs, patch_mutating_webhook_ca, TlsCertificates};
