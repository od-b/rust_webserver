// #![allow(unused_imports)]
use std::io::{ self, /* BufReader */ };
use std::fs::{ self, /* File */ };
use std::ffi::{ OsStr, OsString };
use std::path::{ Path, PathBuf };
use std::collections::{ HashSet, /* VecDeque */ };

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
pub fn tokenize_file<P, T>(path: &P) -> Result<T, io::Error>
where
    T: FromIterator<String>,
    P: AsRef<Path> + ?Sized,
{
    // let f = File::open(path)?;
    // let fsize = f.metadata().unwrap().len();
    // eprintln!("file: {:#?}; size: {:#?}", f, fsize);
    // let reader = BufReader::with_capacity(fsize as usize, f);
    // eprintln!("reader: {:#?}", reader);

    let content = fs::read_to_string(path)?;

    Ok(content
        .split_whitespace()
        .filter_map(format_token)
        .collect::<T>()
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

pub struct FileFinder {
    canonicalize: bool,
    include_no_ext: bool,
    extensions: ExtensionFilter,
}

impl Default for FileFinder {
    /// returns a filefinder that will include any/all files
    fn default() -> Self {
        FileFinder { 
            canonicalize: true,
            include_no_ext: true,
            extensions: ExtensionFilter::Any,
        }
    }
}

#[allow(dead_code)]
impl FileFinder {
    /// same as FileFinder::default()
    pub fn new() -> Self {
        Self::default()
    }

    /// unwraps the extensionfilter of self
    fn into_inner(self) -> ExtensionFilter {
        self.extensions
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
    ) -> io::Result<()> 
    {
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
#[allow(dead_code, unused_variables, unused_mut)]
mod tests {
    #[allow(unused_imports)]
    use crate::{ printdb, printdbf, printerr };
    use super::*;

    #[test]
    fn find_and_tokenize() {
        let finder = FileFinder {
            canonicalize: false,
            include_no_ext: false,
            extensions: ExtensionFilter::Any,
            // extensions: ExtensionFilter::build(vec!["rs", "txt", "html"], true),
        };

        let paths = finder.execute("./", 100, 100)
            .expect(&format!("ah: {}", "nuts"));

        let mut errors: Vec<io::Error> = vec![];

        for path in paths.into_iter() {
            let result: Result<Vec<_>, _> = tokenize_file(&path);

            // if result.is_err() {
            //     errors.push(result.unwrap_err());
            // } /* else {
            //     printdb!("", path, words.len());
            // } */
        }

        // log errors
        if !errors.is_empty() {
            let mut utf_8_errors = 0;

            for (_n, e) in errors.into_iter().enumerate() {
                if e.kind() == io::ErrorKind::InvalidData {
                    utf_8_errors += 1;
                    printerr!(&format!("err {_n}:"), e);
                } else {
                    // let msg = format!("err {n}:");
                    printerr!(&format!("err {_n}:"), e);
                }
            }

            printerr!("", utf_8_errors);
            panic!();
        }
    }

    #[test]
    fn filefinder_empty_dir() {
        let finder = FileFinder {
            canonicalize: true,
            include_no_ext: true,
            extensions: ExtensionFilter::Any
        };

        match finder.execute("./empty/", 100, 100) {
            Ok(paths) => for path in paths.into_iter() {
                eprintln!("{:?}", path);
            },
            Err(e) => panic!("{:#?}", e),
        }
    }

    #[test]
    fn filefinder_any() {
        let finder = FileFinder {
            canonicalize: false,
            include_no_ext: false,
            extensions: ExtensionFilter::Any
        };

        match finder.execute("./", 100, 100) {
            Ok(paths) => for path in paths.into_iter() {
                eprintln!("{:?}", path);
            },
            Err(e) => panic!("{:#?}", e),
        }

        match finder.execute("./src/", 0, 4) {
            Ok(paths) => for path in paths.into_iter() {
                eprintln!("{:?}", path);
            },
            Err(e) => panic!("{:#?}", e),
        }
    }

    #[test]
    fn filefinder_extensionfilter() {
        let query = FileFinder {
            canonicalize: true,
            include_no_ext: false,
            extensions: ExtensionFilter::build(vec!["txt", "rs"], true),
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
