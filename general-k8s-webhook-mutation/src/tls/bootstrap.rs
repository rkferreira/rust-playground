use base64::prelude::*;
use k8s_openapi::api::admissionregistration::v1::MutatingWebhookConfiguration;
use kube::{api::{Patch, PatchParams}, Api, Client};
use rcgen::{
    BasicConstraints, CertificateParams, DnType, IsCa, KeyPair, KeyUsagePurpose,
};
use serde_json::json;
use tracing::info;

use crate::error::Result;

pub struct TlsCertificates {
    pub ca_cert_pem: String,
    pub server_cert_pem: String,
    pub server_key_pem: String,
}

/// Generates a self-signed CA and a server certificate valid for the Kubernetes Service DNS names
pub fn generate_self_signed_certs(service_name: &str, namespace: &str) -> Result<TlsCertificates> {
    // 1. Generate CA KeyPair and Certificate
    let ca_key = KeyPair::generate()?;
    let mut ca_params = CertificateParams::default();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_params
        .distinguished_name
        .push(DnType::CommonName, "Webhook Mutation CA");
    ca_params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
        KeyUsagePurpose::DigitalSignature,
    ];

    let ca_cert = ca_params.self_signed(&ca_key)?;
    let ca_cert_pem = ca_cert.pem();

    // 2. Generate Server KeyPair and Certificate signed by CA
    let server_key = KeyPair::generate()?;
    let dns_names = vec![
        service_name.to_string(),
        format!("{}.{}", service_name, namespace),
        format!("{}.{}.svc", service_name, namespace),
        format!("{}.{}.svc.cluster.local", service_name, namespace),
        "localhost".to_string(),
    ];

    let mut server_params = CertificateParams::new(dns_names)?;
    server_params
        .distinguished_name
        .push(DnType::CommonName, format!("{}.{}.svc", service_name, namespace));
    server_params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];
    server_params.is_ca = IsCa::NoCa;

    let server_cert = server_params.signed_by(&server_key, &ca_cert, &ca_key)?;
    let server_cert_pem = server_cert.pem();
    let server_key_pem = server_key.serialize_pem();

    Ok(TlsCertificates {
        ca_cert_pem,
        server_cert_pem,
        server_key_pem,
    })
}

/// Automatically patches the caBundle of the target MutatingWebhookConfiguration in Kubernetes
pub async fn patch_mutating_webhook_ca(
    client: &Client,
    webhook_config_name: &str,
    ca_cert_pem: &str,
) -> Result<()> {
    let api: Api<MutatingWebhookConfiguration> = Api::all(client.clone());
    let ca_bundle_base64 = BASE64_STANDARD.encode(ca_cert_pem.trim());

    info!(
        webhook_config = %webhook_config_name,
        "Patching MutatingWebhookConfiguration caBundle"
    );

    // Fetch existing configuration to find all registered webhooks
    let current = api.get(webhook_config_name).await?;
    let webhooks_count = current.webhooks.as_ref().map(|w| w.len()).unwrap_or(0);

    // Build JSON Patch array for all webhooks in the config
    let mut patch_ops = Vec::new();
    for i in 0..webhooks_count {
        patch_ops.push(json!({
            "op": "add",
            "path": format!("/webhooks/{}/clientConfig/caBundle", i),
            "value": ca_bundle_base64
        }));
    }

    let patch_payload = json!(patch_ops);
    let patch_params = PatchParams::apply("rust-webhook-ca-bootstrapper").force();
    
    api.patch(webhook_config_name, &patch_params, &Patch::Json::<()>(serde_json::from_value(patch_payload)?)).await?;

    info!("Successfully patched caBundle in MutatingWebhookConfiguration");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cert_generation() {
        let certs = generate_self_signed_certs("test-webhook", "default").unwrap();
        assert!(certs.ca_cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(certs.server_cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(certs.server_key_pem.contains("PRIVATE KEY"));
    }
}
