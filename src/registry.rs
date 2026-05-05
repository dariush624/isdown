
pub struct ProviderDetails {
    pub name: &'static str,
    pub status_url: &'static str,
}

pub static PROVIDER_REGISTRY: phf::Map<&'static str, ProviderDetails> = phf::phf_map! {
    "github" => ProviderDetails { name: "GitHub", status_url: "https://status.github.com" },
};