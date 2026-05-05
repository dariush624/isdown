use crate::registry::{ProviderDetails, PROVIDER_REGISTRY};


#[derive(Debug)]
pub enum Target {
    Provider(ProviderDetails),
}

impl Target {
    pub fn parse(target: &str) -> Option<Target> {
        let key = target.trim().to_lowercase();
        PROVIDER_REGISTRY.get(key.as_str()).map(|details| {
            Target::Provider(*details)
        })
    }
}