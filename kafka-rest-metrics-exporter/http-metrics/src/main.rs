use axum::http::StatusCode;
use axum::{routing::get, Json, Router};
use bincode;
use serde::{Deserialize, Serialize};
use shared_memory::*;

const SHMMEM_FILE: &str = "shm_jmx_exporter";

#[derive(Debug, Serialize, Deserialize)]
struct JmxMetrics {
    pub connections_active: f64,
    pub request_latency_avg: f64,
    pub request_latency_max: f64,
    pub request_latency_95: f64,
    pub request_latency_99: f64,
    pub request_rate: f64,
    pub response_rate: f64,
    pub request_error_rate: f64,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/stats", get(jmx_stats));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn jmx_stats() -> Result<Json<JmxMetrics>, StatusCode> {
    let shmem_ro = match ShmemConf::new().flink(SHMMEM_FILE).open() {
        Ok(shmem) => shmem,
        Err(e) => {
            println!("Failed to create shared memory: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let read_shm = unsafe { shmem_ro.as_slice() };
    let deserialized: JmxMetrics = bincode::deserialize(&read_shm).unwrap();
    println!("deseriazlied: {:?} - {}", deserialized, shmem_ro.len());
    println!("");
    Ok(Json(deserialized))
}
