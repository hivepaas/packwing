//! Which entries are taken, by their paths: patterns to include and to
//! exclude, as `.gitignore` has them.
//!
//! - `*` is any part of a name, and `?` one character of it: neither goes past `/`.
//! - `**`, a whole part of the path, is any folders, or none: `src/**/gen/`.
//! - A pattern with no `/` but at its end is a name anywhere: `*.log` is
//!   `**/*.log`, `node_modules/` is `**/node_modules/`. One with a `/` before
//!   its end starts where the paths do: `docs/drafts/`, or `/docs` with the
//!   `/` that says so.
//! - A pattern that ends in `/` names folders only: `build/` leaves a file
//!   named `build` alone.
//! - A pattern that names a folder names what is in it too: `docs` takes
//!   `docs/a.txt`, as `docs/**` does.
//! - Letters match in either case: `*.jpg` takes `IMG_001.JPG`, as cameras
//!   name their photos.
//! - On Windows, `\` is taken for `/` as its paths are typed.
//!
//! An entry is taken if it is included (everything is, if nothing is said to be)
//! and not excluded: excluding wins.

use packwing_i18n::tr;

use crate::Error;

/// The entries to take: see the module's documentation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    include: Vec<Pattern>,
    exclude: Vec<Pattern>,
}

impl Filter {
    /// What takes the entries `include` says (all, if it says none) but those
    /// `exclude` says.
    pub fn new(include: &[String], exclude: &[String]) -> Result<Filter, Error> {
        let patterns = |typed: &[String]| -> Result<Vec<Pattern>, Error> {
            typed.iter().map(|p| Pattern::new(p)).collect()
        };
        Ok(Filter {
            include: patterns(include)?,
            exclude: patterns(exclude)?,
        })
    }

    /// Why `typed` is no pattern, if it is not.
    pub fn check(typed: &str) -> Result<(), Error> {
        Pattern::new(typed).map(|_| ())
    }

    /// Whether it takes everything.
    pub fn is_everything(&self) -> bool {
        self.include.is_empty() && self.exclude.is_empty()
    }

    /// The patterns to include, as they were typed.
    pub fn included(&self) -> Vec<String> {
        self.include.iter().map(|p| p.typed.clone()).collect()
    }

    /// The patterns to exclude, as they were typed.
    pub fn excluded(&self) -> Vec<String> {
        self.exclude.iter().map(|p| p.typed.clone()).collect()
    }

    /// Whether the entry at `path`, a part at a time, is taken, as a file is
    /// (or anything whose kind is not known): a folder's own pattern takes it
    /// only as what holds it.
    pub fn takes(&self, path: &[String]) -> bool {
        self.includes(path, false) && !self.excludes(path, false)
    }

    /// Whether the folder at `path` is taken.
    pub fn takes_folder(&self, path: &[String]) -> bool {
        self.includes(path, true) && !self.excludes(path, true)
    }

    /// Whether an include pattern names `path` (a folder, if `folder`), or
    /// none is given.
    pub fn includes(&self, path: &[String], folder: bool) -> bool {
        self.include.is_empty() || self.include.iter().any(|p| p.names(path, folder))
    }

    /// Whether some include pattern is given, so that what none names is left out.
    pub fn has_includes(&self) -> bool {
        !self.include.is_empty()
    }

    /// Whether an exclude pattern names `path` (a folder, if `folder`).
    pub fn excludes(&self, path: &[String], folder: bool) -> bool {
        self.exclude.iter().any(|p| p.names(path, folder))
    }
}

/// A pattern, a part of the path at a time, in lower case.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Pattern {
    typed: String,
    parts: Vec<String>,
    /// It ends in `/`: it names folders only.
    folders: bool,
}

impl Pattern {
    fn new(typed: &str) -> Result<Pattern, Error> {
        let slashed = if cfg!(windows) {
            typed.trim().replace('\\', "/")
        } else {
            typed.trim().to_owned()
        };
        let folders = slashed.ends_with('/');
        let trimmed = slashed.trim_matches('/');
        if trimmed.is_empty() {
            return Err(Error::Failed(tr!(
                "“{pattern}” names no file or folder.",
                pattern = typed
            )));
        }
        let mut parts: Vec<String> = trimmed
            .split('/')
            .filter(|p| !p.is_empty())
            .map(str::to_lowercase)
            .collect();
        // A name alone is a name anywhere, unless a `/` before it says not.
        if parts.len() == 1 && parts[0] != "**" && !slashed.starts_with('/') {
            parts.insert(0, "**".to_owned());
        }
        Ok(Pattern {
            typed: typed.trim().to_owned(),
            parts,
            folders,
        })
    }

    /// Whether it names `path` (a folder, if `folder`), or a folder `path` is in.
    fn names(&self, path: &[String], folder: bool) -> bool {
        let lower: Vec<String> = path.iter().map(|p| p.to_lowercase()).collect();
        (1..=lower.len()).any(|n| {
            (n < lower.len() || folder || !self.folders) && whole(&self.parts, &lower[..n])
        })
    }
}

/// Whether `pattern` matches all of `path`, a part against a part, `**` against
/// any parts or none.
fn whole(pattern: &[String], path: &[String]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((first, rest)) if first == "**" => {
            (0..=path.len()).any(|skip| whole(rest, &path[skip..]))
        }
        Some((first, rest)) => path
            .split_first()
            .is_some_and(|(name, under)| part(first, name) && whole(rest, under)),
    }
}

/// Whether the part `pattern`, with `*` and `?`, matches the name `name`.
fn part(pattern: &str, name: &str) -> bool {
    let (p, n): (Vec<char>, Vec<char>) = (pattern.chars().collect(), name.chars().collect());
    // The classic way: where the last `*` was, and where in `name` it took up to.
    let (mut i, mut j) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while j < n.len() {
        match p.get(i) {
            Some('*') => {
                star = Some((i, j));
                i += 1;
            }
            Some(&c) if c == '?' || c == n[j] => {
                i += 1;
                j += 1;
            }
            _ => match star {
                Some((at, took)) => {
                    i = at + 1;
                    j = took + 1;
                    star = Some((at, took + 1));
                }
                None => return false,
            },
        }
    }
    p[i..].iter().all(|&c| c == '*')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(p: &str) -> Vec<String> {
        p.split('/').map(str::to_owned).collect()
    }

    fn filter(include: &[&str], exclude: &[&str]) -> Filter {
        let owned = |l: &[&str]| l.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        Filter::new(&owned(include), &owned(exclude)).expect("patterns")
    }

    #[test]
    fn names_match_as_gitignore_has_them() {
        assert!(part("*.txt", "a.txt") && !part("*.txt", "a.txt.bak"));
        assert!(part("a?c", "abc") && !part("a?c", "ac"));
        assert!(part("*", "") && part("a*b*c", "axxbyyc") && !part("a*b*c", "axxbyy"));
        assert!(part("Ảnh*", "Ảnh 1.jpg"));
    }

    #[test]
    fn what_is_taken() {
        let all = Filter::default();
        assert!(all.is_everything() && all.takes(&path("a/b")));

        let docs = filter(&["docs"], &[]);
        assert!(docs.takes(&path("docs")) && docs.takes(&path("docs/a/b.txt")));
        assert!(!docs.takes(&path("src/docs.txt")));
        // A name alone is anywhere, and a folder takes what is in it.
        let deps = filter(&[], &["node_modules", "*.log"]);
        assert!(!deps.takes(&path("app/node_modules/x/index.js")));
        assert!(!deps.takes(&path("logs/today.log")));
        assert!(deps.takes(&path("app/src/main.js")));

        let both = filter(&["docs/**"], &["**/drafts/**"]);
        assert!(both.takes(&path("docs/a.md")));
        assert!(!both.takes(&path("docs/x/drafts/b.md")));
        assert!(!both.takes(&path("README.md")));

        let anchored = filter(&["/src/*.rs"], &[]);
        assert!(anchored.takes(&path("src/lib.rs")) && !anchored.takes(&path("x/src/lib.rs")));
    }

    #[test]
    fn a_pattern_ending_in_a_slash_names_folders_only() {
        let build = filter(&[], &["build/"]);
        assert!(!build.takes_folder(&path("app/build")));
        assert!(!build.takes(&path("app/build/out.o")));
        // A file of that name is not a folder.
        assert!(build.takes(&path("app/build")));
        // Folders a pattern names at any depth, and anchored ones.
        let deep = filter(&[], &["src/**/gen/"]);
        assert!(!deep.takes(&path("src/gen/a.rs")) && !deep.takes(&path("src/a/b/gen/c.rs")));
        assert!(deep.takes(&path("lib/gen/a.rs")) && deep.takes(&path("src/gen.rs")));
        let sub = filter(&["dir/subdir/"], &[]);
        assert!(sub.takes(&path("dir/subdir/a.txt")) && sub.takes_folder(&path("dir/subdir")));
        assert!(!sub.takes(&path("dir/a.txt")) && !sub.takes(&path("x/dir/subdir/a.txt")));
        // A name with a `/` before it is at the top only.
        let top = filter(&[], &["/notes.txt"]);
        assert!(!top.takes(&path("notes.txt")) && top.takes(&path("docs/notes.txt")));
    }

    #[test]
    fn letters_match_in_either_case() {
        let photos = filter(&["*.jpg"], &["Thumbs/"]);
        assert!(photos.takes(&path("DCIM/IMG_001.JPG")) && photos.takes(&path("a.Jpg")));
        assert!(!photos.takes(&path("thumbs/a.jpg")));
        assert!(part("ảnh*", "ảnh 1.jpg"));
        assert!(filter(&["ẢNH*"], &[]).takes(&path("ảnh 1.jpg")));
    }

    #[test]
    fn what_was_typed_is_kept() {
        let f = filter(&[" *.PDF "], &["node_modules/"]);
        assert_eq!(f.included(), ["*.PDF"]);
        assert_eq!(f.excluded(), ["node_modules/"]);
        assert!(Filter::check("a/").is_ok() && Filter::check("//").is_err());
    }

    #[test]
    fn a_pattern_of_nothing_is_refused() {
        assert!(Filter::new(&["/".to_owned()], &[]).is_err());
    }
}
