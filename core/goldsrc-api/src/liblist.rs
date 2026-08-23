//! Parser and domain model for GoldSrc `liblist.gam` mod manifest files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Standard name of the mod manifest file.
pub const LIBLIST_FILENAME: &str = "liblist.gam";

/// Parsed representation of a GoldSrc `liblist.gam` mod descriptor file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LibList {
    /// Mod title, e.g. "Counter-Strike", "Day of Defeat".
    pub game: Option<String>,
    /// Mod version, e.g. "1.6".
    pub version: Option<String>,
    /// Web URL with mod information.
    pub url_info: Option<String>,
    /// Download URL for mod updates.
    pub url_dl: Option<String>,
    /// Target Half-Life engine version, e.g. "1111".
    pub hl_version: Option<String>,
    /// Server-side Game DLL path on Windows (e.g. "dlls/mp.dll").
    pub gamedll: Option<String>,
    /// Server-side Game DLL path on Linux (e.g. "dlls/cs.so").
    pub gamedll_linux: Option<String>,
    /// Server-side Game DLL path on macOS (e.g. "dlls/cs.dylib").
    pub gamedll_osx: Option<String>,
    /// Commented-out or original Game DLL path discovered in comments.
    pub original_gamedll: Option<String>,
    /// Primary multiplayer spawn entity name, e.g. "info_player_start".
    pub mp_entity: Option<String>,
    /// Training or tutorial map name, e.g. "tr_1".
    pub train_map: Option<String>,
    /// Fallback mod directory name, typically "valve".
    pub fallback_dir: Option<String>,
    /// Maximum configured edicts limit (e.g. 1800 or 2048, default 900 / 2048).
    pub edicts: Option<u32>,
    /// Server-only mod flag (`svonly`).
    pub svonly: Option<bool>,
    /// Client-side DLL enabled (`cldll`).
    pub cldll: Option<bool>,
    /// Secure VAC flag (`secure`).
    pub secure: Option<bool>,
    /// Mod type descriptor, e.g. "multiplayer_only" or "singleplayer_only".
    pub game_type: Option<String>,
    /// Disable custom player models (`nomodels`).
    pub no_models: Option<bool>,
    /// Disable high quality player models (`nohimodel`).
    pub no_hi_model: Option<bool>,
    /// All raw key-value pairs parsed from the manifest.
    pub raw_entries: HashMap<String, String>,
}

impl LibList {
    /// Parses a `liblist.gam` string content into a structured [`LibList`].
    pub fn parse(content: &str) -> Self {
        let mut liblist = Self::default();
        let mut commented_gamedlls = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check for commented out gamedll (e.g. `; gamedll "dlls/mp.dll"` or `// gamedll "dlls/mp.dll"`)
            if (trimmed.starts_with(';') || trimmed.starts_with("//"))
                && let Some(uncommented) = trimmed
                    .strip_prefix(';')
                    .or_else(|| trimmed.strip_prefix("//"))
            {
                let clean = uncommented.trim();
                if let Some((k, v)) = Self::parse_key_value(clean)
                    && k.starts_with("gamedll")
                    && !v.contains("goldsrc")
                {
                    commented_gamedlls.push(v);
                }
                continue;
            }

            if let Some((key, val)) = Self::parse_key_value(trimmed) {
                let key_lower = key.to_ascii_lowercase();
                match key_lower.as_str() {
                    "game" => liblist.game = Some(val.clone()),
                    "version" => liblist.version = Some(val.clone()),
                    "url_info" => liblist.url_info = Some(val.clone()),
                    "url_dl" => liblist.url_dl = Some(val.clone()),
                    "hlversion" => liblist.hl_version = Some(val.clone()),
                    "gamedll" => liblist.gamedll = Some(val.clone()),
                    "gamedll_linux" => liblist.gamedll_linux = Some(val.clone()),
                    "gamedll_osx" => liblist.gamedll_osx = Some(val.clone()),
                    "mpentity" => liblist.mp_entity = Some(val.clone()),
                    "trainmap" => liblist.train_map = Some(val.clone()),
                    "fallback_dir" => liblist.fallback_dir = Some(val.clone()),
                    "edicts" => liblist.edicts = val.parse::<u32>().ok(),
                    "svonly" => {
                        liblist.svonly = Some(val == "1" || val.eq_ignore_ascii_case("true"))
                    }
                    "cldll" => liblist.cldll = Some(val == "1" || val.eq_ignore_ascii_case("true")),
                    "secure" => {
                        liblist.secure = Some(val == "1" || val.eq_ignore_ascii_case("true"))
                    }
                    "type" => liblist.game_type = Some(val.clone()),
                    "nomodels" => {
                        liblist.no_models = Some(val == "1" || val.eq_ignore_ascii_case("true"))
                    }
                    "nohimodel" => {
                        liblist.no_hi_model = Some(val == "1" || val.eq_ignore_ascii_case("true"))
                    }
                    _ => {}
                }
                liblist.raw_entries.insert(key_lower, val);
            }
        }

        // Resolve original gamedll from commented entries if active one is our proxy
        if let Some(first_commented) = commented_gamedlls.into_iter().next() {
            liblist.original_gamedll = Some(first_commented);
        } else if let Some(ref active) = liblist.gamedll
            && !active.contains("goldsrc")
        {
            liblist.original_gamedll = Some(active.clone());
        }

        liblist
    }

    /// Loads and parses a `liblist.gam` file from the specified path.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::parse(&content))
    }

    /// Searches for `liblist.gam` in common directory candidates and returns the first found.
    pub fn find_and_load(mod_dirs: &[&str]) -> Option<(PathBuf, Self)> {
        for mod_dir in mod_dirs {
            let candidate = PathBuf::from(mod_dir).join(LIBLIST_FILENAME);
            if candidate.exists()
                && let Ok(manifest) = Self::load_from_file(&candidate)
            {
                return Some((candidate, manifest));
            }
        }
        None
    }

    /// Returns the active or original GameDLL target path for the current OS.
    pub fn target_gamedll(&self) -> Option<&str> {
        #[cfg(target_os = "windows")]
        {
            self.original_gamedll.as_deref().or(self.gamedll.as_deref())
        }
        #[cfg(target_os = "linux")]
        {
            self.gamedll_linux
                .as_deref()
                .or(self.original_gamedll.as_deref())
                .or(self.gamedll.as_deref())
        }
        #[cfg(target_os = "macos")]
        {
            self.gamedll_osx
                .as_deref()
                .or(self.original_gamedll.as_deref())
                .or(self.gamedll.as_deref())
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            self.original_gamedll.as_deref().or(self.gamedll.as_deref())
        }
    }

    /// Returns the effective max edicts limit, defaulting to `fallback` if not specified.
    pub fn max_edicts_or(&self, fallback: u32) -> u32 {
        self.edicts.unwrap_or(fallback)
    }

    /// Parses a single `key "value"` or `key value` line.
    fn parse_key_value(line: &str) -> Option<(String, String)> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        // Case 1: First token ends before first whitespace/quote
        let space_idx = line.find(|c: char| c.is_whitespace())?;
        let (k, rem) = line.split_at(space_idx);
        let (key, remainder) = (k.trim(), rem.trim());

        // Key may also be quoted
        let clean_key = key.trim_matches('"');

        // Case 2: Value may be quoted ` "val" ` or unquoted ` val `
        let clean_val = if let Some(stripped) = remainder.strip_prefix('"') {
            if let Some(end_quote) = stripped.find('"') {
                &stripped[..end_quote]
            } else {
                stripped.trim_matches('"')
            }
        } else {
            remainder.split_whitespace().next().unwrap_or(remainder)
        };

        Some((clean_key.to_string(), clean_val.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cs16_liblist() {
        let content = r#"
// Counter-Strike Game Definition File
game "Counter-Strike"
url_info "www.counter-strike.net"
url_dl ""
version "1.6"
size "184000000"
svonly "0"
cldll "1"
hlversion "1111"
nomodels "1"
nohimodel "1"
mpentity "info_player_start"
gamedll "dlls/mp.dll"
gamedll_linux "dlls/cs.so"
gamedll_osx "dlls/cs.dylib"
trainmap "tr_1"
edicts "1800"
type "multiplayer_only"
fallback_dir "valve"
secure "1"
"#;
        let manifest = LibList::parse(content);
        assert_eq!(manifest.game.as_deref(), Some("Counter-Strike"));
        assert_eq!(manifest.version.as_deref(), Some("1.6"));
        assert_eq!(manifest.gamedll.as_deref(), Some("dlls/mp.dll"));
        assert_eq!(manifest.gamedll_linux.as_deref(), Some("dlls/cs.so"));
        assert_eq!(manifest.edicts, Some(1800));
        assert_eq!(manifest.svonly, Some(false));
        assert_eq!(manifest.cldll, Some(true));
        assert_eq!(manifest.secure, Some(true));
        assert_eq!(manifest.fallback_dir.as_deref(), Some("valve"));
        assert_eq!(manifest.mp_entity.as_deref(), Some("info_player_start"));
        assert_eq!(manifest.max_edicts_or(900), 1800);
    }

    #[test]
    fn test_parse_commented_proxy_gamedll() {
        let content = r#"
game "Counter-Strike"
; gamedll "dlls/mp.dll"
gamedll "cstrike/goldsrc/bin/goldsrc_standalone.dll"
edicts "2048"
"#;
        let manifest = LibList::parse(content);
        assert_eq!(
            manifest.gamedll.as_deref(),
            Some("cstrike/goldsrc/bin/goldsrc_standalone.dll")
        );
        assert_eq!(manifest.original_gamedll.as_deref(), Some("dlls/mp.dll"));
        assert_eq!(manifest.edicts, Some(2048));
    }
}
