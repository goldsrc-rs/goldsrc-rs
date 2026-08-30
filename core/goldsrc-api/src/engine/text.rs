//! Text encoding utilities and chat color escape code converters for GoldSrc engine.

/// Converts formatting color escape sequences, scoped color macros `@{g(...)}`, and escape codes into GoldSrc `SayText` control bytes:
/// - `@{g(...)}` / `@{green(...)}` -> `\x04...\x01`
/// - `@{t(...)}` / `@{team(...)}` -> `\x03...\x01`
/// - `@{d(...)}` / `@{w(...)}` / `@{default(...)}` -> `\x01...\x01`
/// - `^1` -> `\x01` (Standard / Yellow)
/// - `^3` -> `\x03` (Team color: CT=Blue, T=Red, Spec=Grey)
/// - `^4` -> `\x04` (Green)
/// - `\t` -> `"    "` (4 spaces)
/// - `\\`, `\{`, `\}`, `\^` -> literal `\`, `{`, `}`, `^`
pub fn format_say_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 1);
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    'n' => {
                        chars.next();
                        out.push('\n');
                        continue;
                    }
                    't' => {
                        chars.next();
                        out.push_str("    ");
                        continue;
                    }
                    '\\' | '{' | '}' | '^' | '@' => {
                        chars.next();
                        out.push(next);
                        continue;
                    }
                    _ => {}
                }
            }
            out.push('\\');
        } else if c == '@' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut name = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == '(' || nc == '}' || nc.is_whitespace() {
                    break;
                }
                name.push(nc);
                chars.next();
            }

            if chars.peek() == Some(&'(') {
                chars.next(); // consume '('
                let mut paren_depth = 1;
                let mut arg = String::new();
                for ac in chars.by_ref() {
                    if ac == '(' {
                        paren_depth += 1;
                        arg.push(ac);
                    } else if ac == ')' {
                        paren_depth -= 1;
                        if paren_depth == 0 {
                            break;
                        }
                        arg.push(ac);
                    } else {
                        arg.push(ac);
                    }
                }
                // consume closing '}' if present
                if chars.peek() == Some(&'}') {
                    chars.next();
                }

                let clean_arg = arg.trim_matches(['\'', '"']);
                let formatted_inner = format_say_text(clean_arg);
                match name.to_lowercase().as_str() {
                    "g" | "green" => {
                        out.push('\x04');
                        out.push_str(&formatted_inner);
                        out.push('\x01');
                    }
                    "t" | "team" | "r" | "red" | "b" | "blue" => {
                        out.push('\x03');
                        out.push_str(&formatted_inner);
                        out.push('\x01');
                    }
                    "d" | "w" | "default" | "white" | "yellow" => {
                        out.push('\x01');
                        out.push_str(&formatted_inner);
                        out.push('\x01');
                    }
                    _ => {
                        out.push_str(&formatted_inner);
                    }
                }
                continue;
            } else {
                out.push('@');
                out.push('{');
                out.push_str(&name);
            }
        } else if c == '^'
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
            out.push(c);
        } else {
            out.push(c);
        }
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
    fn test_format_say_text_scoped_macros() {
        let input = "@{g([Admin])} Welcome, @{t(Gold-Player)}!";
        let formatted = format_say_text(input);
        assert_eq!(formatted, "\x04[Admin]\x01 Welcome, \x03Gold-Player\x01!");
    }

    #[test]
    fn test_format_say_text_escape_caret_and_brackets() {
        let input = "Escape ^^4 and \\{tag\\} and \\\\";
        let formatted = format_say_text(input);
        assert_eq!(formatted, "Escape ^4 and {tag} and \\");
    }

    #[test]
    fn test_utf8_to_cp1251_basic() {
        let input = "Привет";
        let bytes = utf8_to_cp1251(input);
        assert_eq!(bytes, vec![0xCF, 0xF0, 0xE8, 0xE2, 0xE5, 0xF2]);
    }
}
