use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{borrow::Borrow, fmt, ops::Deref};

/// A compact string with unique ownership and no reference count.
#[repr(transparent)]
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniqueString(byteyarn::Yarn);
impl UniqueString {
    pub fn from_string(value: String) -> Self {
        Self(byteyarn::Yarn::from_string(value))
    }
    pub const fn from_static_str(value: &'static str) -> Self {
        Self(byteyarn::Yarn::from_static(value))
    }
    pub fn from_fmt(args: fmt::Arguments<'_>) -> Self {
        Self(byteyarn::Yarn::from_fmt(args))
    }
}
impl Deref for UniqueString {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.as_str()
    }
}
impl AsRef<str> for UniqueString {
    fn as_ref(&self) -> &str {
        self
    }
}
impl Borrow<str> for UniqueString {
    fn borrow(&self) -> &str {
        self
    }
}
impl fmt::Display for UniqueString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
impl fmt::Debug for UniqueString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}
impl Serialize for UniqueString {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self)
    }
}
impl<'de> Deserialize<'de> for UniqueString {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d).map(Self::from_string)
    }
}
/// Format a compact string.
#[macro_export]
macro_rules! uformat { ($($arg:tt)*) => { $crate::strings::UniqueString::from_fmt(format_args!($($arg)*)) }; }
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn constructors_and_layout() {
        let s = UniqueString::from_static_str("a static string longer than the inline capacity");
        assert_eq!(
            s.as_ptr(),
            "a static string longer than the inline capacity".as_ptr()
        );
        assert_eq!(&*UniqueString::from_string("hello".into()), "hello");
        assert_eq!(&*uformat!("{} {}", "hello", 42), "hello 42");
        assert_eq!(
            serde_json::to_string(&s).unwrap(),
            serde_json::to_string(&*s).unwrap()
        );
        assert_eq!(
            serde_json::from_str::<UniqueString>("\"hello\"")
                .unwrap()
                .as_ref(),
            "hello"
        );
        assert_eq!(
            std::mem::size_of::<Option<UniqueString>>(),
            std::mem::size_of::<UniqueString>()
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(std::mem::size_of::<UniqueString>(), 16);
    }
}
