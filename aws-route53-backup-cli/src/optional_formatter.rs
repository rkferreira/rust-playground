use aws_sdk_route53 as aws_route53;
use crate::datahelpers as datahelpers;

pub async fn format(res: &mut datahelpers::Record, record: aws_route53::types::ResourceRecordSet) {
    let record = record.clone();
    if let Some(set_identifier) = record.set_identifier {
        res.resourcerecordset.setidentifier = Some(set_identifier);
    }
    if let Some(weight) = record.weight {
        res.resourcerecordset.weight = Some(weight);
    }
    if let Some(region) = record.region {
        res.resourcerecordset.region = match region {
            aws_route53::types::ResourceRecordSetRegion::UsEast1 => Some("us-east-1".to_string()),
            aws_route53::types::ResourceRecordSetRegion::UsEast2 => Some("us-east-2".to_string()),
            aws_route53::types::ResourceRecordSetRegion::UsWest1 => Some("us-west-1".to_string()),
            aws_route53::types::ResourceRecordSetRegion::UsWest2 => Some("us-west-2".to_string()),
            aws_route53::types::ResourceRecordSetRegion::EuWest1 => Some("eu-west-1".to_string()),
            aws_route53::types::ResourceRecordSetRegion::SaEast1 => Some("sa-east-1".to_string()),
            _ => Some("us-east-1".to_string()),
        }
    }
    if let Some(geo_location) = record.geo_location {
        res.resourcerecordset.geolocation = Some(datahelpers::GeoLocation {
            continentcode: geo_location.continent_code,
            countrycode: geo_location.country_code,
            subdivisioncode: geo_location.subdivision_code,
        });
    }
    if let Some(failover) = record.failover {
        res.resourcerecordset.failover = match failover {
            aws_route53::types::ResourceRecordSetFailover::Primary => Some("PRIMARY".to_string()),
            aws_route53::types::ResourceRecordSetFailover::Secondary => Some("SECONDARY".to_string()),
            _ => None,
        };
    }
    if let Some(multi_value_answer) = record.multi_value_answer {
        res.resourcerecordset.multivalueanswer = Some(multi_value_answer);
    }
    if let Some(ttl) = record.ttl {
        res.resourcerecordset.ttl = Some(ttl);
    }
    if let Some(alias_target) = record.alias_target {
        res.resourcerecordset.aliastarget = Some(datahelpers::AliasTarget {
            dnsname: alias_target.dns_name,
            evaluatetargethealth: alias_target.evaluate_target_health,
            hostedzoneid: alias_target.hosted_zone_id,
        });
    }
    if let Some(health_check_id) = record.health_check_id {
        res.resourcerecordset.healthcheckid = Some(health_check_id);
    }
    if let Some(traffic_policy_instance_id) = record.traffic_policy_instance_id {
        res.resourcerecordset.trafficpolicyinstanceid = Some(traffic_policy_instance_id);
    }
    if let Some(cidr) = record.cidr_routing_config {
        res.resourcerecordset.cidrroutingconfig = Some(datahelpers::CidrRoutingConfig {
            collectionid: cidr.collection_id,
            locationname: cidr.location_name,
        });
    }
    if let Some(geo_proximity) = record.geo_proximity_location {
        res.resourcerecordset.geoproximitylocation = Some(datahelpers::GeoProximityLocation {
            awsregion: geo_proximity.aws_region,
            localzonegroup: geo_proximity.local_zone_group,
            bias: geo_proximity.bias,
            coordinates: None
        });
    }
}
