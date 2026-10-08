//! Plugin crash diagnostic analysis, symbol demangling, and report formatting.

use rustc_demangle::demangle;

/// Structured representation of a plugin crash or runtime trap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginCrashReport {
    /// Canonical plugin name.
    pub plugin_name: String,
    /// Root cause of the trap/panic (e.g. `UnreachableCodeReached`, `MemoryOutOfBounds`).
    pub root_cause: String,
    /// One-line summary suitable for console alerts and log headers.
    pub one_line_summary: String,
    /// Detailed multi-line crash report with demangled frames and source locations.
    pub full_report: String,
    /// Primary source code location where the trap occurred, if resolved via DWARF.
    pub top_source_location: Option<String>,
}

/// Parses, demangles, and extracts source locations from a Wasmtime error backtrace.
pub fn format_crash_report(plugin_name: &str, err: &wasmtime::Error) -> PluginCrashReport {
    let raw_backtrace = err.to_string();
    let root_cause = format!("{:?}", err.root_cause());
    let (demangled_backtrace, top_source_location) = parse_and_demangle_backtrace(&raw_backtrace);

    let loc_suffix = match &top_source_location {
        Some(loc) => format!(" (at {loc})"),
        None => String::new(),
    };

    let one_line_summary = format!("[{plugin_name}] trapped: {root_cause}{loc_suffix}");

    let loc_str = top_source_location
        .as_deref()
        .unwrap_or("<unknown location (compiled without debuginfo / strip=debuginfo)>");

    let full_report = format!(
        "================================================================================\n\
         PLUGIN CRASH REPORT: {plugin_name}\n\
         Root Cause:       {root_cause}\n\
         Primary Location: {loc_str}\n\
         \n\
         Demangled Backtrace:\n\
         {demangled_backtrace}\n\
         \n\
         Raw Error Details:\n\
         {err:#?}\n\
         ================================================================================"
    );

    PluginCrashReport {
        plugin_name: plugin_name.to_string(),
        root_cause,
        one_line_summary,
        full_report,
        top_source_location,
    }
}

/// Demangles all Rust symbol names inside a Wasmtime backtrace string and extracts DWARF source locations.
pub fn parse_and_demangle_backtrace(raw: &str) -> (String, Option<String>) {
    let mut demangled_lines = Vec::new();
    let mut top_source_location = None;

    for line in raw.lines() {
        let trimmed = line.trim();

        // Check for source location line: "at path/file.rs:line:col"
        if trimmed.starts_with("at ") && top_source_location.is_none() {
            let loc = trimmed.trim_start_matches("at ").trim();
            if !loc.is_empty() {
                top_source_location = Some(loc.to_string());
            }
        }

        // Demangle symbol if line contains `<module>!<symbol>`
        if let Some(bang_idx) = line.find('!') {
            let prefix = &line[..=bang_idx];
            let rest = &line[bang_idx + 1..];

            // Split symbol from potential trailing whitespace/notes
            let (sym, suffix) = match rest.find(|c: char| c.is_whitespace()) {
                Some(end) => (&rest[..end], &rest[end..]),
                None => (rest, ""),
            };

            let demangled_sym = format!("{:#}", demangle(sym));
            demangled_lines.push(format!("{prefix}{demangled_sym}{suffix}"));
        } else {
            demangled_lines.push(line.to_string());
        }
    }

    (demangled_lines.join("\n"), top_source_location)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_demangle_backtrace() {
        let raw = "\
wasm backtrace:
    0:   0x1d13 - chat_director.wasm!_ZN76_$LT$chat_director..ChatDirector$u20$as$u20$goldsrc_api..bindings..Guest$GT$8on_frame17h6b7b95c4c6017c88E
       at plugins/chat_director/src/lib.rs:136:9
    1:   0x34ae - chat_director.wasm!_ZN11goldsrc_api8bindings21_export_on_frame_cabi17h83a33a259d2e146cE
    2:   0x1eb1 - chat_director.wasm!on-frame";

        let (demangled, loc) = parse_and_demangle_backtrace(raw);
        assert_eq!(
            loc.as_deref(),
            Some("plugins/chat_director/src/lib.rs:136:9")
        );
        assert!(
            demangled.contains(
                "<chat_director::ChatDirector as goldsrc_api::bindings::Guest>::on_frame"
            )
        );
        assert!(demangled.contains("goldsrc_api::bindings::_export_on_frame_cabi"));
    }

    #[test]
    fn test_format_crash_report_fallback_when_no_dwarf() {
        let raw = "\
wasm backtrace:
    0:   0x1234 - coreutils.wasm!_ZN9coreutils4init17h1234567890abcdefE";

        let (demangled, loc) = parse_and_demangle_backtrace(raw);
        assert_eq!(loc, None);
        assert!(demangled.contains("coreutils::init"));
    }
}
