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
            return None;
        }

        let current_index = self.current_index;
        
        self.content[current_index..]
            .index_at_nth(self.limit)
            .map(|(limit, _)| current_index + limit)
            .map(|limit| {
                self.content[current_index..limit]
                    .rfind(self.last)
                    .map(|match_index| current_index + match_index + self.last.len())
                    .map(|end_index| {
                        self.current_index = end_index;
                        &self.content[current_index..end_index]
                    })
            })
            .unwrap_or_else(|| {
                self.current_index = self.content.len();
                Some(&self.content[current_index..])
            })

    }
}

pub trait MagicStr {
    /// A shortcut for `s.chars().nth(index)`
    fn get_char(&self, nth: usize) -> Option<char>;
    
    /// A shortcut for `s.chars().count()`
    /// This should be replacement for `s.len()` in mose case scenario
    fn count(&self) -> usize;
    
    /// Get index at the nth char
    /// Return the index of begin and end of that char
    /// the different will be `1` unless that's some weird character e.g `漢字`
    fn index_at_nth(&self, nth: usize) -> Option<(usize, usize)>;
    
    /// Split a content at a specific length limit
    /// This will not remove the char like the `str::split` method
    /// Will yield an iterator of &str, it will end when it reach the end of str
    /// or cannot split the str with the desired limit.
    fn split_at_limit<'a>(&'a self, limit: usize, last: &'a str) -> SplitAtLimit<'a>;
    
    /// return `None` if the string is empty
    fn to_option(&self) -> Option<&str>;
}

impl MagicStr for str {
    #[inline]
    fn get_char(&self, nth: usize) -> Option<char> {
        self.chars().nth(nth)
    }

    #[inline]
    fn count(&self) -> usize {
        self.chars().count()
    }
    
    #[inline]
    fn index_at_nth(&self, nth: usize) -> Option<(usize, usize)> {
        self.char_indices().nth(nth).map(|(i, c)| (i, i + c.len_utf8()))
    }

    fn split_at_limit<'a>(&'a self, limit: usize, last: &'a str) -> SplitAtLimit<'a> {
        SplitAtLimit {
            content: self,
            current_index: 0,
            limit,
            last,
        }
    }

    #[inline]
    fn to_option(&self) -> Option<&str> {
        if self.is_empty() { None } else { Some(self) }
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
