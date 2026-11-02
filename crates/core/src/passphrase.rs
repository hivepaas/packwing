//! A passphrase of random words for the user to take: stronger than most of what is
//! typed, and able to be written down. An archive's password can be tried offline, as
//! fast as an attacker's hardware goes, and six of these words are 77 bits.
//!
//! The words are the EFF's large list for dice (CC BY 3.0 US): see README.md.

use crate::Error;

const WORDS: &str = include_str!("passphrase_words.txt");

/// As many words as make the strength it promises.
const LEAST_WORDS: usize = 6;

/// The most a passphrase holds, whatever length is asked for.
const MOST_CHARACTERS: usize = 4096;

/// A number below `below`, each as likely as the others.
fn fair(below: usize, next: &mut impl FnMut() -> Result<u32, Error>) -> Result<usize, Error> {
    let below = below as u32;
    // The largest multiple of it: above that, some numbers would come up more.
    let fair_end = u32::MAX - (u32::MAX % below);
    loop {
        let n = next()?;
        if n < fair_end {
            return Ok((n % below) as usize);
        }
    }
}

fn by(
    least_characters: usize,
    mut next: impl FnMut() -> Result<u32, Error>,
) -> Result<String, Error> {
    let words: Vec<&str> = WORDS.lines().collect();
    let least = least_characters.min(MOST_CHARACTERS);
    let mut chosen: Vec<&str> = Vec::new();
    while chosen.len() < LEAST_WORDS || chosen.join("-").chars().count() < least {
        chosen.push(words[fair(words.len(), &mut next)?]);
    }
    Ok(chosen.join("-"))
}

/// Six random words or more, with hyphens between, at least `least_characters` long.
pub fn suggest_passphrase(least_characters: usize) -> Result<String, Error> {
    by(least_characters, || {
        let mut bytes = [0; 4];
        getrandom::fill(&mut bytes).map_err(|e| Error::Failed(e.to_string()))?;
        Ok(u32::from_le_bytes(bytes))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_is_the_one_its_strength_is_counted_for() {
        let words: std::collections::HashSet<&str> = WORDS.lines().collect();
        assert_eq!(words.len(), 7776);
        assert!(words.contains("abacus") && words.contains("zoom"));
    }

    #[test]
    fn six_words_by_default() {
        let made = suggest_passphrase(0).expect("random bytes");
        assert!(made.split('-').count() >= 6);
        assert!(made.chars().count() >= 15);
    }

    #[test]
    fn a_longer_one_takes_more_words() {
        let made = suggest_passphrase(100).expect("random bytes");
        assert!(made.chars().count() >= 100);
    }

    #[test]
    fn what_would_favour_some_words_is_thrown_away() {
        // Just past the last fair number: taken again, and the next one says word 1.
        let mut numbers = [u32::MAX, 1].into_iter().cycle();
        let made = by(0, || Ok(numbers.next().expect("it cycles"))).expect("words");
        assert_eq!(made, ["abdomen"; 6].join("-"));
    }

    #[test]
    fn two_are_not_alike() {
        let (a, b) = (suggest_passphrase(0), suggest_passphrase(0));
        assert_ne!(a.expect("random bytes"), b.expect("random bytes"));
    }
}
