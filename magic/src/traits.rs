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

pub struct SplitAtLimit<'a> {
    content: &'a str,
    limit: usize,
    last: &'a str,
    current_index: usize,
}

impl<'a> Iterator for SplitAtLimit<'a> {
    type Item = &'a str;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.content.len() == self.current_index {
            return None
        }
        
        let current_index = self.current_index;
        let limit = current_index + self.limit;
        
        if self.content.len() <= limit {
            self.current_index = self.content.len();
            return Some(&self.content[current_index..])
        }
        
        self.content[current_index..limit]
            .rfind(self.last)
            .map(|match_index| match_index + self.last.len())
            .map(|end_index| {
                self.current_index = end_index;
                &self.content[current_index..end_index]
            })
    }
}

pub trait MagicStr {
    /// A shortcut for `s.chars().nth(index)`
    fn get(&self, index: usize) -> Option<char>;
    /// A shortcut for `s.chars().count()`
    /// This should be replacement for `s.len()` in mose case scenario
    fn count(&self) -> usize;
    /// Split a content at a specific length limit
    /// This will not remove the char like the `str::split` method
    /// Will yield an iterator of String
    fn split_at_limit<'a>(&'a self, limit: usize, last: &'a str) -> SplitAtLimit<'a>;
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
    
    fn split_at_limit<'a>(&'a self, limit: usize, last: &'a str) -> SplitAtLimit<'a> {
        SplitAtLimit {
            content: self,
            current_index: 0,
            limit,
            last,
        }
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
