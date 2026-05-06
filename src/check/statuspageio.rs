use crate::check::{CheckError, CheckStatus, ProviderCheck};
use serde_json::Value;

macro_rules! statuspage_provider {
    ($struct_name:ident, $name:literal, $host:literal) => {
        pub struct $struct_name;

        impl ProviderCheck for $struct_name {
            fn provider(&self) -> &'static str {
                $name
            }

            fn url(&self) -> &str {
                concat!("https://", $host, "/api/v2/summary.json")
            }

            fn parse_status(&self, value: &Value) -> Result<String, CheckError> {
                value
                    .get("status")
                    .and_then(|s| s.get("indicator"))
                    .and_then(|i| i.as_str())
                    .map(|s| s.to_string())
                    .ok_or(CheckError::ParseError)
            }

            fn map_status(&self, status: String) -> CheckStatus {
                match status.as_str() {
                    "none" => CheckStatus::Up,
                    "minor" => CheckStatus::Degraded,
                    _ => CheckStatus::Down,
                }
            }

            fn causes(&self, value: &Value) -> Vec<String> {
                value
                    .get("incidents")
                    .and_then(|i| i.as_array())
                    .map(|incidents| {
                        incidents
                            .iter()
                            .filter_map(|i| {
                                let name = i.get("name")?.as_str()?;
                                let status = i.get("status")?.as_str()?;
                                Some(format!("{} ({})", name, status))
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            }
        }
    };
}

statuspage_provider!(AtlassianCheck, "Atlassian", "status.atlassian.com");
statuspage_provider!(CircleCICheck, "CircleCI", "status.circleci.com");
statuspage_provider!(CloudflareCheck, "Cloudflare", "www.cloudflarestatus.com");
statuspage_provider!(DatadogCheck, "Datadog", "status.datadoghq.com");
statuspage_provider!(DiscordCheck, "Discord", "discordstatus.com");
statuspage_provider!(LinearCheck, "Linear", "linearstatus.com");
statuspage_provider!(NetlifyCheck, "Netlify", "www.netlifystatus.com");
statuspage_provider!(NpmCheck, "npm", "status.npmjs.org");
statuspage_provider!(OpenAICheck, "OpenAI", "status.openai.com");
statuspage_provider!(VercelCheck, "Vercel", "www.vercel-status.com");
statuspage_provider!(GitHubCheck, "GitHub", "www.githubstatus.com");
statuspage_provider!(ClaudeCheck, "Claude", "status.claude.com");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::{Check, CheckCtx, CheckStatus, ProviderCheck};
    use serde_json::json;

    fn check() -> GitHubCheck {
        GitHubCheck
    }

    #[test]
    fn parse_status_ok() {
        let v = json!({ "status": { "indicator": "none" } });
        assert_eq!(check().parse_status(&v).unwrap(), "none");
    }

    #[test]
    fn parse_status_missing_returns_error() {
        assert!(check().parse_status(&json!({})).is_err());
    }

    #[test]
    fn map_status_none_is_up() {
        assert_eq!(check().map_status("none".to_string()), CheckStatus::Up);
    }

    #[test]
    fn map_status_minor_is_degraded() {
        assert_eq!(
            check().map_status("minor".to_string()),
            CheckStatus::Degraded
        );
    }

    #[test]
    fn map_status_other_is_down() {
        assert_eq!(check().map_status("major".to_string()), CheckStatus::Down);
        assert_eq!(
            check().map_status("critical".to_string()),
            CheckStatus::Down
        );
    }

    #[test]
    fn causes_empty_when_no_incidents() {
        assert!(check().causes(&json!({ "incidents": [] })).is_empty());
    }

    #[test]
    fn causes_formats_name_and_status() {
        let v = json!({ "incidents": [{ "name": "API outage", "status": "investigating" }] });
        assert_eq!(check().causes(&v), vec!["API outage (investigating)"]);
    }

    #[test]
    fn causes_multiple_incidents() {
        let v = json!({
            "incidents": [
                { "name": "Slow builds", "status": "monitoring" },
                { "name": "Pages down", "status": "identified" }
            ]
        });
        let causes = check().causes(&v);
        assert_eq!(causes.len(), 2);
        assert!(causes.contains(&"Slow builds (monitoring)".to_string()));
        assert!(causes.contains(&"Pages down (identified)".to_string()));
    }

    #[test]
    fn causes_skips_malformed_incidents() {
        assert!(
            check()
                .causes(&json!({ "incidents": [{ "no_name": true }] }))
                .is_empty()
        );
    }

    struct MockProvider {
        url: String,
    }

    impl ProviderCheck for MockProvider {
        fn provider(&self) -> &'static str {
            "TestProvider"
        }
        fn url(&self) -> &str {
            &self.url
        }

        fn parse_status(&self, value: &serde_json::Value) -> Result<String, CheckError> {
            value
                .get("status")
                .and_then(|s| s.get("indicator"))
                .and_then(|i| i.as_str())
                .map(|s| s.to_string())
                .ok_or(CheckError::ParseError)
        }

        fn map_status(&self, status: String) -> CheckStatus {
            match status.as_str() {
                "none" => CheckStatus::Up,
                "minor" => CheckStatus::Degraded,
                _ => CheckStatus::Down,
            }
        }

        fn causes(&self, value: &serde_json::Value) -> Vec<String> {
            value
                .get("incidents")
                .and_then(|i| i.as_array())
                .map(|incidents| {
                    incidents
                        .iter()
                        .filter_map(|i| {
                            let name = i.get("name")?.as_str()?;
                            let status = i.get("status")?.as_str()?;
                            Some(format!("{} ({})", name, status))
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
    }

    #[tokio::test]
    async fn check_returns_up_when_indicator_none() {
        let server = httpmock::MockServer::start();
        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/summary.json");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"status":{"indicator":"none"},"incidents":[]}"#);
        });

        let provider = MockProvider {
            url: server.url("/summary.json"),
        };
        let client = reqwest::Client::new();
        let ctx = CheckCtx {
            http_client: &client,
        };
        let outcomes = provider.check(ctx).await.unwrap();

        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].status, CheckStatus::Up);
        assert_eq!(outcomes[0].provider, "TestProvider");
        assert!(outcomes[0].causes.is_empty());
    }

    #[tokio::test]
    async fn check_returns_degraded_with_causes() {
        let server = httpmock::MockServer::start();
        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/summary.json");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"status":{"indicator":"minor"},"incidents":[{"name":"Slow API","status":"investigating"}]}"#);
        });

        let provider = MockProvider {
            url: server.url("/summary.json"),
        };
        let client = reqwest::Client::new();
        let ctx = CheckCtx {
            http_client: &client,
        };
        let outcomes = provider.check(ctx).await.unwrap();

        assert_eq!(outcomes[0].status, CheckStatus::Degraded);
        assert_eq!(outcomes[0].causes, vec!["Slow API (investigating)"]);
    }

    #[tokio::test]
    async fn check_returns_error_on_http_failure() {
        let server = httpmock::MockServer::start();
        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/summary.json");
            then.status(503);
        });

        let provider = MockProvider {
            url: server.url("/summary.json"),
        };
        let client = reqwest::Client::new();
        let ctx = CheckCtx {
            http_client: &client,
        };
        assert!(provider.check(ctx).await.is_err());
    }
}
