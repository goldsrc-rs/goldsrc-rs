//! Multi-plugin menu section registry.

use goldsrc::prelude::*;

/// A registered sub-menu section contributed by another plugin.
#[derive(Debug, Clone)]
pub struct RegisteredMenuSection {
    pub id: u32,
    pub title: String,
    pub command: String,
    pub capability: Option<String>,
}

/// Registry storing all contributed menu sections across the plugin suite.
#[derive(Default)]
pub struct MenuRegistry {
    sections: Vec<RegisteredMenuSection>,
}

impl MenuRegistry {
    pub fn new() -> Self {
        let mut reg = Self::default();
        // Register standard canonical sections
        reg.register(RegisteredMenuSection {
            id: 5001,
            title: "Управление сервером (Админ-меню)".to_string(),
            command: "grs_adminmenu".to_string(),
            capability: Some("admin:engine".to_string()),
        });
        reg.register(RegisteredMenuSection {
            id: 5002,
            title: "Модерация и дисциплина (Мод-меню)".to_string(),
            command: "grs_modmenu".to_string(),
            capability: Some("moderation:inspect".to_string()),
        });
        reg.register(RegisteredMenuSection {
            id: 5003,
            title: "Меню VIP привилегий".to_string(),
            command: "grs_privmenu".to_string(),
            capability: Some("vip.access".to_string()),
        });
        reg.register(RegisteredMenuSection {
            id: 5004,
            title: "Номинации и ротация карт".to_string(),
            command: "say /maps".to_string(),
            capability: None, // Public
        });
        reg
    }

    pub fn register(&mut self, section: RegisteredMenuSection) {
        self.sections.retain(|s| s.id != section.id);
        self.sections.push(section);
    }

    pub fn sections_for_player(&self, player: &Player) -> Vec<&RegisteredMenuSection> {
        self.sections
            .iter()
            .filter(|s| match &s.capability {
                Some(cap) => player.has_capability(cap),
                None => true,
            })
            .collect()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&RegisteredMenuSection> {
        self.sections.iter().find(|s| s.id == id)
    }
}
