use crate::check::{CheckError, CheckStatus, ProviderCheck};
use serde_json::Value;

macro_rules! statuspage_provider {
    ($struct_name:ident, $name:literal, $host:literal) => {
        pub struct $struct_name;

        impl ProviderCheck for $struct_name {
            fn provider(&self) -> &'static str {
                $name
            }

            fn url(&self) -> &'static str {
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
