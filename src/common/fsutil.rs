#![allow(clippy::len_zero)]
#![allow(unused_imports)]
// use smartstring::alias::String;
use std::io::{self, /* BufReader */};
use std::fs::{self, /* File */};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
// use std::collections::{HashSet, /* VecDeque */};
// use rustc_hash::{FxHashMap, FxHashSet};
use ahash::{AHashMap, AHashSet};

/* #[inline(always)]
fn format_token(slice: &str) -> Option<String> {
    let slice = slice
        .chars()
        .filter(|c| c.is_ascii())
        .map(|c| c.to_ascii_lowercase())
        .collect::<String>();

    if slice.is_empty() {
        None
    } else {
        Some(slice)
    }
} */

#[inline(always)]
fn format_token(slice: &str) -> Option<String> {
    let slice = slice
        .as_bytes()
        .iter()
        .filter(|c| c.is_ascii())
        .map(|c| c.to_ascii_lowercase())
        .collect::<Vec<u8>>();

    if slice.len() > 0 {
        // we know with 100% certainty content is utf8 at this point
        // read_to_string would otherwise have panicked, etc, etc
        unsafe { Some(String::from_utf8_unchecked(slice)) }
    } else {
        None
    }
}

#[inline]
pub fn tokenize_file<P, T>(path: &P) -> Result<T, io::Error>
where
    T: FromIterator<String>,
    P: AsRef<Path> + ?Sized,
{
    /* let f = File::open(path)?;
    let fsize = f.metadata().unwrap().len();
    eprintln!("file: {:#?}; size: {:#?}", f, fsize);
    let reader = BufReader::with_capacity(fsize as usize, f);
    eprintln!("reader: {:#?}", reader); */

    let content = fs::read_to_string(path)?;

    Ok(content
        .split_whitespace()
        .filter_map(format_token)
        .collect::<T>()
    )
}

pub enum ExtensionFilter {
    Inclusive(AHashSet<OsString>),
    Exclusive(AHashSet<OsString>),
    AllInclusive,
}

#[allow(dead_code)]
impl ExtensionFilter {
    pub fn build<S>(terms: Vec<S>, inclusive: bool) -> Self
    where
        OsString: From<S>,
    {
        let terms = terms
            .into_iter()
            .map(|s| s.into())
            .collect::<AHashSet<OsString>>();

        if inclusive {
            Self::Inclusive(terms)
        } else {
            Self::Exclusive(terms)
        }
    }

    pub fn into_inner(self) -> AHashSet<OsString> {
        match self {
            Self::AllInclusive => panic!("self has no inner set"),
            Self::Exclusive(set) => set,
            Self::Inclusive(set) => set,
        }
    }

    pub fn make_exclusive(self) -> Self {
        Self::Exclusive(self.into_inner())
    }

    pub fn make_inclusive(self) -> Self {
        Self::Inclusive(self.into_inner())
    }

    #[inline(always)]
    pub fn verify(&self, val: &OsStr) -> bool {
        match self {
            Self::Inclusive(set) => set.contains(val),
            Self::Exclusive(set) => !set.contains(val),
            Self::AllInclusive => true,
        }
    }
}

impl <T: Into<OsString>> From<Vec<T>> for ExtensionFilter {
    fn from(terms: Vec<T>) -> Self {
        let terms = terms
            .into_iter()
            .map(|s| s.into())
            .collect();

        Self::Inclusive(terms)
    }
}

pub struct FileFinder {
    canonicalize: bool,
    include_no_ext: bool,
    filter: ExtensionFilter,
}

#[allow(dead_code)]
impl FileFinder {
    /// see default
    pub fn new() -> Self {
        Self::default()
    }

    fn build(canonicalize: bool, include_no_ext: bool, filter: ExtensionFilter) -> Self {
        Self { canonicalize, include_no_ext, filter }
    }

    /// consumes/unwraps self, returning the filter
    fn into_inner(self) -> ExtensionFilter {
        self.filter
    }

    #[inline]
    fn valid_extension(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            self.filter.verify(ext)
        } else {
            self.include_no_ext
        }
        /* match path.extension() {
            None => self.include_no_ext,
            Some(ext) => self.filter.verify(ext),
        } */
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

    /* #[inline]
    fn queue_find(
        &self, 
        dir: PathBuf, 
        results: &mut Vec<PathBuf>, 
        sizehint_dirs: usize
    ) -> io::Result<()> 
    {
        let mut dir_queue = Vec::with_capacity(sizehint_dirs);
        dir_queue.push(dir);

        let mut i = 0;

        while dir_queue.len() > i {
            for entry in fs::read_dir(&dir_queue[i])? {
                let entry = entry?.path();

                if entry.is_dir() {
                    dir_queue.push(entry);
                } else if self.valid_extension(&entry) {
                    results.push(entry);
                }
            }
            i += 1;
        }

        // eprintln!("n_dirs = {}", dir_queue.len());

        Ok(())
    } */

    pub fn execute<P: Into<PathBuf>>(
        &self,
        dir: P,
        sizehint_dirs: usize,
        sizehint_files: usize,
    ) -> Result<Vec<PathBuf>, io::Error>
    {
        let dir = match self.canonicalize {
            false => dir.into(),
            true => fs::canonicalize(dir.into())?,
        };

        if !dir.is_dir() {
            panic!("path '{:?}' is not a directory", dir);
        }

        let mut results = Vec::with_capacity(sizehint_files);
        let mut dir_queue = Vec::with_capacity(sizehint_dirs);
        dir_queue.push(dir);

        let mut i = 0;
        while dir_queue.len() > i {
            for entry in fs::read_dir(&dir_queue[i])? {
                let entry = entry?.path();

                if entry.is_dir() {
                    dir_queue.push(entry);
                } else if self.valid_extension(&entry) {
                    results.push(entry);
                }
            }
            i += 1;
        }

        // eprintln!("n_dirs = {}", dir_queue.len());

        // if sizehint_dirs > 0 {
        //     self.queue_find(dir, &mut results, sizehint_dirs)?;
        // } else {
        //     self.rec_find(dir, &mut results)?;
        // }

        // shrink if vec is oversized
        if (sizehint_files != 0) && (results.len() < results.capacity()) {
            results.shrink_to_fit()
        }

        Ok(results)
    }
}

impl Default for FileFinder {
    /// default file finder, canonicalizes paths
    /// includes all files except for those with no extension
    fn default() -> Self {
        Self::build(true, false, ExtensionFilter::AllInclusive)
    }
}

impl From<ExtensionFilter> for FileFinder {
    /// convenience function, like default but with a filter
    fn from(filter: ExtensionFilter) -> Self {
        Self::build(true, false, filter)
    }
}

impl <T: Into<OsString>> From<Vec<T>> for FileFinder {
    /// convenience function, like default but with a filter built from a vector
    fn from(terms: Vec<T>) -> Self {
        Self::build(true, false, ExtensionFilter::from(terms))
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
        // let finder = FileFinder::from(vec!["rs", "txt", "html", "c", "js", "json", "py"]);
        let finder = FileFinder::default();
        let paths = finder.execute("/users/odin/code/", 2000, 10000).expect(&format!("err: "));
        let mut errors: Vec<io::Error> = vec![];

        let mut n_words: usize = 0;
        let mut n_files: usize = paths.len();

        for path in paths.into_iter() {
            match tokenize_file::<_, Vec<_>>(&path) {
                Ok(tokens) => {
                    n_words += tokens.len();
                    // printdb!(&format!("{:?}", path), tokens.len());
                }
                Err(e) => errors.push(e)
            }
        }

        eprintln!("n_words={}, n_files={}", n_words, n_files);

        // log errors
        if errors.len() != 0 {
            let mut utf_8_errors = 0;

            for (_n, e) in errors.into_iter().enumerate() {
                if e.kind() == io::ErrorKind::InvalidData {
                    utf_8_errors += 1;
                } else {
                    printerr!(&format!("err {_n}:"), e);
                }
            }
            printerr!("", utf_8_errors);
            panic!();
        }
    }

    #[test]
    fn filefinder_empty_dir() {
        let finder = FileFinder::default();

        match finder.execute("./empty/", 100, 100) {
            Ok(paths) => for path in paths.into_iter() {
                eprintln!("{:?}", path);
            },
            Err(e) => panic!("{:#?}", e),
        }
    }

    #[test]
    fn filefinder_allinclusive() {
        let finder = FileFinder::default();

        match finder.execute("./", 100, 100) {
            Ok(paths) => for path in paths.into_iter() {
                eprintln!("{:?}", path);
            },
            Err(e) => panic!("{:#?}", e),
        }
    }

    #[test]
    fn filefinder_extensionfilter() {
        let s = "dkjwak".to_owned();

        let query = FileFinder {
            canonicalize: true,
            include_no_ext: false,
            filter: ExtensionFilter::from(vec!["txt", "rs"]),
        };

        match query.execute("./", 100, 0) {
            Ok(paths) => for path in paths.into_iter() {
                eprintln!("{:?}", path);
            },
            Err(e) => panic!("{:#?}", e),
        }
    }

    #[test]
    fn tokenize_hamlet() {
        let path = fs::canonicalize("./data/hamlet.txt").unwrap();
        let tokenized: Result<Vec<_>, _> = tokenize_file(&path);

        match tokenized {
            Ok(words) => for s in words.into_iter() {
                assert!(!s.contains(' ') && (s != " "));
            },
            Err(e) => panic!("expected vector of words, got: {e}")
        }
    }

    #[test]
    fn tokenize_wordless_files() {
        let a: Vec<_> = tokenize_file("./data/empty_file.txt").unwrap();
        let b: Vec<_> = tokenize_file("./data/empty_file.html").unwrap();
        assert!(a.is_empty());
        assert!(b.is_empty());
    }
}
