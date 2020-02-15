use std::fmt::{self, Write as _};

/// Magic on Iterator(s)
pub trait MagicIter: Iterator {
    fn join(&mut self, seperator: impl ToString) -> String
    where
        Self::Item: fmt::Display,
    {
        match self.next() {
            None => String::new(),
            Some(first) => {
                let sep = seperator.to_string();
                let (lower, _) = self.size_hint();
                let mut result = String::with_capacity(sep.len() * lower);

                write!(&mut result, "{}", first).ok();

                for n in self {
                    result.push_str(&sep);
                    write!(&mut result, "{}", n).ok();
                }

                result
            }
        }
    }
}

impl<T: ?Sized> MagicIter for T where T: Iterator {}

/// Try to mimic the nightly feature on the std library
/// This way we don't have to use nightly for this only feature
pub trait MagicBool {
    fn then_some<T>(self, value: T) -> Option<T>;
    fn then<T, F: FnOnce() -> T>(self, f: F) -> Option<T>;
}

impl MagicBool for bool {
    #[inline]
    fn then_some<T>(self, value: T) -> Option<T> {
        if self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    fn then<T, F: FnOnce() -> T>(self, f: F) -> Option<T> {
        if self {
            Some(f())
        } else {
            None
        }
    }
}

pub trait MagicStr {
    /// A shortcut for `s.chars().nth(index)`
    fn get(&self, index: usize) -> Option<char>;
    /// A shortcut for `s.chars().count()`
    /// This should be replacement for `s.len()` in mose case scenario
    fn count(&self) -> usize;
    /// return `None` if the string is empty
    fn to_option(&self) -> Option<String>;
}

impl MagicStr for str {
    #[inline]
    fn get(&self, index: usize) -> Option<char> {
        self.chars().nth(index)    
    }
    
    #[inline]
    fn count(&self) -> usize {
        self.chars().count()
    }
    
    fn to_option(&self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some(self.to_owned())
        }
    }
}

pub trait MagicOption<T> {
    fn extend_inner<U>(&mut self, value: U)
    where
        T: Default + Extend<U>;
}

impl<T> MagicOption<T> for Option<T> {
    fn extend_inner<U>(&mut self, value: U)
    where
        T: Default + Extend<U>,
    {
        match self {
            Some(ref mut v) => v.extend(Some(value)),
            None => {
                let mut data = T::default();
                data.extend(Some(value));
                *self = Some(data);
            }
        }
    }
}
