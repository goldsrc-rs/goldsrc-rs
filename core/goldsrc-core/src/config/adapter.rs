//! Bidirectional configuration binder bridging disk TOML files, in-memory domain models,
//! and GoldSrc engine console variables.

use goldsrc_api::cvar::ConfigModel;
use goldsrc_spi::engine::Engine;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Reactive configuration binder for managing disk persistence and CVAR synchronizations.
pub struct ConfigBinder<T: ConfigModel> {
    path: PathBuf,
    model: T,
    engine: Option<Arc<dyn Engine>>,
}

impl<T: ConfigModel + Serialize + DeserializeOwned + Default> ConfigBinder<T> {
    /// Creates a new configuration binder for the specified path and optional engine bridge.
    pub fn new(
        path: impl Into<PathBuf>,
        default_model: T,
        engine: Option<Arc<dyn Engine>>,
    ) -> Self {
        Self {
            path: path.into(),
            model: default_model,
            engine,
        }
    }

    /// Returns a reference to the active in-memory configuration model.
    #[inline]
    pub fn model(&self) -> &T {
        &self.model
    }

    /// Returns a mutable reference to the active in-memory configuration model.
    #[inline]
    pub fn model_mut(&mut self) -> &mut T {
        &mut self.model
    }

    /// Returns the target file path.
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads the configuration from disk, or generates and saves default template if missing.
    ///
    /// Automatically registers and synchronizes engine CVARs if an engine reference is present.
    pub fn load_or_create(&mut self) -> Result<(), std::io::Error> {
        if self.path.exists() {
            let content = fs::read_to_string(&self.path)?;
            match toml::from_str::<T>(&content) {
                Ok(parsed) => {
                    self.model = parsed;
                    if let Some(engine) = &self.engine {
                        self.model.register_cvars(engine.as_ref());
                        self.model.sync_to_cvars(engine.as_ref());
                    }
                }
                Err(err) => {
                    log::warn!(
                        target: goldsrc_api::consts::log_targets::CORE,
                        "Failed to parse TOML configuration at \"{}\": {err}. Preserving active defaults.",
                        self.path.display()
                    );
                }
            }
        } else {
            if let Some(parent) = self.path.parent() {
                fs::create_dir_all(parent)?;
            }
            if let Some(engine) = &self.engine {
                self.model.register_cvars(engine.as_ref());
            }
            let toml_content = self.model.to_toml();
            fs::write(&self.path, toml_content)?;
        }

        Ok(())
    }

    /// Writes the current in-memory model to disk in documented TOML format.
    pub fn save_to_disk(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = self.model.to_toml();
        fs::write(&self.path, content)
    }

    /// Exports GoldSrc console variable `.cfg` script bindings to the specified path.
    pub fn export_cvars_cfg(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = self.model.to_cvars();
        fs::write(path, content)
    }

    /// Synchronizes the in-memory model from current engine CVAR values.
    pub fn sync_from_cvars(&mut self) {
        if let Some(engine) = &self.engine {
            self.model.sync_from_cvars(engine.as_ref());
        }
    }

    /// Writes current in-memory model values into the engine's CVARs.
    pub fn sync_to_cvars(&self) {
        if let Some(engine) = &self.engine {
            self.model.sync_to_cvars(engine.as_ref());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use goldsrc_api::cvar::CvarEngine;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    struct MockPluginConfig {
        pub tag: String,
        pub bonus_hp: i32,
        pub enabled: bool,
    }

    impl ConfigModel for MockPluginConfig {
        fn to_toml(&self) -> String {
            format!(
                "# Bonus health on spawn\nbonus_hp = {}\n# Chat tag\ntag = \"{}\"\n# Plugin switch\nenabled = {}\n",
                self.bonus_hp, self.tag, self.enabled
            )
        }

        fn to_cvars(&self) -> String {
            format!(
                "test_bonus_hp \"{}\" // Bonus HP\ntest_tag \"{}\" // Tag\ntest_enabled \"{}\" // Switch\n",
                self.bonus_hp,
                self.tag,
                if self.enabled { 1 } else { 0 }
            )
        }

        fn register_cvars(&self, engine: &dyn CvarEngine) {
            engine.cvar_register(
                "test_bonus_hp",
                &self.bonus_hp.to_string(),
                goldsrc_api::cvar::CvarFlags::ARCHIVE,
            );
            engine.cvar_register("test_tag", &self.tag, goldsrc_api::cvar::CvarFlags::ARCHIVE);
            engine.cvar_register(
                "test_enabled",
                if self.enabled { "1" } else { "0" },
                goldsrc_api::cvar::CvarFlags::ARCHIVE,
            );
        }

        fn sync_from_cvars(&mut self, engine: &dyn CvarEngine) {
            self.bonus_hp = engine.cvar_get_float("test_bonus_hp") as i32;
            if let Some(t) = engine.cvar_get_string("test_tag") {
                self.tag = t;
            }
            self.enabled = engine.cvar_get_float("test_enabled") > 0.0;
        }

        fn sync_to_cvars(&self, engine: &dyn CvarEngine) {
            engine.cvar_set_float("test_bonus_hp", self.bonus_hp as f32);
            engine.cvar_set_string("test_tag", &self.tag);
            engine.cvar_set_float("test_enabled", if self.enabled { 1.0 } else { 0.0 });
        }
    }

    #[test]
    fn test_config_binder_load_or_create_and_export() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("goldsrc_test_cfg_{unique}"));
        let cfg_path = temp_dir.join("test_plugin.toml");

        let default_cfg = MockPluginConfig {
            bonus_hp: 25,
            tag: "VIP".to_string(),
            enabled: true,
        };

        let mut binder = ConfigBinder::new(&cfg_path, default_cfg.clone(), None);
        assert!(!cfg_path.exists());

        binder.load_or_create().unwrap();
        assert!(cfg_path.exists());

        // Verify disk content has comments and keys
        let on_disk = fs::read_to_string(&cfg_path).unwrap();
        assert!(on_disk.contains("# Bonus health on spawn"));
        assert!(on_disk.contains("bonus_hp = 25"));

        // Verify cvar export
        let cvars_path = temp_dir.join("test_cvars.cfg");
        binder.export_cvars_cfg(&cvars_path).unwrap();
        let cvar_content = fs::read_to_string(&cvars_path).unwrap();
        assert!(cvar_content.contains("test_bonus_hp \"25\" // Bonus HP"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
