#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ProviderDetails {
    pub name: &'static str,
    pub kind: ProviderKind,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ProviderKind {
    GitHub,
    Slack,
    Atlassian,
    CircleCI,
    Cloudflare,
    Datadog,
    Discord,
    Linear,
    Netlify,
    Npm,
    OpenAI,
    Vercel,
    Aws,
}

pub static PROVIDER_REGISTRY: phf::Map<&'static str, ProviderDetails> = phf::phf_map! {
    "github" => ProviderDetails { name: "GitHub", kind: ProviderKind::GitHub },
    "slack" => ProviderDetails { name: "Slack", kind: ProviderKind::Slack },
    "atlassian" => ProviderDetails { name: "Atlassian", kind: ProviderKind::Atlassian },
    "circleci" => ProviderDetails { name: "CircleCI", kind: ProviderKind::CircleCI },
    "cloudflare" => ProviderDetails { name: "Cloudflare", kind: ProviderKind::Cloudflare },
    "datadog" => ProviderDetails { name: "Datadog", kind: ProviderKind::Datadog },
    "discord" => ProviderDetails { name: "Discord", kind: ProviderKind::Discord },
    "linear" => ProviderDetails { name: "Linear", kind: ProviderKind::Linear },
    "netlify" => ProviderDetails { name: "Netlify", kind: ProviderKind::Netlify },
    "npm" => ProviderDetails { name: "npm", kind: ProviderKind::Npm },
    "openai" => ProviderDetails { name: "OpenAI", kind: ProviderKind::OpenAI },
    "vercel" => ProviderDetails { name: "Vercel", kind: ProviderKind::Vercel },
    "aws" => ProviderDetails { name: "AWS", kind: ProviderKind::Aws },
};
