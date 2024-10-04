use jmx::MBeanClient;
use jmx::MBeanAddress;
use jmx::MBeanClientTrait;

fn main() {
    static JMX_PORT: i32 = 5555;

    println!("Hello, world!");

    let url = MBeanAddress::service_url(format!(
        "service:jmx:rmi://localhost:{}/jndi/rmi://localhost:{}/jmxrmi",
        JMX_PORT, JMX_PORT
    ));

    let client = MBeanClient::connect(url)
        .expect("Failed to connect to the JMX server");

    println!("{:?}", client.get_mbean_info("kafka.rest:type=jetty-metrics")
        .expect("Failed to get the mbean info"));
    let conns: f64 = client.get_attribute("kafka.rest:type=jetty-metrics", "connections-active")
        .expect("Failed to get the connections-active attribute");
    println!("connections-active: {:?}", conns);

    println!("query: {:?}", client.query_names("kafka.rest:type=*", "*:*")
        .expect("Failed to query names"));
}
