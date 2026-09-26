use std::sync::Arc;
use std::time::Duration;
use futures::StreamExt;
use kube::{
    api::{Api, Patch, PatchParams, ResourceExt},
    runtime::{
        controller::{Action, Controller},
        finalizer::{finalizer, Error as FinalizerError, Event as FinalizerEvent},
        watcher,
    },
    Client,
};
use serde_json::json;
use tracing::{error, info, warn};

use crate::crd::{MutationRule, MutationRuleStatus};
use crate::error::{Error, Result};
use crate::state::{CompiledRule, RuleStore};

const FINALIZER_NAME: &str = "mutation.webhook.io/finalizer";

pub struct ControllerContext {
    pub client: Client,
    pub store: RuleStore,
}

impl ControllerContext {
    pub fn new(client: Client, store: RuleStore) -> Self {
        Self { client, store }
    }
}

/// Applies/Upserts the mutation rule in the shared store and updates its status
async fn apply_rule(rule: Arc<MutationRule>, ctx: Arc<ControllerContext>) -> Result<Action> {
    let name = rule.name_any();
    let namespace = rule.namespace();
    let key = match &namespace {
        Some(ns) => format!("{}/{}", ns, name),
        None => name.clone(),
    };

    info!(rule = %key, "Reconciling and compiling MutationRule");

    // Validate rule
    let mut is_valid = true;
    let mut err_msg: Option<String> = None;

    if rule.spec.mutations.add_labels.is_empty()
        && rule.spec.mutations.add_annotations.is_empty()
        && rule.spec.mutations.add_env.is_empty()
        && rule.spec.mutations.add_containers.is_empty()
        && rule.spec.mutations.add_init_containers.is_empty()
        && rule.spec.mutations.add_volumes.is_empty()
        && rule.spec.mutations.add_volume_mounts.is_empty()
        && rule.spec.mutations.raw_patches.is_empty()
    {
        is_valid = false;
        err_msg = Some("Rule defines no mutation actions".to_string());
        warn!(rule = %key, "Rule has empty mutation actions");
    }

    if is_valid {
        let compiled = CompiledRule {
            name: name.clone(),
            namespace: namespace.clone(),
            priority: rule.spec.priority,
            spec: rule.spec.clone(),
        };
        ctx.store.upsert(key.clone(), compiled).await;
        let active_count = ctx.store.count().await;
        info!(rule = %key, active_rules = active_count, "Successfully stored MutationRule");
    } else {
        ctx.store.remove(&key).await;
    }

    // Patch status subresource
    let api: Api<MutationRule> = match &namespace {
        Some(ns) => Api::namespaced(ctx.client.clone(), ns),
        None => Api::all(ctx.client.clone()),
    };

    let status = MutationRuleStatus {
        valid: is_valid,
        error_message: err_msg,
        observed_generation: rule.metadata.generation,
    };

    let status_patch = json!({
        "status": status
    });

    let patch_params = PatchParams::apply("rust-mutation-controller");
    let _ = api
        .patch_status(&name, &patch_params, &Patch::Merge(&status_patch))
        .await
        .map_err(|e| {
            warn!(rule = %key, ?e, "Failed to patch MutationRule status");
            e
        });

    Ok(Action::requeue(Duration::from_secs(3600)))
}

/// Cleans up the rule from the shared memory store upon deletion
async fn cleanup_rule(rule: Arc<MutationRule>, ctx: Arc<ControllerContext>) -> Result<Action> {
    let name = rule.name_any();
    let namespace = rule.namespace();
    let key = match &namespace {
        Some(ns) => format!("{}/{}", ns, name),
        None => name.clone(),
    };

    info!(rule = %key, "Cleaning up MutationRule from memory");
    ctx.store.remove(&key).await;
    Ok(Action::await_change())
}

/// Primary reconciler function
async fn reconciler(rule: Arc<MutationRule>, ctx: Arc<ControllerContext>) -> Result<Action> {
    let namespace = rule.namespace();
    let api: Api<MutationRule> = match &namespace {
        Some(ns) => Api::namespaced(ctx.client.clone(), ns),
        None => Api::all(ctx.client.clone()),
    };

    finalizer(&api, FINALIZER_NAME, rule, |event| async {
        match event {
            FinalizerEvent::Apply(rule) => apply_rule(rule, ctx.clone()).await,
            FinalizerEvent::Cleanup(rule) => cleanup_rule(rule, ctx.clone()).await,
        }
    })
    .await
    .map_err(|e| match e {
        FinalizerError::ApplyFailed(err) => err,
        FinalizerError::CleanupFailed(err) => err,
        FinalizerError::AddFinalizer(err) => Error::Kube(err),
        FinalizerError::RemoveFinalizer(err) => Error::Kube(err),
        FinalizerError::UnnamedObject => Error::InvalidRule("Unnamed object".to_string()),
        FinalizerError::InvalidFinalizer => {
            Error::InvalidRule("Invalid finalizer specification".to_string())
        }
    })
}

/// Error handler with exponential / fixed retry backoff
fn on_error(rule: Arc<MutationRule>, error: &Error, _ctx: Arc<ControllerContext>) -> Action {
    error!(
        rule = %rule.name_any(),
        ?error,
        "Reconciliation encountered an error, requeueing in 10s"
    );
    Action::requeue(Duration::from_secs(10))
}

/// Starts the Kubernetes controller loop
pub async fn run_controller(client: Client, store: RuleStore) -> Result<()> {
    info!("Starting MutationRule controller runtime");
    let api: Api<MutationRule> = Api::all(client.clone());
    let ctx = Arc::new(ControllerContext::new(client, store));

    Controller::new(api, watcher::Config::default())
        .run(reconciler, on_error, ctx)
        .for_each(|res| async {
            match res {
                Ok((rule_ref, action)) => {
                    info!(rule = ?rule_ref, ?action, "Reconciled successfully");
                }
                Err(err) => {
                    error!(?err, "Reconciliation error occurred in controller stream");
                }
            }
        })
        .await;

    Ok(())
}
