use std::net::SocketAddr;
use std::path::PathBuf;
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use kube::{Client, CustomResourceExt};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use general_k8s_webhook_mutation::{
    controller::run_controller,
    crd::MutationRule,
    tls::{generate_self_signed_certs, patch_mutating_webhook_ca},
    webhook::{build_app, run_tls_server},
    RuleStore,
};

#[derive(Parser, Debug)]
#[command(
    name = "general-k8s-webhook-mutation",
    author = "Rodrigo",
    version,
    about = "Generic Kubernetes Mutating Admission Webhook and Controller in Rust"
)]
struct Args {
    /// Port to listen for HTTPS admission requests
    #[arg(short, long, env = "PORT", default_value_t = 8443)]
    port: u16,

    /// Bind host address
    #[arg(long, env = "HOST", default_value = "0.0.0.0")]
    host: String,

    /// Path to directory containing tls.crt and tls.key (e.g. mounted from cert-manager Secret)
    #[arg(long, env = "CERT_DIR")]
    cert_dir: Option<PathBuf>,

    /// Automatically generate self-signed CA & certificate with SANs
    #[arg(long, env = "SELF_BOOTSTRAP_TLS", default_value_t = true)]
    self_bootstrap_tls: bool,

    /// Name of the MutatingWebhookConfiguration in Kubernetes to patch with the CA bundle
    #[arg(long, env = "WEBHOOK_CONFIG_NAME")]
    webhook_config_name: Option<String>,

    /// Kubernetes service name for SAN generation
    #[arg(long, env = "SERVICE_NAME", default_value = "webhook-mutation-service")]
    service_name: String,

    /// Kubernetes namespace where service is deployed
    #[arg(long, env = "NAMESPACE", default_value = "default")]
    namespace: String,

    /// Print generated CustomResourceDefinition YAML and exit
    #[arg(long)]
    export_crd: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize structured logging
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "general_k8s_webhook_mutation=info,kube=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    // 2. Export CRD mode
    if args.export_crd {
        let crd = MutationRule::crd();
        let yaml = serde_yaml::to_string(&crd)?;
        println!("{}", yaml);
        return Ok(());
    }

    info!("Starting Generic Kubernetes Mutating Webhook & Controller");

    // 3. Connect to Kubernetes API server
    let client = match Client::try_default().await {
        Ok(c) => {
            info!("Connected to Kubernetes cluster");
            c
        }
        Err(e) => {
            error!(?e, "Failed to connect to Kubernetes cluster");
            return Err(e.into());
        }
    };

    // 4. Configure TLS
    let tls_config = if let Some(cert_dir) = args.cert_dir {
        let cert_path = cert_dir.join("tls.crt");
        let key_path = cert_dir.join("tls.key");
        info!(cert = ?cert_path, key = ?key_path, "Loading TLS certificates from directory");
        RustlsConfig::from_pem_file(cert_path, key_path).await?
    } else if args.self_bootstrap_tls {
        info!(
            service = %args.service_name,
            namespace = %args.namespace,
            "Generating self-signed CA and server certificates"
        );
        let certs = generate_self_signed_certs(&args.service_name, &args.namespace)?;

        // If target MutatingWebhookConfiguration name is specified, patch its caBundle
        if let Some(config_name) = &args.webhook_config_name {
            if let Err(e) = patch_mutating_webhook_ca(&client, config_name, &certs.ca_cert_pem).await {
                error!(?e, "Failed to auto-patch MutatingWebhookConfiguration caBundle");
            }
        }

        RustlsConfig::from_pem(
            certs.server_cert_pem.as_bytes().to_vec(),
            certs.server_key_pem.as_bytes().to_vec(),
        )
        .await?
    } else {
        return Err("No TLS certificates provided and --self-bootstrap-tls is disabled".into());
    };

    // 5. Initialize in-memory RuleStore and Axum router
    let store = RuleStore::new();
    let app = build_app(store.clone());

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;

    // 6. Spawn Controller and Webhook Server concurrently
    let controller_handle = {
        let client = client.clone();
        let store = store.clone();
        tokio::spawn(async move {
            if let Err(e) = run_controller(client, store).await {
                error!(?e, "Controller encountered fatal error");
            }
        })
    };

    let server_handle = tokio::spawn(async move {
        if let Err(e) = run_tls_server(addr, app, tls_config).await {
            error!(?e, "HTTPS Webhook server encountered fatal error");
        }
    });

    info!(%addr, "Webhook controller running. Press Ctrl+C to terminate");

    // 7. Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal (Ctrl+C). Initiating graceful shutdown");
        }
        res = controller_handle => {
            info!(?res, "Controller task completed");
        }
        res = server_handle => {
            info!(?res, "Server task completed");
        }
    }

    info!("Server shutdown completed");
    Ok(())
}
