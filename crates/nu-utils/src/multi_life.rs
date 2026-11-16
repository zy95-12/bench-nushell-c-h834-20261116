use std::ops::Deref;

/// A reference from either an outer or a shorter local lifetime.
pub enum MultiLife<'out, 'local, T>
where
    'out: 'local,
    T: ?Sized,
{
    Out(&'out T),
    Local(&'local T),
}

impl<'out, 'local, T> Deref for MultiLife<'out, 'local, T>
where
    'out: 'local,
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match *self {
            MultiLife::Out(x) => x,
            MultiLife::Local(x) => x,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn local<'out: 'local, 'local>(outer: &'out str, inner: &'local str) {
        let a: MultiLife<'out, 'local, str> = MultiLife::Out(outer);
        let b: MultiLife<'out, 'local, str> = MultiLife::Local(inner);
        assert!(std::ptr::eq(&*a, outer));
        assert!(std::ptr::eq(&*b, inner));
    }
    #[test]
    fn unsized_deref() {
        local("static", &String::from("local"));
    }
}
