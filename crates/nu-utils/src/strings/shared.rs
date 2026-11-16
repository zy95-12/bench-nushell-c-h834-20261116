use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{borrow::Borrow, fmt, ops::Deref};

/// A compact, immutable string with inexpensive shared clones.
#[repr(transparent)]
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SharedString(lean_string::LeanString);
impl SharedString {
    pub fn from_string(value: String) -> Self {
        Self(lean_string::LeanString::from(value))
    }
    pub const fn from_static_str(value: &'static str) -> Self {
        Self(lean_string::LeanString::from_static_str(value))
    }
    pub fn from_fmt(args: fmt::Arguments<'_>) -> Self {
        use std::fmt::Write;
        let mut value = lean_string::LeanString::new();
        value.write_fmt(args).expect("formatting a string failed");
        Self(value)
    }
}
impl Deref for SharedString {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.as_str()
    }
}
impl AsRef<str> for SharedString {
    fn as_ref(&self) -> &str {
        self
    }
}
impl Borrow<str> for SharedString {
    fn borrow(&self) -> &str {
        self
    }
}
impl fmt::Display for SharedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
impl fmt::Debug for SharedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}
impl Serialize for SharedString {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self)
    }
}
impl<'de> Deserialize<'de> for SharedString {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d).map(Self::from_string)
    }
}
/// Format a compact string.
#[macro_export]
macro_rules! sformat { ($($arg:tt)*) => { $crate::strings::SharedString::from_fmt(format_args!($($arg)*)) }; }
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn constructors_and_layout() {
        let s = SharedString::from_static_str("a static string longer than the inline capacity");
        assert_eq!(
            s.as_ptr(),
            "a static string longer than the inline capacity".as_ptr()
        );
        assert_eq!(&*SharedString::from_string("hello".into()), "hello");
        assert_eq!(&*sformat!("{} {}", "hello", 42), "hello 42");
        assert_eq!(
            serde_json::to_string(&s).unwrap(),
            serde_json::to_string(&*s).unwrap()
        );
        assert_eq!(
            serde_json::from_str::<SharedString>("\"hello\"")
                .unwrap()
                .as_ref(),
            "hello"
        );
        assert_eq!(
            std::mem::size_of::<Option<SharedString>>(),
            std::mem::size_of::<SharedString>()
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(std::mem::size_of::<SharedString>(), 16);
    }
}

#[test]
fn clone_shares_heap_data() {
    let a = SharedString::from_string("x".repeat(100));
    let b = a.clone();
    assert_eq!(a.as_ptr(), b.as_ptr());
}
