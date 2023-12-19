use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaError;
use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
use rdkafka::consumer::{BaseConsumer, Consumer};
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use std::time::Duration;
use apache_avro::types::Value;
use std::env;

struct KafkaServer {
    pub bootstrap_servers: String,
    pub security_protocol: String,
    pub sasl_mechanisms: String,
    pub sasl_username: String,
    pub sasl_password: String
}

impl KafkaServer {
    pub fn new(bootstrap_servers: String, security_protocol: String, sasl_mechanisms: String, sasl_username: String, sasl_password: String) -> KafkaServer {
        KafkaServer {
            bootstrap_servers,
            security_protocol,
            sasl_mechanisms,
            sasl_username,
            sasl_password
        }
    }
}

impl Default for KafkaServer {
    fn default() -> Self {
        KafkaServer {
            bootstrap_servers: String::from("b-4.xxx.us-east-1.amazonaws.com:9096"),
            security_protocol: String::from("sasl_ssl"),
            sasl_mechanisms: String::from("SCRAM-SHA-512"),
            sasl_username: String::from("admin"),
            sasl_password: String::from("admin")
        }
    }
}

fn kafka_client_config(kafka_server: &KafkaServer) -> ClientConfig {
    let mut client_config = ClientConfig::new();
    client_config
        .set("bootstrap.servers", &kafka_server.bootstrap_servers)
        .set("security.protocol", &kafka_server.security_protocol)
        .set("sasl.mechanisms", &kafka_server.sasl_mechanisms)
        .set("sasl.username", &kafka_server.sasl_username)
        .set("sasl.password", &kafka_server.sasl_password);

    client_config
}

fn kafka_producer_send(producer: &BaseProducer, topic: &str, key: &str, value: &str) -> Result<(), KafkaError> {
    let record = BaseRecord::to(topic).key(key).payload(value);
    producer.send(record).expect("Failed to enqueue");
    for _ in 0..10 {
        producer.poll(Duration::from_millis(100));
    }
    producer.flush(Duration::from_secs(1))
}

async fn kafka_producer_encode_send(producer: &BaseProducer, topic: &str, key: &str, value: &str, schema_url: &str) -> Result<(), KafkaError> {
    let sr_settings = SrSettings::new(schema_url.to_string());

    let encoder = AvroEncoder::new(sr_settings);

    let strategy = SubjectNameStrategy::TopicNameStrategy(topic.to_string(), false);
    let payload = encoder.encode(vec![(value, Value::String("here we go again".to_string()))], strategy).await.unwrap();
    let record = BaseRecord::to(&topic).key(key).payload(&payload);

    producer.send(record).expect("Failed to enqueue");
    for _ in 0..10 {
        producer.poll(Duration::from_millis(100));
    }
    producer.flush(Duration::from_secs(1))
}

fn set_params() -> (String, String) {
    let kafka_topic;
    let kafka_schema_url;

    if let Ok(val) = env::var("KAFKA_TOPIC") {
        println!("..using env KAFKA_TOPIC: {}", val);
        kafka_topic = val;
    } else {
        kafka_topic  = "dummy".to_string();
    }

    if let Ok(val) = env::var("KAFKA_SCHEMA_URL") {
        println!("..using env KAFKA_SCHEMA_URL: {}", val);
        kafka_schema_url = val;
    } else {
        kafka_schema_url   = "http://127.0.0.1:80".to_string();
    }

    return (kafka_topic, kafka_schema_url);
}

#[tokio::main]
async fn main() {
    println!("\n Welcome to kafka client test! \n");

    let (kafka_topic, schema_url)  = set_params();

    let kafka_server = KafkaServer::default();

    let kafka_producer: BaseProducer = kafka_client_config(&kafka_server).create().expect("Producer creation error");
    //let kafka_consumer: BaseConsumer = kafka_client_config(&kafka_server).create().expect("Consumer creation error");

    // Producer
    kafka_producer_send(&kafka_producer, &kafka_topic, "horse-key", "horsevalue").expect("Failed to send message");

    // Avro encoded
    kafka_producer_encode_send(&kafka_producer, &kafka_topic, "horse-key", "horsevalue", &schema_url).await.expect("Failed to send message");
}
