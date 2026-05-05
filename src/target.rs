use crate::registry::{ProviderDetails, PROVIDER_REGISTRY};

pub enum Target {
    Provider(ProviderDetails), // TODO: It should contain the details from a registry
}

impl Target {
    pub fn parse(target: &str) -> Option<Target> {
        let key = target.trim().to_lowercase();
        PROVIDER_REGISTRY.get(key.as_str()).map(|details| {
            Target::Provider(ProviderDetails {
                name: details.name,
                status_url: details.status_url
            })
        })
    }
}