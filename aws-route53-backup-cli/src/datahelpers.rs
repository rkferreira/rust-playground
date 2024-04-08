use serde::{Deserialize, Serialize};

// https://docs.rs/aws-sdk-route53/latest/aws_sdk_route53/types/struct.ResourceRecordSet.html
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Record {
    #[serde(rename = "Action")]
    pub action: String,
    #[serde(rename = "ResourceRecordSet")]
    pub resourcerecordset: ResourceRecordSet,
}

impl Default for Record {
    fn default() -> Self {
        Record {
            action: "UPSERT".to_string(),
            resourcerecordset: ResourceRecordSet {
                name: "".to_string(),
                r#type: "".to_string(),
                setidentifier: None,
                weight: None,
                region: None,
                geolocation: None,
                failover: None,
                multivalueanswer: None,
                ttl: Some(300),
                resourcerecords: Vec::new(),
                aliastarget: None,
                healthcheckid: None,
                trafficpolicyinstanceid: None,
                cidrroutingconfig: None,
                geoproximitylocation: None,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Changes<T: Serialize> {
    #[serde(rename = "Changes")]
    pub changes: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceRecordSet {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub r#type: String,
    #[serde(rename = "SetIdentifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setidentifier: Option<String>,
    #[serde(rename = "Weight")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
    #[serde(rename = "Region")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(rename = "GeoLocation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geolocation: Option<GeoLocation>,
    #[serde(rename = "Failover")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failover: Option<String>,
    #[serde(rename = "MultiValueAnswer")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multivalueanswer: Option<bool>,
    #[serde(rename = "TTL")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<i64>,
    #[serde(rename = "ResourceRecords")]
    pub resourcerecords: Vec<ResourceRecord>,
    #[serde(rename = "AliasTarget")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliastarget: Option<AliasTarget>,
    #[serde(rename = "HealthCheckId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheckid: Option<String>,
    #[serde(rename = "TrafficPolicyInstanceId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trafficpolicyinstanceid: Option<String>,
    #[serde(rename = "CidrRoutingConfig")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidrroutingconfig: Option<CidrRoutingConfig>,
    #[serde(rename = "GeoProximityLocation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geoproximitylocation: Option<GeoProximityLocation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeoLocation {
    pub continentcode: Option<String>,
    pub countrycode: Option<String>,
    pub subdivisioncode: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceRecord {
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AliasTarget {
    pub hostedzoneid: String,
    pub dnsname: String,
    pub evaluatetargethealth: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CidrRoutingConfig {
    pub collectionid: String,
    pub locationname: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeoProximityLocation {
    pub awsregion: Option<String>,
    pub localzonegroup: Option<String>,
    pub coordinates: Option<Coordinates>,
    pub bias: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Coordinates {
    pub latitude: String,
    pub longitude: String,
}
