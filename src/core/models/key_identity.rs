/// Represents an authorized recipient (public key) that can
/// decrypt secrets encrypted by Vaultic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyIdentity {
    pub public_key: String,
    pub label: Option<String>,
    pub added_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl std::fmt::Display for KeyIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.label {
            Some(label) => write!(f, "{} ({})", self.public_key, label),
            None => write!(f, "{}", self.public_key),
        }
    }
}
