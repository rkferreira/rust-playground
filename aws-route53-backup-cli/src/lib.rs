use aws_sdk_route53 as aws_route53;
use aws_sdk_s3 as aws_s3;
use aws_smithy_runtime_api::client::result::SdkError;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::byte_stream::ByteStream;
use chrono::{DateTime, Utc};

mod datahelpers;
mod optional_formatter;

const AWS_S3_BUCKET: &str = "s3-test";
const MAX_FILE_ITEMS: usize = 5;

#[derive(Debug, Clone)]
pub struct Domain {
    pub name: String,
    pub id: String,
}

#[tracing::instrument(skip(client))]
pub async fn list_domains(client: aws_route53::Client) -> Vec<Domain> {
    let mut domains: Vec<Domain> = Vec::new();
    let mut resp = client
        .list_hosted_zones_by_name()
        .max_items(100)
        .send()
        .await
        .unwrap();
    for zone in resp.hosted_zones {
        domains.push(Domain {
            name: zone.name,
            id: zone.id,
        });
    }
    while *(&resp.is_truncated) {
        resp = client
            .list_hosted_zones_by_name()
            .max_items(100)
            .dns_name(&*resp.next_dns_name.unwrap())
            .hosted_zone_id(&*resp.next_hosted_zone_id.unwrap())
            .send()
            .await
            .unwrap();
        for zone in resp.hosted_zones {
            domains.push(Domain {
                name: zone.name,
                id: zone.id,
            });
        }
    }
    domains
}

#[tracing::instrument(skip(client))]
pub async fn list_resource_records(
    client: aws_route53::Client,
    hosted_zone: Domain,
) -> Vec<aws_route53::types::ResourceRecordSet> {
    let mut records: Vec<aws_route53::types::ResourceRecordSet> = Vec::new();
    let mut resp = client
        .list_resource_record_sets()
        .hosted_zone_id(&hosted_zone.id)
        .max_items(100)
        .send()
        .await
        .unwrap();
    for record in resp.resource_record_sets {
        records.push(record);
    }
    while *(&resp.is_truncated) {
        resp = client
            .list_resource_record_sets()
            .hosted_zone_id(&hosted_zone.id)
            .max_items(100)
            .start_record_name(&*resp.next_record_name.unwrap())
            .send()
            .await
            .unwrap();
        for record in resp.resource_record_sets {
            records.push(record);
        }
    }
    records
}

#[tracing::instrument(skip(client))]
pub async fn format_and_write(
    client: aws_s3::Client,
    domain: String,
    records: Vec<aws_route53::types::ResourceRecordSet>,
) {
    let utc: DateTime<Utc> = Utc::now();
    let utc_day = utc.format("%Y-%m-%d").to_string();
    let utc_time = utc.format("%H:%M:%S").to_string();

    let mut formatted_records: Vec<datahelpers::Record> = Vec::new();
    let mut file_index = 0;
    let mut file_name_index = 0;
    let total_records = records.len();

    for record in records {
        if (record.r#type == aws_sdk_route53::types::RrType::Ns)
            || (record.r#type == aws_sdk_route53::types::RrType::Soa)
        {
            continue;
        }
        let mut res: datahelpers::Record = Default::default();
        let mut resource_records: Vec<datahelpers::ResourceRecord> = Vec::new();
        if let Some(rr) = record.resource_records.to_owned() {
            for r in rr {
                resource_records.push(datahelpers::ResourceRecord { value: r.value });
            }
        }
        // Mandatory fields
        // https://docs.rs/aws-sdk-route53/latest/aws_sdk_route53/types/struct.ResourceRecordSet.html
        res.resourcerecordset.name = record.name.to_owned();
        res.resourcerecordset.r#type = record.r#type.to_string();
        res.resourcerecordset.resourcerecords = resource_records;
        // Optional fields, some decomposing, its ugly
        optional_formatter::format(&mut res, record.clone()).await;

        formatted_records.push(res);
        file_index += 1;
        if total_records > MAX_FILE_ITEMS && file_index == MAX_FILE_ITEMS {
            let changes = datahelpers::Changes {
                changes: formatted_records.clone(),
            };
            let json = serde_json::to_string(&changes).unwrap();
            let stream = ByteStream::new(SdkBody::from(json.as_bytes()));
            let file_name = format!(
                "{}/{}/{}/{}.json",
                domain, utc_day, utc_time, file_name_index
            );
            write_to_s3(client.clone(), stream, file_name)
                .await
                .unwrap();
            formatted_records.clear();
            file_index = 0;
            file_name_index += 1;
        }
    }
    let changes = datahelpers::Changes {
        changes: formatted_records.clone(),
    };
    let json = serde_json::to_string(&changes).unwrap();
    let stream = ByteStream::new(SdkBody::from(json.as_bytes()));
    let file_name = format!(
        "{}/{}/{}/{}.json",
        domain, utc_day, utc_time, file_name_index
    );
    write_to_s3(client.clone(), stream, file_name)
        .await
        .unwrap();
}

#[tracing::instrument(skip(client, stream))]
async fn write_to_s3(
    client: aws_s3::Client,
    stream: ByteStream,
    file_name: String,
) -> Result<
    aws_s3::operation::put_object::PutObjectOutput,
    SdkError<
        aws_s3::operation::put_object::PutObjectError,
        aws_smithy_runtime_api::client::orchestrator::HttpResponse,
    >,
> {
    client
        .put_object()
        .bucket(AWS_S3_BUCKET)
        .key(&file_name)
        .body(stream)
        .send()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_config::BehaviorVersion;
    use aws_sdk_route53 as r53;
    use aws_smithy_runtime::client::http::test_util::{ReplayEvent, StaticReplayClient};
    use aws_smithy_types::body::SdkBody;

    fn aws_credentials_provider() -> r53::config::Credentials {
        r53::config::Credentials::new(
            "access_key".to_string(),
            "secret_key".to_string(),
            Some("token".to_string()),
            None,
            "name",
        )
    }

    #[tokio::test]
    async fn test_list_domains_2_domains() {
        let http_request = http::Request::builder()
            .method("GET")
            .uri("https://route53.amazonaws.com/2013-04-01/hostedzonesbyname?maxitems=100")
            .body(SdkBody::empty())
            .unwrap();

        let http_response = http::Response::builder()
            .status(200)
            .body(SdkBody::from(include_str!("./testing/list_domains.xml")))
            .unwrap();

        let event = ReplayEvent::new(http_request, http_response);
        let event_client = StaticReplayClient::new(vec![event]);
        let client = r53::Client::from_conf(
            r53::Config::builder()
                .behavior_version(BehaviorVersion::latest())
                .credentials_provider(aws_credentials_provider())
                .region(r53::config::Region::new("us-east-1"))
                .http_client(event_client)
                .build(),
        );
        let domains = list_domains(client).await;
        assert_eq!(domains.len(), 2);
    }
}
