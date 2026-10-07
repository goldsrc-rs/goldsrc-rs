//! Periodic rotating announcements and banner broadcaster.

/// Rotating message broadcaster.
pub struct BroadcastRotator {
    messages: Vec<String>,
    current_index: usize,
    last_broadcast_time: f32,
}

impl Default for BroadcastRotator {
    fn default() -> Self {
        Self {
            messages: vec![
                "[Инфо] Сервер работает под управлением высокоскоростного GoldSrc.rs".to_string(),
                "[Помощь] Напишите /menu для вызова главного меню функций сервера".to_string(),
                "[Правила] Уважайте других игроков, голосовой и текстовый спам наказуем"
                    .to_string(),
                "[VIP] Напишите /vip для доступа к бонусам экипировки и арсеналу".to_string(),
            ],
            current_index: 0,
            last_broadcast_time: 0.0,
        }
    }
}

impl BroadcastRotator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates if next message is ready to be dispatched based on elapsed time.
    pub fn check_tick(&mut self, now: f32, interval_secs: f32) -> Option<String> {
        if self.messages.is_empty() {
            return None;
        }

        if now - self.last_broadcast_time >= interval_secs {
            self.last_broadcast_time = now;
            let msg = self.messages[self.current_index].clone();
            self.current_index = (self.current_index + 1) % self.messages.len();
            Some(msg)
        } else {
            None
        }
    }

    pub fn add_message(&mut self, text: String) {
        self.messages.push(text);
    }
}
