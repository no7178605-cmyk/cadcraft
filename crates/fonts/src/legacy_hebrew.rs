//! Legacy Israeli Hebrew SHX text (miryl, sivan, hebtxt, dimheb, oron …).
//!
//! Before Unicode, Israeli AutoCAD drawings stored Hebrew as the Latin keys of the Hebrew
//! keyboard layout (SI 1452): `jkui` is חלון, `ehr ,unl` is קיר תומך. The SHX fonts draw the
//! Hebrew letter at each key's code point and lay the line out right to left, so runs that read
//! left to right — digits and capital Latin letters — were typed reversed (`081` shows 180,
//! `CVP` shows PVC, `05q041` shows 140/50). Those fonts are not ours to ship, so text set in one
//! is shown by converting it to Unicode Hebrew in logical order for display; the drawing keeps
//! its stored text, so it round-trips unchanged.

/// Whether a text style font (`miryl`, `SIVAN_M.SHX`, `hebtxt` …) is one of the legacy Israeli
/// Hebrew SHX fonts whose text is keyboard-encoded (see the module docs).
pub fn is_legacy_hebrew_font(font: &str) -> bool {
    let name = font.trim().to_ascii_lowercase();
    let stem = name.strip_suffix(".shx").unwrap_or(&name);
    if stem.ends_with(".ttf") || stem.ends_with(".otf") || stem.ends_with(".ttc") {
        return false;
    }
    const FAMILIES: &[&str] = &["miryl", "sivan", "hebtxt", "dimheb", "oron", "hanit", "heb", "mavat"];
    FAMILIES.iter().any(|f| stem.starts_with(f))
}

/// The Hebrew letter on a key of the SI 1452 layout (lower case letters and `,` `;` `q` `w`),
/// else the character itself.
fn key(c: char) -> char {
    match c {
        'q' => '/',
        'w' => '\'',
        'e' => 'ק',
        'r' => 'ר',
        't' => 'א',
        'y' => 'ט',
        'u' => 'ו',
        'i' => 'ן',
        'o' => 'ם',
        'p' => 'פ',
        'a' => 'ש',
        's' => 'ד',
        'd' => 'ג',
        'f' => 'כ',
        'g' => 'ע',
        'h' => 'י',
        'j' => 'ח',
        'k' => 'ל',
        'l' => 'ך',
        ';' => 'ף',
        'z' => 'ז',
        'x' => 'ס',
        'c' => 'ב',
        'v' => 'ה',
        'b' => 'נ',
        'n' => 'מ',
        'm' => 'צ',
        ',' => 'ת',
        other => other,
    }
}

/// A character of a left-to-right run as stored: a digit or a capital Latin letter.
fn strong_ltr(c: char) -> bool {
    c.is_ascii_digit() || c.is_ascii_uppercase()
}

/// Keyboard-encoded legacy text as Unicode Hebrew in logical order: Hebrew keys become letters,
/// and each left-to-right run (digits and capitals, with `. / : - + %` between them) is put back
/// in reading order. A `q` inside a number is its `/` (`05q041` → `140/50`).
pub fn legacy_hebrew_text(stored: &str) -> String {
    let chars: Vec<char> = stored.chars().collect();
    let mut out = String::with_capacity(stored.len() * 2);
    let mut i = 0;
    while let Some(&c) = chars.get(i) {
        if !strong_ltr(c) {
            out.push(key(c));
            i += 1;
            continue;
        }
        // The run: strong characters with separators between them; it ends at its last strong one.
        let inner = |d: char| strong_ltr(d) || matches!(d, '.' | '/' | ':' | '-' | '+' | '%' | 'q');
        let mut end = i;
        let mut j = i;
        while let Some(&d) = chars.get(j) {
            if !inner(d) {
                break;
            }
            if strong_ltr(d) {
                end = j;
            }
            j += 1;
        }
        let run = chars.get(i..=end).unwrap_or_default();
        out.extend(run.iter().rev().map(|d| if *d == 'q' { '/' } else { *d }));
        i = end + 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_the_israeli_shx_families_only() {
        for f in ["miryl", "MIRYL.SHX", "sivan_m", "sivan_m.shx", "hebtxt", "HEBTXT", "DIMHEB", "ORON", " hebtxt.shx "] {
            assert!(is_legacy_hebrew_font(f), "{f}");
        }
        for f in ["txt", "romans", "arial.ttf", "romanc", "monosi", "", "hebrew.ttf"] {
            assert!(!is_legacy_hebrew_font(f), "{f}");
        }
    }

    #[test]
    fn keyboard_text_becomes_hebrew_with_numbers_in_reading_order() {
        // Strings from a real shelter floor plan (קומת מקלט).
        assert_eq!(legacy_hebrew_text("jkui rppv"), "חלון רפפה");
        assert_eq!(legacy_hebrew_text("ehr ,unl"), "קיר תומך");
        assert_eq!(legacy_hebrew_text("jsr yrpu"), "חדר טרפו");
        assert_eq!(legacy_hebrew_text("p,hjv 081"), "פתיחה 180");
        assert_eq!(legacy_hebrew_text("05q041"), "140/50");
        assert_eq!(legacy_hebrew_text("mbr, CVP"), "צנרת PVC");
        assert_eq!(legacy_hebrew_text("thyuo+bheuz"), "איטום+ניקוז");
        assert_eq!(legacy_hebrew_text("xuko gkh, fckho"), "סולם עלית כבלים");
        assert_eq!(legacy_hebrew_text(""), "");
    }
}
