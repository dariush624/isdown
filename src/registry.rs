#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ProviderDetails {
    pub name: &'static str,
    pub kind: ProviderKind,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ProviderKind {
    GitHub,
}

pub static PROVIDER_REGISTRY: phf::Map<&'static str, ProviderDetails> = phf::phf_map! {
    "github" => ProviderDetails { name: "GitHub", kind: ProviderKind::GitHub },
};
