use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info, warn};

use crate::engine::evaluate_and_patch;
use crate::state::RuleStore;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdmissionReviewRequest {
    pub api_version: Option<String>,
    pub kind: Option<String>,
    pub request: Option<AdmissionRequestPayload>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdmissionRequestPayload {
    pub uid: String,
    pub kind: GroupVersionKind,
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub operation: String,
    pub object: Option<Value>,
    #[serde(default)]
    pub dry_run: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GroupVersionKind {
    pub group: String,
    pub version: String,
    pub kind: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdmissionReviewResponse {
    pub api_version: String,
    pub kind: String,
    pub response: AdmissionResponsePayload,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdmissionResponsePayload {
    pub uid: String,
    pub allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AdmissionStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdmissionStatus {
    pub code: u16,
    pub message: String,
}

/// Handler for /healthz liveness probe
pub async fn healthz_handler() -> &'static str {
    "ok"
}

/// Handler for /readyz readiness probe
pub async fn readyz_handler(State(store): State<RuleStore>) -> impl IntoResponse {
    let count = store.count().await;
    (StatusCode::OK, format!("ready (active rules: {})", count))
}

/// Handler for /mutate admission review
pub async fn mutate_handler(
    State(store): State<RuleStore>,
    Json(payload): Json<AdmissionReviewRequest>,
) -> Response {
    let api_version = payload
        .api_version
        .unwrap_or_else(|| "admission.k8s.io/v1".to_string());

    let req = match payload.request {
        Some(r) => r,
        None => {
            warn!("Received AdmissionReview without request body");
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "apiVersion": api_version,
                    "kind": "AdmissionReview",
                    "response": {
                        "uid": "",
                        "allowed": false,
                        "status": {
                            "code": 400,
                            "message": "Missing admission request payload"
                        }
                    }
                })),
            )
                .into_response();
        }
    };

    let uid = req.uid.clone();
    let kind = req.kind.kind.clone();
    let namespace = req.namespace.as_deref();

    info!(
        uid = %uid,
        kind = %kind,
        namespace = ?namespace,
        operation = %req.operation,
        "Processing admission mutation request"
    );

    let raw_object = match req.object {
        Some(obj) => obj,
        None => {
            // Nothing to mutate, allow request as is
            return Json(AdmissionReviewResponse {
                api_version,
                kind: "AdmissionReview".into(),
                response: AdmissionResponsePayload {
                    uid,
                    allowed: true,
                    status: None,
                    patch_type: None,
                    patch: None,
                },
            })
            .into_response();
        }
    };

    let rules = store.get_all_sorted().await;
    match evaluate_and_patch(&rules, &kind, namespace, &raw_object) {
        Ok(maybe_patch) => {
            let (patch_type, patch) = match maybe_patch {
                Some(p) => (Some("JSONPatch".to_string()), Some(p)),
                None => (None, None),
            };

            Json(AdmissionReviewResponse {
                api_version,
                kind: "AdmissionReview".into(),
                response: AdmissionResponsePayload {
                    uid,
                    allowed: true,
                    status: None,
                    patch_type,
                    patch,
                },
            })
            .into_response()
        }
        Err(err) => {
            error!(uid = %uid, ?err, "Failed to evaluate mutations for admission request");
            Json(AdmissionReviewResponse {
                api_version,
                kind: "AdmissionReview".into(),
                response: AdmissionResponsePayload {
                    uid,
                    allowed: false,
                    status: Some(AdmissionStatus {
                        code: 500,
                        message: format!("Mutation evaluation error: {}", err),
                    }),
                    patch_type: None,
                    patch: None,
                },
            })
            .into_response()
        }
    }
}
