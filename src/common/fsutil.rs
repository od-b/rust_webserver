#![allow(clippy::len_zero)]
#![allow(unused_imports)]
use std::io::{self/*, BufReader, ErrorKind */};
use std::fs::{self, /* File */};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
// use ahash::{AHashMap, HashSet};

#[inline(always)]
fn format_token(slice: &str) -> Option<String> {
    let slice = slice
        .as_bytes()
        .iter()
        .filter(|c| c.is_ascii())
        .map(|c| c.to_ascii_lowercase())
        .collect::<Vec<u8>>();

    if !slice.is_empty() {
        // SAFE: already checked for valid utf-8 on mass read
        unsafe {
            Some(String::from_utf8_unchecked(slice))
        }
    } else {
        None
    }
}

#[inline]
pub fn tokenize_file<P, T>(path: &P) -> Result<T, io::Error>
where
    P: AsRef<Path> + ?Sized,
    T: FromIterator<String>,
{
    let content = fs::read_to_string(path)?;

    Ok(content
        .split_ascii_whitespace()
        .filter_map(format_token)
        .collect::<T>()
    )
}

pub enum OsStringFilter {
    Inclusive(HashSet<OsString>),
    Exclusive(HashSet<OsString>),
    AllInclusive,
}

impl OsStringFilter {
    pub fn build<S: Into<OsString>>(terms: Vec<S>, inclusive: bool) -> Self {
        let terms = terms
            .into_iter()
            .map(|s| s.into())
            .collect::<HashSet<OsString>>();

        if inclusive {
            Self::Inclusive(terms)
        } else {
            Self::Exclusive(terms)
        }
    }

    /// unwraps this filter, returning the underlying HashSet
    /// panics if filter is AllInclusive
    pub fn into_inner(self) -> HashSet<OsString> {
        match self {
            Self::AllInclusive => panic!("self has no inner set"),
            Self::Exclusive(set) => set,
            Self::Inclusive(set) => set,
        }
    }

    #[inline(always)]
    pub fn validate(&self, val: &OsStr) -> bool {
        match self {
            Self::Inclusive(set) => set.contains(val),
            Self::Exclusive(set) => !set.contains(val),
            Self::AllInclusive => true,
        }
    }
}

impl <T: Into<OsString>> From<Vec<T>> for OsStringFilter {
    fn from(terms: Vec<T>) -> Self {
        let terms = terms
            .into_iter()
            .map(|s| s.into())
            .collect();

        Self::Inclusive(terms)
    }
}

pub struct FileFinder {
    include_no_ext: bool,
    fmt_realpath: bool,
    filter: OsStringFilter,
}

#[allow(dead_code)]
impl FileFinder {
    pub fn new() -> Self {
        Self::default()
    }

    fn build(
        include_no_ext: bool,
        fmt_realpath: bool,
        filter: OsStringFilter
    ) -> Self {
        Self { include_no_ext, fmt_realpath, filter }
    }

    #[inline(always)]
    fn valid_extension(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            self.filter.validate(ext)
        } else {
            self.include_no_ext
        }
    }

    fn _search_recur(&self, dir: &PathBuf, results: &mut Vec<PathBuf>) -> io::Result<()> {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?.path();

            if entry.is_dir() {
                self._search_recur(&entry, results)?;
            } else if self.valid_extension(&entry) {
                results.push(entry);
            }
        }

        Ok(())
    }

    pub fn search_recur<P: AsRef<Path>>(
        &self,
        dir: P,
        hint_n_files: usize,
    ) -> Result<Vec<PathBuf>, io::Error>
    {
        let dir = match self.fmt_realpath {
            false => PathBuf::from(dir.as_ref()),
            true => fs::canonicalize(dir)?
        };

        let mut results: Vec<PathBuf> = Vec::with_capacity(hint_n_files);
        self._search_recur(&dir, &mut results)?;
        // results.shrink_to_fit();

        Ok(results)
    }

    /// search for files iteratively
    pub fn search<P: AsRef<Path>>(
        &self,
        dir: P,
        hint_n_files: usize,
        hint_n_dirs: usize,
    ) -> Result<Vec<PathBuf>, io::Error>
    {
        let dir = match self.fmt_realpath {
            false => PathBuf::from(dir.as_ref()),
            true => fs::canonicalize(dir)?
        };

        let mut results = Vec::with_capacity(hint_n_files);
        let mut dir_queue = Vec::with_capacity(1 + hint_n_dirs);
        dir_queue.push(dir);

        let mut i: usize = 0;
        while dir_queue.len() > i {
            let curr_dir = fs::read_dir(&dir_queue[i])?;
            for entry in curr_dir {
                let entry = entry?.path();

                if entry.is_dir() {
                    dir_queue.push(entry);
                } else if self.valid_extension(&entry) {
                    results.push(entry);
                }
            }
            i += 1;
        }
        // results.shrink_to_fit();

        Ok(results)
    }
}

impl Default for FileFinder {
    /// Will include all files that has an extension. Canonicalizes paths.
    fn default() -> Self {
        Self::build(false, true, OsStringFilter::AllInclusive)
    }
}

impl From<OsStringFilter> for FileFinder {
    fn from(filter: OsStringFilter) -> Self {
        Self::build(false, true, filter)
    }
}

impl <T: Into<OsString>> From <Vec<T>> for FileFinder {
    fn from(terms: Vec<T>) -> Self {
        Self::build(false, true, OsStringFilter::from(terms))
    }
}




// ----------- tests -----------

#[cfg(test)]
#[allow(dead_code, unused_variables, unused_mut)]
mod tests {
    #[allow(unused_imports)]
    use crate::{ printdb, printdbf, printerr };
    use super::*;

    #[test]
    fn find_and_tokenize() {
        // find files to tokenize
        let ext_targets = vec!["rs", "txt", "html", "c", "js", "json", "py"];
        let finder = FileFinder::from(ext_targets);
        let paths = finder
            .search("/users/odin/code/", 5000, 2000)
            .expect(&format!("err: "));

        // misc extra info
        let mut errors: Vec<io::Error> = vec![];
        let mut n_words: usize = 0;
        let mut n_files: usize = paths.len();

        // tokenize all files
        for path in paths.into_iter() {
            let tokens: Result<Vec<_>, _> = tokenize_file(&path);
            match tokens {
                Ok(tokens) => {
                    n_words += tokens.len();
                    // print filepath + tokenized words from file
                    printdb!(&format!("{:?}", path), tokens.len());
                }
                Err(e) => errors.push(e)
            }
        }

        // if any errors, log errors
        if errors.len() != 0 {
            let mut utf_8_errors = 0;

            for (i, err) in errors.into_iter().enumerate() {
                if err.kind() == io::ErrorKind::InvalidData {
                    utf_8_errors += 1;
                } else {
                    printerr!(&format!("err {i}:"), err);
                }
            }
            printerr!("", utf_8_errors);

            if n_words != 0 {
                eprintln!("succeeded for: n_words={}, n_files={}", n_words, n_files);
            } else {
                eprintln!("no words were tokenized");
            }
            panic!("errors found");
        }
        eprintln!("No errors. tokenized total: n_words={}, n_files={}", n_words, n_files);
    }

    #[test]
    fn search_nofilter() {
        let finder = FileFinder::build(true, false, OsStringFilter::AllInclusive);
        let dir = "./";

        match finder.search(&dir, 1300, 100) {
            Ok(paths) => eprintln!("found {} paths from '{}'", paths.len(), dir),
            Err(e) => panic!("{:#?}", e),
        }
    }

    #[test]
    fn search_usr_code() {
        let finder = FileFinder::default();
        let dir = "/Users/odin/code/";

        match finder.search(&dir, 10000, 1000) {
            Ok(paths) => eprintln!("found {} paths from '{}'", paths.len(), dir),
            Err(e) => panic!("{:#?}", e),
        }
    }

    #[test]
    fn search_usr_code_recursive() {
        let finder = FileFinder::default();
        let dir = "/Users/odin/code/";

        match finder.search_recur(dir, 10000) {
            Ok(paths) => eprintln!("found {} paths from '{}'", paths.len(), dir),
            Err(e) => panic!("{:#?}", e),
        }
    }

    #[test]
    fn tokenize_hamlet() {
        let path = fs::canonicalize("./static/data/hamlet.txt").unwrap();
        let tokenized: Result<Vec<_>, _> = tokenize_file(&path);

        match tokenized {
            Ok(words) => for s in words.into_iter() {
                assert!(!s.contains(' ') && (s != " "));
            },
            Err(e) => panic!("expected vector of words, got: {e}")
        }
    }
}

// #[test]
// fn search_empty_dir() {
//     match FileFinder::default().search("./empty/", 100, 100) {
//         Ok(paths) => for path in paths.into_iter() {
//             eprintln!("{:?}", path);
//         },
//         Err(e) => panic!("{:#?}", e),
//     }
// }

// #[test]
// fn tokenize_space_only_files() {
//     let a: Vec<_> = tokenize_file("./data/empty_file.txt").unwrap();
//     let b: Vec<_> = tokenize_file("./data/empty_file.html").unwrap();
//     assert!(a.is_empty());
//     assert!(b.is_empty());
// }