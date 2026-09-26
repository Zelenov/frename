//! Russian → QWERTY key-position transliteration for tag search.
//!
//! When a user types with the Russian keyboard layout active, each physical key
//! produces a Cyrillic character. `ru_to_qwerty` maps those characters back to
//! the Latin letters that the same keys produce on a standard QWERTY layout,
//! so a Russian-layout query can match English tag names.

/// Convert every Cyrillic character that corresponds to a QWERTY key to its
/// Latin equivalent. Non-Cyrillic characters are passed through unchanged.
pub fn ru_to_qwerty(s: &str) -> String {
    s.chars()
        .map(|c| ru_char_to_qwerty(c).unwrap_or(c))
        .collect()
}

fn ru_char_to_qwerty(c: char) -> Option<char> {
    Some(match c {
        // Row 1
        'й' => 'q',
        'ц' => 'w',
        'у' => 'e',
        'к' => 'r',
        'е' => 't',
        'н' => 'y',
        'г' => 'u',
        'ш' => 'i',
        'щ' => 'o',
        'з' => 'p',
        // Row 2
        'ф' => 'a',
        'ы' => 's',
        'в' => 'd',
        'а' => 'f',
        'п' => 'g',
        'р' => 'h',
        'о' => 'j',
        'л' => 'k',
        'д' => 'l',
        // Row 3
        'я' => 'z',
        'ч' => 'x',
        'с' => 'c',
        'м' => 'v',
        'и' => 'b',
        'т' => 'n',
        'ь' => 'm',
        // Uppercase
        'Й' => 'Q',
        'Ц' => 'W',
        'У' => 'E',
        'К' => 'R',
        'Е' => 'T',
        'Н' => 'Y',
        'Г' => 'U',
        'Ш' => 'I',
        'Щ' => 'O',
        'З' => 'P',
        'Ф' => 'A',
        'Ы' => 'S',
        'В' => 'D',
        'А' => 'F',
        'П' => 'G',
        'Р' => 'H',
        'О' => 'J',
        'Л' => 'K',
        'Д' => 'L',
        'Я' => 'Z',
        'Ч' => 'X',
        'С' => 'C',
        'М' => 'V',
        'И' => 'B',
        'Т' => 'N',
        'Ь' => 'M',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_latin() {
        assert_eq!(ru_to_qwerty("hello"), "hello");
    }

    #[test]
    fn converts_russian_to_qwerty() {
        // "привет" on Russian layout = "ghbdtn" keys on QWERTY
        assert_eq!(ru_to_qwerty("привет"), "ghbdtn");
    }

    #[test]
    fn mixed_keeps_non_cyrillic() {
        assert_eq!(ru_to_qwerty("test123"), "test123");
    }
}
