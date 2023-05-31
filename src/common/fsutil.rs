#![allow(dead_code)]
#![allow(unused_imports)]

use std::borrow::Borrow;
use std::ffi::{ OsStr, OsString };
// use std::borrow::Borrow;
// use std::cmp::{ Eq, PartialOrd };

use std::io::{self, prelude::*, BufWriter };
use std::fs;
use std::marker::PhantomData;
use std::path::{ Path, PathBuf };
use std::collections::{ HashSet, VecDeque };
use std::hash::Hash;

#[inline(always)]
fn format_token(slice: &str) -> Option<String> {
    let slice = slice
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect::<String>();

    if slice.is_empty() {
        None
    } else {
        Some(slice)
    }
}

#[inline]
pub fn tokenize_file<P>(path: &P) -> Result<Vec<String>, io::Error>
where
    P: AsRef<Path> + ?Sized,
{
    let content = fs::read_to_string(path)?;

    Ok(content
        .split_whitespace()
        .filter_map(format_token)
        .collect::<Vec<String>>()
    )
}

pub enum ExtensionFilter {
    Inclusive(HashSet<OsString>),
    Exclusive(HashSet<OsString>),
    Any,
}

impl ExtensionFilter {
    pub fn build<S>(terms: Vec<S>, inclusive: bool) -> Self
    where
        OsString: From<S>,
    {
        let terms = terms
            .into_iter()
            .map(|s| s.into())
            .collect();


        if inclusive {
            ExtensionFilter::Inclusive(terms)
        } else {
            ExtensionFilter::Exclusive(terms)
        }
    }

    #[inline]
    pub fn verify(&self, val: &OsStr) -> bool {
        match self {
            ExtensionFilter::Inclusive(set) => set.contains(val),
            ExtensionFilter::Exclusive(set) => !set.contains(val),
            _ => true,
        }
    }
}

pub struct FileFinder<'a> {
    canonicalize: bool,
    include_no_ext: bool,
    extensions: &'a ExtensionFilter,
}

impl<'a> Default for FileFinder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl <'a>FileFinder<'a> {
    /// new, all-inclusive searcher, using full paths
    pub fn new() -> Self {
        FileFinder { 
            canonicalize: true,
            include_no_ext: true,
            extensions: &ExtensionFilter::Any,
        }
    }

    #[inline]
    fn valid_extension(&self, path: &Path) -> bool {
        match path.extension() {
            None => self.include_no_ext,
            Some(ext) => self.extensions.verify(ext),
        }
    }

    fn rec_find(&self, dir: PathBuf, results: &mut Vec<PathBuf>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?.path();

            if entry.is_dir() {
                self.rec_find(entry, results)?;
            } else if self.valid_extension(&entry) {
                results.push(entry);
            }
        }

        Ok(())
    }

    #[inline]
    fn queue_find(
        &self, 
        dir: PathBuf, 
        results: &mut Vec<PathBuf>, 
        dirs_sizehint: usize
    ) -> io::Result<()> {
        let mut dir_queue = Vec::with_capacity(dirs_sizehint);
        dir_queue.push(dir);

        while let Some(dir) = dir_queue.pop() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?.path();

                if entry.is_dir() {
                    dir_queue.push(entry);
                } else if self.valid_extension(&entry) {
                    results.push(entry);
                }
            }
        }

        Ok(())
    }

    pub fn execute<P>(
        &self,
        dir: P,
        dirs_sizehint: usize,
        files_sizehint: usize,
    ) -> Result<Vec<PathBuf>, io::Error>
    where
        P: Into<PathBuf>,
    {
        let dir = match self.canonicalize {
            false => dir.into(),
            true => fs::canonicalize(dir.into())?,
        };

        if !dir.is_dir() {
            panic!("path '{:?}' is not a directory", dir);
        }

        let mut results = Vec::with_capacity(files_sizehint);

        if dirs_sizehint > 0 {
            self.queue_find(dir, &mut results, dirs_sizehint)?;
        } else {
            self.rec_find(dir, &mut results)?;
        }

        if (files_sizehint != 0) && (results.len() < results.capacity()) {
            results.shrink_to_fit()
        }

        Ok(results)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filefinder_any() {
        let query = FileFinder {
            canonicalize: false,
            include_no_ext: false,
            extensions: &ExtensionFilter::Any
        };

        match query.execute("./", 100, 100) {
            Ok(paths) => {
                for path in paths.into_iter() {
                    eprintln!("{:?}", path);
                }
            },
            Err(e) => {
                panic!("{:#?}", e);
            }
        }

        match query.execute("./src/", 0, 4) {
            Ok(paths) => {
                for path in paths.into_iter() {
                    eprintln!("{:?}", path);
                }
            },
            Err(e) => {
                panic!("{:#?}", e);
            }
        }
    }

    #[test]
    fn filefinder_extensionfilter() {
        let ext_filter = ExtensionFilter::build(vec!["txt", "rs"], true);

        let query = FileFinder {
            canonicalize: true,
            include_no_ext: false,
            extensions: &ext_filter
        };

        match query.execute("./", 100, 0) {
            Ok(paths) => {
                for path in paths.into_iter() {
                    eprintln!("{:?}", path);
                }
            },
            Err(e) => {
                panic!("{:#?}", e);
            }
        }
    }

    #[test]
    fn tokenize_hamlet() {
        let fp = "./data/hamlet.txt";

        match tokenize_file(&fp) {
            Ok(words) => {
                let space = String::from(" ");

                for s in words.clone() {
                    assert!(!s.contains(&space) && (s != space));
                }

                for i in 0..100 {
                    eprint!("'{}', ", words[i]);
                }

                io::stdout().flush().unwrap()
            }
            Err(e) => panic!("expected vector of words, got: {e}")
        }
    }

    #[test]
    fn tokenize_no_words() {
        let a = tokenize_file("./data/empty_file.txt").unwrap();
        let b = tokenize_file("./data/empty_file.html").unwrap();

        if a.len() != 0 {
            for val in a.iter() {
                println!("{val}");
            }
        }

        if b.len() != 0 {
            for val in b.iter() {
                println!("{val}");
            }
        }

        assert_eq!(a.len(), 0);
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn tokenize_canonicalize() {
        let path = fs::canonicalize("./data/hamlet.txt").unwrap();
        let words = tokenize_file(&path).unwrap();
        let space = String::from(" ");

        for s in words.clone() {
            assert!(!s.contains(&space) && (s != space));
        }
    }
}


/* fn consume_with_relish<F>(func: F)
    where F: FnOnce() -> String
{
    // `func` consumes its captured variables, so it cannot be run more
    // than once.
    println!("Consumed: {}", func());

    println!("Delicious!");

    // Attempting to invoke `func()` again will throw a `use of moved
    // value` error for `func`.
}

fn consume_fn() {
    let x = String::from("x");
    let consume_and_return_x = move || x;
    consume_with_relish(consume_and_return_x);
} */

/* pub trait VerifyComponent<T>{
    fn verify_component(&self, s: T) -> bool;
}

impl <C>VerifyComponent<C> for ExtensionFilter<C>
where
    C: AsRef<OsStr> + Hash + Eq,
{
    fn verify_component(&self, s: C) -> bool {
        self.terms.contains(&s) == self.inclusive
    }
} */


/* #[derive(Debug, PartialEq, /* Clone, */ )]
enum FilterState {
    Inactive,
    Inclusive(bool)
}

impl FilterState {
    fn val(&self) -> bool {
        match *self {
            FilterState::Inactive => true,
            FilterState::Inclusive(b) => b,
        }
    }
}

impl From<bool> for FilterState {
    fn from(val: bool) -> Self {
        FilterState::Inclusive(val)
    }
}

impl From<Option<bool>> for FilterState {
    fn from(item: Option<bool>) -> Self {
        match item {
            None => FilterState::Inactive,
            Some(val) => FilterState::from(val),
        }
    }
} */
