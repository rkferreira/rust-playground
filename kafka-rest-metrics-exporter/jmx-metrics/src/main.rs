use bincode;
use jmx::*;
use serde::{Deserialize, Serialize};
use shared_memory::*;
use std::collections::HashMap;

//use shmap::{Shmap, ShmapError};

const JMX_PORT: i32 = 5555;
const LOOP_INTERVAL: u64 = 10;
const SHMMEM_FILE: &str = "shm_jmx_exporter";
const SHMEM_SIZE: usize = 64;

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

impl JmxMetrics {
    pub fn new() -> Self {
        JmxMetrics {
            connections_active: 0.0,
            request_latency_avg: 0.0,
            request_latency_max: 0.0,
            request_latency_95: 0.0,
            request_latency_99: 0.0,
            request_rate: 0.0,
            response_rate: 0.0,
            request_error_rate: 0.0,
        }
    }
}

fn main() {
    //let shmap = Shmap::new();
    let jmx_attributes = HashMap::from([
        (
            "kafka.rest:type=jetty-metrics",
            Vec::from(["connections-active"]),
        ),
        (
            "kafka.rest:type=jersey-metrics",
            Vec::from([
                "request-latency-avg",
                "request-latency-max",
                "request-latency-95",
                "request-latency-99",
                "request-rate",
                "response-rate",
                "request-error-rate",
            ]),
        ),
    ]);

    let mut shmem = match ShmemConf::new()
        .size(SHMEM_SIZE)
        .flink(SHMMEM_FILE)
        .create()
    {
        Ok(shmem) => shmem,
        Err(ShmemError::LinkExists) => ShmemConf::new()
            .size(SHMEM_SIZE)
            .flink(SHMMEM_FILE)
            .force_create_flink()
            .create()
            .unwrap(),
        Err(e) => {
            println!("Failed to create shared memory: {}", e);
            return;
        }
    };

    //let shm_ptr = shmem.as_ptr();
    let shm_slice = unsafe { shmem.as_slice_mut() };
    println!("Starting JMX exporter");
    let jmx_url = MBeanAddress::service_url(format!(
        "service:jmx:rmi://localhost:{}/jndi/rmi://localhost:{}/jmxrmi",
        JMX_PORT, JMX_PORT
    ));
    let jmx_client = MBeanClient::connect(jmx_url).expect("Failed to connect to the JMX server");
    //let mut jmx_metrics: JmxMetrics = JmxMetrics::new();

    loop {
        let mut jmx_data = HashMap::new();
        let mut jmx_metrics = JmxMetrics::new();
        for (mbean, attribute_list) in jmx_attributes.iter() {
            for attribute in attribute_list.iter() {
                println!("Getting attribute {} from {}", attribute, mbean);
                if let Ok(value) = jmx_client.get_attribute(mbean.to_owned(), attribute.to_owned())
                {
                    println!("Got attribute {} from {}: {}", attribute, mbean, value);
                    jmx_data.insert(format!("{}:{}", mbean, attribute), value);
                    //shmap.insert(attribute, value).expect("Failed to insert into shared memory");
                } else {
                    println!("Failed to get attribute");
                    jmx_data.insert(format!("{}:{}", mbean, attribute), 0.0);
                    //shmap.insert(attribute, 0.0).expect("Failed to insert into shared memory");
                }
            }
        }
        println!("{:?}", jmx_data);
        for (key, value) in jmx_data.iter() {
            match key.as_str() {
                "kafka.rest:type=jetty-metrics:connections-active" => {
                    jmx_metrics.connections_active = *value
                }
                "kafka.rest:type=jersey-metrics:request-latency-avg" => {
                    jmx_metrics.request_latency_avg = *value
                }
                "kafka.rest:type=jersey-metrics:request-latency-max" => {
                    jmx_metrics.request_latency_max = *value
                }
                "kafka.rest:type=jersey-metrics:request-latency-95" => {
                    jmx_metrics.request_latency_95 = *value
                }
                "kafka.rest:type=jersey-metrics:request-latency-99" => {
                    jmx_metrics.request_latency_99 = *value
                }
                "kafka.rest:type=jersey-metrics:request-rate" => jmx_metrics.request_rate = *value,
                "kafka.rest:type=jersey-metrics:response-rate" => {
                    jmx_metrics.response_rate = *value
                }
                "kafka.rest:type=jersey-metrics:request-error-rate" => {
                    jmx_metrics.request_error_rate = *value
                }
                _ => (),
            }
        }
        let serialized: Vec<u8> = bincode::serialize(&jmx_metrics).unwrap();
        shm_slice.copy_from_slice(&serialized);
        std::thread::sleep(std::time::Duration::from_secs(LOOP_INTERVAL));
        /*
        let shmem_ro = match ShmemConf::new().size(SHMEM_SIZE).flink(SHMMEM_FILE).open() {
        Ok(shmem) => shmem,
        Err(e) => {
            println!("Failed to create shared memory: {}", e);
            return;
            }
        };
        let read_shm = unsafe { shmem_ro.as_slice() };
        let deserialized: JmxMetrics = bincode::deserialize(&read_shm).unwrap();
        println!("deseriazlied: {:?} - {}", deserialized, shmem_ro.len());
        println!("");
        */
    }
}
