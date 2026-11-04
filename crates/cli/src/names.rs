//! What the user types, read: an archive's format from its name or a word, a
//! level, a size; and what is written back: a date.

use packwing_core::{Kind, Level};

/// The formats `pkw create` makes, by the names it takes, the longest first
/// so that `.tar.gz` is not taken for `.gz`.
const FORMATS: [(&str, Kind); 18] = [
    ("tar.zst", Kind::TarZst),
    ("tar.gz", Kind::TarGz),
    ("tar.xz", Kind::TarXz),
    ("tar.bz2", Kind::TarBz2),
    ("tar.lz4", Kind::TarLz4),
    ("tzst", Kind::TarZst),
    ("tgz", Kind::TarGz),
    ("txz", Kind::TarXz),
    ("tbz2", Kind::TarBz2),
    ("tlz4", Kind::TarLz4),
    ("zip", Kind::Zip),
    ("7z", Kind::SevenZip),
    ("tar", Kind::Tar),
    ("zst", Kind::Zst),
    ("gz", Kind::Gz),
    ("xz", Kind::Xz),
    ("bz2", Kind::Bz2),
    ("lz4", Kind::Lz4),
];

/// The format named `word` (`zip`, `7z`, `tar.zst`…), in any case.
pub fn format_named(word: &str) -> Option<Kind> {
    let word = word.trim_start_matches('.').to_ascii_lowercase();
    FORMATS.iter().find(|(n, _)| *n == word).map(|(_, k)| *k)
}

/// The format a file named `name` is in, by its extension.
pub fn format_of(name: &str) -> Option<Kind> {
    let name = name.to_ascii_lowercase();
    FORMATS
        .iter()
        .find(|(n, _)| name.strip_suffix(n).is_some_and(|s| s.ends_with('.')))
        .map(|(_, k)| *k)
}

/// The names `pkw create --format` takes, for its help.
pub fn format_words() -> String {
    FORMATS.map(|(n, _)| n).join(", ")
}

/// A level by its name: store, lowest, low, medium, high or highest.
pub fn level_named(word: &str) -> Option<Level> {
    Some(match word.to_ascii_lowercase().as_str() {
        "store" | "0" => Level::Store,
        "lowest" | "1" => Level::Lowest,
        "low" | "2" => Level::Low,
        "medium" | "3" => Level::Medium,
        "high" | "4" => Level::High,
        "highest" | "5" => Level::Highest,
        _ => return None,
    })
}

/// A size as typed, `25MB`, `4 GB`, `700k`, `1.5G` or bytes alone, in the
/// units the app counts in: a kilobyte is 1000 bytes.
pub fn size_typed(text: &str) -> Option<u64> {
    let text = text.trim().to_ascii_lowercase();
    let split = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ','))
        .unwrap_or(text.len());
    let (number, unit) = text.split_at(split);
    let number: f64 = number.replace(',', ".").parse().ok()?;
    let unit = match unit.trim().trim_end_matches('b') {
        "" => 1.,
        "k" => 1e3,
        "m" => 1e6,
        "g" => 1e9,
        "t" => 1e12,
        _ => return None,
    };
    let bytes = number * unit;
    (bytes.is_finite() && bytes >= 1.).then(|| bytes.round() as u64)
}

/// `seconds` since 1970, in UTC, as `2026-11-04 09:30`.
pub fn date(seconds: i64) -> String {
    let (days, rest) = (seconds.div_euclid(86_400), seconds.rem_euclid(86_400));
    // Howard Hinnant's days-to-civil.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        rest / 3600,
        rest / 60 % 60
    )
}

/// A path in an archive as typed, `docs/notes.txt`, as its names.
pub fn entry_path(typed: &str) -> Vec<String> {
    typed
        .split(['/', '\\'])
        .filter(|s| !s.is_empty() && *s != ".")
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_format_is_told_by_its_name_or_a_word() {
        assert_eq!(format_of("backup.tar.zst"), Some(Kind::TarZst));
        assert_eq!(format_of("Backup.TGZ"), Some(Kind::TarGz));
        assert_eq!(format_of("notes.txt.gz"), Some(Kind::Gz));
        assert_eq!(format_of("a.7z"), Some(Kind::SevenZip));
        assert_eq!(format_of("zip"), None, "a word alone is no file name");
        assert_eq!(format_of("a.rar"), None, "packwing does not write RAR");
        assert_eq!(format_named(".tar.xz"), Some(Kind::TarXz));
        assert_eq!(format_named("ZIP"), Some(Kind::Zip));
    }

    #[test]
    fn sizes_and_levels_are_read_as_typed() {
        assert_eq!(size_typed("25MB"), Some(25_000_000));
        assert_eq!(size_typed("4 GB"), Some(4_000_000_000));
        assert_eq!(size_typed("1,5g"), Some(1_500_000_000));
        assert_eq!(size_typed("700k"), Some(700_000));
        assert_eq!(size_typed("123"), Some(123));
        assert_eq!(size_typed("lots"), None);
        assert_eq!(size_typed("0"), None);
        assert_eq!(level_named("Highest"), Some(Level::Highest));
        assert_eq!(level_named("0"), Some(Level::Store));
        assert_eq!(level_named("max"), None);
    }

    #[test]
    fn dates_and_paths_are_written_and_read() {
        assert_eq!(date(0), "1970-01-01 00:00");
        assert_eq!(date(1_793_786_400), "2026-11-04 10:00");
        assert_eq!(date(951_782_400), "2000-02-29 00:00");
        assert_eq!(entry_path("docs/./notes.txt"), ["docs", "notes.txt"]);
        assert_eq!(entry_path("/a\\b/"), ["a", "b"]);
    }
}
