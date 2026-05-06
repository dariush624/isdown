use crate::check::{Check, CheckCtx, CheckError, CheckOutcome, CheckStatus};
use async_trait::async_trait;
use serde_json::Value;

const URL: &str = "https://health.aws.amazon.com/public/currentevents";

pub struct AwsCheck;

#[async_trait]
impl Check for AwsCheck {
    async fn check(&self, ctx: CheckCtx<'_>) -> Result<Vec<CheckOutcome>, CheckError> {
        let bytes = ctx
            .http_client
            .get(URL)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let json_str = decode_utf16(&bytes)?;
        let incidents: Vec<Value> = serde_json::from_str(&json_str)?;
        Ok(parse_incidents(&incidents))
    }
}

pub(crate) fn parse_incidents(incidents: &[Value]) -> Vec<CheckOutcome> {
    if incidents.is_empty() {
        return vec![CheckOutcome {
            provider: "AWS".to_string(),
            status: CheckStatus::Up,
            causes: vec![],
        }];
    }

    let mut outcomes = Vec::new();
    for incident in incidents {
        let impacted = match incident.get("impacted_services").and_then(|s| s.as_object()) {
            Some(m) => m,
            None => continue,
        };
        let summary = incident
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("Unknown issue");
        let region = incident
            .get("region_name")
            .and_then(|s| s.as_str())
            .unwrap_or("Unknown region");
        let status_code = incident
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("0");

        let status = map_status(status_code);

        for (_, service) in impacted {
            let service_name = service
                .get("service_name")
                .and_then(|s| s.as_str())
                .unwrap_or("Unknown service")
                .to_string();
            outcomes.push(CheckOutcome {
                provider: service_name,
                status: status.clone(),
                causes: vec![format!("{} ({})", summary, region)],
            });
        }
    }

    if outcomes.is_empty() {
        outcomes.push(CheckOutcome {
            provider: "AWS".to_string(),
            status: CheckStatus::Up,
            causes: vec![],
        });
    }

    outcomes
}

fn map_status(code: &str) -> CheckStatus {
    match code {
        "2" => CheckStatus::Degraded,
        "3" => CheckStatus::Down,
        _ => CheckStatus::Up,
    }
}

fn decode_utf16(bytes: &[u8]) -> Result<String, CheckError> {
    let encoding = if bytes.starts_with(&[0xFF, 0xFE]) {
        encoding_rs::UTF_16LE
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        encoding_rs::UTF_16BE
    } else {
        encoding_rs::UTF_8
    };

    let (result, _, had_errors) = encoding.decode(bytes);
    if had_errors {
        return Err(CheckError::ParseError);
    }
    Ok(result.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn map_status_degraded() {
        assert_eq!(map_status("2"), CheckStatus::Degraded);
    }

    #[test]
    fn map_status_down() {
        assert_eq!(map_status("3"), CheckStatus::Down);
    }

    #[test]
    fn map_status_up_for_unknown_codes() {
        assert_eq!(map_status("0"), CheckStatus::Up);
        assert_eq!(map_status("1"), CheckStatus::Up);
        assert_eq!(map_status(""), CheckStatus::Up);
        assert_eq!(map_status("999"), CheckStatus::Up);
    }

    #[test]
    fn decode_utf16_plain_utf8() {
        assert_eq!(decode_utf16(b"[]").unwrap(), "[]");
    }

    #[test]
    fn decode_utf16_utf16le_bom() {
        let mut bytes = vec![0xFF, 0xFE];
        bytes.extend_from_slice(b"h\x00i\x00");
        assert_eq!(decode_utf16(&bytes).unwrap(), "hi");
    }

    #[test]
    fn decode_utf16_utf16be_bom() {
        let mut bytes = vec![0xFE, 0xFF];
        bytes.extend_from_slice(b"\x00h\x00i");
        assert_eq!(decode_utf16(&bytes).unwrap(), "hi");
    }

    #[test]
    fn parse_incidents_empty_is_up() {
        let outcomes = parse_incidents(&[]);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].status, CheckStatus::Up);
        assert_eq!(outcomes[0].provider, "AWS");
    }

    #[test]
    fn parse_incidents_single_incident() {
        let incidents = vec![json!({
            "summary": "S3 errors",
            "region_name": "us-east-1",
            "status": "3",
            "impacted_services": {
                "s3": { "service_name": "Amazon S3" }
            }
        })];
        let outcomes = parse_incidents(&incidents);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].provider, "Amazon S3");
        assert_eq!(outcomes[0].status, CheckStatus::Down);
        assert_eq!(outcomes[0].causes, vec!["S3 errors (us-east-1)"]);
    }

    #[test]
    fn parse_incidents_multiple_impacted_services() {
        let incidents = vec![json!({
            "summary": "Networking issue",
            "region_name": "eu-west-1",
            "status": "2",
            "impacted_services": {
                "ec2": { "service_name": "Amazon EC2" },
                "elb": { "service_name": "Elastic Load Balancing" }
            }
        })];
        let outcomes = parse_incidents(&incidents);
        assert_eq!(outcomes.len(), 2);
        assert!(outcomes.iter().all(|o| o.status == CheckStatus::Degraded));
        let names: Vec<&str> = outcomes.iter().map(|o| o.provider.as_str()).collect();
        assert!(names.contains(&"Amazon EC2"));
        assert!(names.contains(&"Elastic Load Balancing"));
    }

    #[test]
    fn parse_incidents_skips_incident_without_impacted_services() {
        let incidents = vec![
            json!({ "summary": "No services listed", "status": "3" }),
            json!({
                "summary": "Real issue",
                "region_name": "us-west-2",
                "status": "3",
                "impacted_services": { "lambda": { "service_name": "AWS Lambda" } }
            }),
        ];
        let outcomes = parse_incidents(&incidents);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].provider, "AWS Lambda");
    }

    #[test]
    fn parse_incidents_all_skipped_falls_back_to_up() {
        let incidents = vec![json!({ "summary": "No services listed", "status": "3" })];
        let outcomes = parse_incidents(&incidents);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].status, CheckStatus::Up);
        assert_eq!(outcomes[0].provider, "AWS");
    }


    #[tokio::test]
    async fn check_parses_http_response() {
        let server = httpmock::MockServer::start();
        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/currentevents");
            then.status(200)
                .header("content-type", "application/json")
                .body("[]");
        });

        let url = server.url("/currentevents");
        let client = reqwest::Client::new();
        let body = client.get(&url).send().await.unwrap().bytes().await.unwrap();
        let json_str = decode_utf16(&body).unwrap();
        let incidents: Vec<Value> = serde_json::from_str(&json_str).unwrap();
        let outcomes = parse_incidents(&incidents);

        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].status, CheckStatus::Up);
    }
}
