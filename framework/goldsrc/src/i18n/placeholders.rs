//! Placeholder formatting and variable substitution engine.

/// Replaces `{name}`, `{name='default'}`, `{0}`, and `{0='default'}` placeholders in string.
pub fn format_placeholders(
    template: &str,
    named_args: &[(&str, &str)],
    pos_args: &[&str],
) -> String {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek()
                && (next == '{' || next == '}' || next == '$' || next == '@')
            {
                result.push(next);
                chars.next();
                continue;
            }
            result.push('\\');
        } else if c == '{' {
            let mut placeholder = String::new();
            let mut closed = false;
            for p in chars.by_ref() {
                if p == '}' {
                    closed = true;
                    break;
                }
                placeholder.push(p);
            }

            if !closed {
                result.push('{');
                result.push_str(&placeholder);
                continue;
            }

            let (param_name, default_val) = match placeholder.split_once('=') {
                Some((name, def)) => {
                    let trimmed_def = def.trim().trim_matches('\'').trim_matches('"');
                    (name.trim(), Some(trimmed_def))
                }
                None => (placeholder.trim(), None),
            };

            // Check named args
            let mut replaced = false;
            for &(name, value) in named_args {
                if name == param_name {
                    result.push_str(value);
                    replaced = true;
                    break;
                }
            }

            // Check positional args if param_name is numeric
            if !replaced
                && let Ok(idx) = param_name.parse::<usize>()
                && let Some(&value) = pos_args.get(idx)
            {
                result.push_str(value);
                replaced = true;
            }

            // Apply default value if available
            if !replaced {
                if let Some(def) = default_val {
                    result.push_str(def);
                } else {
                    // Keep original placeholder if not provided
                    result.push('{');
                    result.push_str(&placeholder);
                    result.push('}');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}
