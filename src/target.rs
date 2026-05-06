use crate::registry::{PROVIDER_REGISTRY, ProviderDetails};

#[derive(Debug)]
pub enum Target {
    Provider(ProviderDetails),
    Url(String),
    
}

impl Target {
    pub fn parse(target: &str) -> Option<Target> {
        let key = target.trim().to_lowercase();
        let result = PROVIDER_REGISTRY
            .get(key.as_str())
            .map(|details| Target::Provider(*details));
        
        result.or_else(|| {
            if key.starts_with("http") || key.starts_with("https") {
                Some(Target::Url(target.to_string()))
            } else {
                None
            }
        });
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ProviderKind;

    #[test]
    fn parse_url() {
        let Target::Url(url) = Target::parse("https://example.com").unwrap();
        assert_eq!(url, "https://example.com");
    }
    
    #[test]
    fn parse_known_provider() {
        let Target::Provider(d) = Target::parse("github").unwrap();
        assert_eq!(d.kind, ProviderKind::GitHub);
    }

    #[test]
    fn parse_case_insensitive() {
        let Target::Provider(d) = Target::parse("GitHub").unwrap();
        assert_eq!(d.kind, ProviderKind::GitHub);
        let Target::Provider(d) = Target::parse("GITHUB").unwrap();
        assert_eq!(d.kind, ProviderKind::GitHub);
    }

    #[test]
    fn parse_trims_whitespace() {
        let Target::Provider(d) = Target::parse("  aws  ").unwrap();
        assert_eq!(d.kind, ProviderKind::Aws);
    }

    #[test]
    fn parse_unknown_returns_none() {
        assert!(Target::parse("unknown_provider").is_none());
        assert!(Target::parse("").is_none());
    }
}
