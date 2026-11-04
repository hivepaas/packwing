//! Which entries of an archive are taken, by their paths in it: patterns to
//! include and to exclude, as `.gitignore` has them.
//!
//! - `*` is any part of a name, and `?` one character of it: neither goes past `/`.
//! - `**`, a whole part of the path, is any folders, or none.
//! - A pattern with no `/` is a name anywhere: `*.log` is `**/*.log`.
//! - A pattern that names a folder names what is in it too: `docs` takes
//!   `docs/a.txt`, as `docs/**` does.
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

    /// Whether it takes everything.
    pub fn is_everything(&self) -> bool {
        self.include.is_empty() && self.exclude.is_empty()
    }

    /// Whether the entry at `path`, a part at a time, is taken.
    pub fn takes(&self, path: &[String]) -> bool {
        let included = self.include.is_empty() || self.include.iter().any(|p| p.names(path));
        included && !self.exclude.iter().any(|p| p.names(path))
    }
}

/// A pattern, a part of the path at a time.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Pattern {
    parts: Vec<String>,
}

impl Pattern {
    fn new(typed: &str) -> Result<Pattern, Error> {
        let trimmed = typed.trim_matches('/');
        if trimmed.is_empty() {
            return Err(Error::Failed(tr!(
                "“{pattern}” names no file or folder.",
                pattern = typed
            )));
        }
        let mut parts: Vec<String> = trimmed
            .split('/')
            .filter(|p| !p.is_empty())
            .map(str::to_owned)
            .collect();
        // A name alone is a name anywhere.
        if parts.len() == 1 && parts[0] != "**" {
            parts.insert(0, "**".to_owned());
        }
        Ok(Pattern { parts })
    }

    /// Whether it names `path`, or a folder `path` is in.
    fn names(&self, path: &[String]) -> bool {
        (1..=path.len()).any(|n| whole(&self.parts, &path[..n]))
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
    fn a_pattern_of_nothing_is_refused() {
        assert!(Filter::new(&["/".to_owned()], &[]).is_err());
    }
}
