use aws_config as aws;
use aws_sdk_route53 as aws_route53;
use tokio;

const HOSTED_ZONE_ID: &str = "MyHostedZoneID";

#[tokio::main]
async fn main() {
    let config = aws::from_env()
        .region("us-east-1")
        .retry_config(aws::retry::RetryConfig::adaptive().with_max_attempts(8))
        .load()
        .await;

    let client = aws_route53::Client::new(&config);
    let mut resp = client
        .list_resource_record_sets()
        .hosted_zone_id(HOSTED_ZONE_ID)
        .max_items(100)
        .send()
        .await
        .unwrap();
    for record in resp.resource_record_sets {
        //println!("{:?}", record);
        if record.resource_records.is_some() {
            println!("{} ; {} ; {:?}", record.name, record.r#type, record.resource_records.unwrap());
        }
        if record.alias_target.is_some() {
            println!("{} ; {} ; {:?}", record.name, record.r#type, record.alias_target.unwrap());
        }
    }
    while *(&resp.is_truncated) {
        resp = client
            .list_resource_record_sets()
            .hosted_zone_id(HOSTED_ZONE_ID)
            .max_items(100)
            .start_record_name(&*resp.next_record_name.unwrap())
            .send()
            .await
            .unwrap();
        for record in resp.resource_record_sets {
            //println!("{:?}", record);
            if record.resource_records.is_some() {
                println!("{} ; {} ; {:?}", record.name, record.r#type, record.resource_records.unwrap());
            }
            if record.alias_target.is_some() {
                println!("{} ; {} ; {:?}", record.name, record.r#type, record.alias_target.unwrap());
            }
        }
    }
}
