use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// A borrowed or owned value whose representations may have different types.
#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum NuCow<B, O> {
    Borrowed(B),
    Owned(O),
}

impl<'de, B, O: Deserialize<'de>> Deserialize<'de> for NuCow<B, O> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        O::deserialize(deserializer).map(Self::Owned)
    }
}

impl<B: PartialEq + PartialEq<O>, O: PartialEq> PartialEq for NuCow<B, O> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Borrowed(a), Self::Borrowed(b)) => a == b,
            (Self::Owned(a), Self::Owned(b)) => a == b,
            (Self::Borrowed(a), Self::Owned(b)) | (Self::Owned(b), Self::Borrowed(a)) => a == b,
        }
    }
}

impl<B: fmt::Debug, O: fmt::Debug> fmt::Debug for NuCow<B, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alternate = f.alternate();
        match self {
            Self::Borrowed(x) if alternate => f.debug_tuple("Borrowed").field(x).finish(),
            Self::Owned(x) if alternate => f.debug_tuple("Owned").field(x).finish(),
            Self::Borrowed(x) => x.fmt(f),
            Self::Owned(x) => x.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn representation() {
        let borrowed: NuCow<&str, String> = NuCow::Borrowed("hello");
        let owned = NuCow::Owned("hello".to_owned());
        assert_eq!(borrowed, owned);
        assert_eq!(owned, borrowed.clone());
        for value in [borrowed, owned] {
            let json = serde_json::to_string(&value).unwrap();
            assert_eq!(json, "\"hello\"");
            let decoded: NuCow<&str, String> = serde_json::from_str(&json).unwrap();
            assert!(matches!(decoded, NuCow::Owned(_)));
            assert_eq!(format!("{decoded:?}"), "\"hello\"");
            assert!(format!("{decoded:#?}").starts_with("Owned("));
        }
        assert!(format!("{:#?}", NuCow::<_, String>::Borrowed("x")).starts_with("Borrowed("));
    }
}
