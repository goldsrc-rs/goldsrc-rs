//! Text encoding utilities and chat color escape code converters for GoldSrc engine.

/// Converts formatting color escape sequences (`^1`, `^3`, `^4`) into GoldSrc `SayText` control bytes:
/// - `^1` -> `\x01` (Standard / Yellow)
/// - `^3` -> `\x03` (Team color: CT=Blue, T=Red, Spec=Grey)
/// - `^4` -> `\x04` (Green)
pub fn format_say_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 1);
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '^'
            && let Some(&next_c) = chars.peek()
        {
            match next_c {
                '1' => {
                    chars.next();
                    out.push('\x01');
                    continue;
                }
                '2' => {
                    chars.next();
                    out.push('\x02');
                    continue;
                }
                '3' => {
                    chars.next();
                    out.push('\x03');
                    continue;
                }
                '4' => {
                    chars.next();
                    out.push('\x04');
                    continue;
                }
                '^' => {
                    chars.next();
                    out.push('^');
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }
    out
}

/// Converts a UTF-8 string into Windows-1251 (CP1251) byte representation for legacy console / center print.
/// Non-mappable Unicode codepoints are replaced with ASCII fallback `?`.
pub fn utf8_to_cp1251(input: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    for c in input.chars() {
        let b = match c {
            '\0'..='~' => c as u8,
            'А'..='я' => (c as u32 - 0x0410 + 0xC0) as u8,
            'Ё' => 0xA8,
            'ё' => 0xB8,
            'І' => 0x49, // Latin I fallback
            'і' => 0x69, // Latin i fallback
            'Ї' => 0xAF,
            'ї' => 0xBF,
            'Є' => 0xAA,
            'є' => 0xBA,
            '«' => 0xAB,
            '»' => 0xBB,
            '—' => 0x97,
            '–' => 0x96,
            '№' => 0xB9,
            _ => b'?',
        };
        out.push(b);
    }
    out
}

/// Formats a center HUD message following AMX Mod X / HLSDK conventions:
/// Converts newline characters (`\n`) into carriage returns (`\r`),
/// which the GoldSrc / Counter-Strike 1.6 HUD renderer requires for multi-line center text.
pub fn format_center_text(input: &str) -> String {
    input.replace('\n', "\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_center_text_newlines() {
        let input = "Line 1\nLine 2\nLine 3";
        let formatted = format_center_text(input);
        assert_eq!(formatted, "Line 1\rLine 2\rLine 3");
    }

    #[test]
    fn test_format_say_text_colors() {
        let input = "^4[VIP]^1 You received ^3M4A1^1!";
        let formatted = format_say_text(input);
        assert_eq!(formatted, "\x04[VIP]\x01 You received \x03M4A1\x01!");
    }

    #[test]
    fn test_format_say_text_escape_caret() {
        let input = "^^4 not green";
        let formatted = format_say_text(input);
        assert_eq!(formatted, "^4 not green");
    }

    #[test]
    fn test_utf8_to_cp1251_basic() {
        let input = "Привет, Мир!";
        let bytes = utf8_to_cp1251(input);
        assert_eq!(bytes[0], 0xCF); // П
        assert_eq!(bytes[1], 0xF0); // р
        assert_eq!(bytes[2], 0xE8); // и
        assert_eq!(bytes[3], 0xE2); // в
        assert_eq!(bytes[4], 0xE5); // е
        assert_eq!(bytes[5], 0xF2); // т
    }
}
