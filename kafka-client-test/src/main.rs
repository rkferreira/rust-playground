use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaError;
use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
use rdkafka::consumer::{BaseConsumer, Consumer};
use std::time::Duration;

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

fn kafka_produtor_send(producer: &BaseProducer, topic: &str, key: &str, value: &str) -> Result<(), KafkaError> {
    let record = BaseRecord::to(topic).key(key).payload(value);
    producer.send(record).expect("Failed to enqueue");
    for _ in 0..10 {
        producer.poll(Duration::from_millis(100));
    }
    producer.flush(Duration::from_secs(1))
}

fn main() {
    println!("Hello, world!");

    let kafka_server = KafkaServer::default();

    let kafka_producer: BaseProducer = kafka_client_config(&kafka_server).create().expect("Producer creation error");
    let kafka_consumer: BaseConsumer = kafka_client_config(&kafka_server).create().expect("Consumer creation error");

    // Producer
    kafka_produtor_send(&kafka_producer, "test01", "horse-key", "horse-value").expect("Failed to send message");

}
