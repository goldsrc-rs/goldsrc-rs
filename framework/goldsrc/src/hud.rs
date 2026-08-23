//! Network serialization and dispatching for HUD and DHUD screen messages.

use goldsrc_api::engine::Engine;
use goldsrc_api::hud::{HudEffect, HudKind, HudMessage};

/// Serializes and sends a `HudMessage` to a target player (or all players if player_idx is None).
pub fn send_hud_message(engine: &dyn Engine, player_idx: Option<i32>, msg: &HudMessage) {
    let dest = match player_idx {
        Some(_) => goldsrc_api::MessageDest::One as i32,
        None => goldsrc_api::MessageDest::All as i32,
    };

    // Calculate fixed-point coordinates (-1.0 maps to -1.0)
    let x_fixed = if msg.position.x < 0.0 {
        -1.0
    } else {
        msg.position.x
    };
    let y_fixed = if msg.position.y < 0.0 {
        -1.0
    } else {
        msg.position.y
    };

    let x_val = (x_fixed * 8192.0) as i32;
    let y_val = (y_fixed * 8192.0) as i32;

    let (effect_val, fade_in, fade_out, hold_time, fx_time) = match msg.effect {
        HudEffect::FadeInOut {
            fade_in,
            fade_out,
            hold_time,
        } => (0, fade_in, fade_out, hold_time, 0.0),
        HudEffect::Flicker { fx_time, hold_time } => (1, 0.0, 0.0, hold_time, fx_time),
        HudEffect::Typewriter {
            char_time,
            fade_out,
            hold_time,
        } => (2, char_time, fade_out, hold_time, 0.0),
    };

    let fade_in_val = (fade_in * 256.0) as i32;
    let fade_out_val = (fade_out * 256.0) as i32;
    let hold_time_val = (hold_time * 256.0) as i32;
    let fx_time_val = (fx_time * 256.0) as i32;

    match msg.kind {
        HudKind::Classic { channel } => {
            // SVC_TEMPENTITY -> TE_TEXTMESSAGE
            engine.message_begin(dest, goldsrc_api::consts::SVC_TEMPENTITY, None, player_idx);
            engine.write_byte(goldsrc_api::consts::TE_TEXTMESSAGE as i32);
            engine.write_byte(channel as i32);
            engine.write_short(x_val);
            engine.write_short(y_val);
            engine.write_byte(effect_val);
            engine.write_byte(msg.color.r as i32);
            engine.write_byte(msg.color.g as i32);
            engine.write_byte(msg.color.b as i32);
            engine.write_byte(msg.color.a as i32);
            engine.write_byte(msg.color2.r as i32);
            engine.write_byte(msg.color2.g as i32);
            engine.write_byte(msg.color2.b as i32);
            engine.write_byte(msg.color2.a as i32);
            engine.write_short(fade_in_val);
            engine.write_short(fade_out_val);
            engine.write_short(hold_time_val);
            engine.write_short(fx_time_val);
            engine.write_string(&msg.text);
            engine.message_end();
        }
        HudKind::Dhud => {
            // Director Message (SVC_DIRECTOR / DRC_CMD_MESSAGE)
            let full_text = &msg.text;
            let text_bytes = full_text.as_bytes();
            let total_len = 1 + 1 + 2 + 2 + 1 + 4 + 4 + 2 + 2 + 2 + 2 + text_bytes.len() + 1;

            if total_len <= 500 {
                engine.message_begin(dest, goldsrc_api::consts::SVC_DIRECTOR, None, player_idx);
                engine.write_byte(total_len as i32);
                engine.write_byte(goldsrc_api::consts::DRC_CMD_MESSAGE as i32);
                engine.write_byte(effect_val);
                engine.write_short(x_val);
                engine.write_short(y_val);
                engine.write_byte(msg.color.r as i32);
                engine.write_byte(msg.color.g as i32);
                engine.write_byte(msg.color.b as i32);
                engine.write_byte(msg.color.a as i32);
                engine.write_byte(msg.color2.r as i32);
                engine.write_byte(msg.color2.g as i32);
                engine.write_byte(msg.color2.b as i32);
                engine.write_byte(msg.color2.a as i32);
                engine.write_short(fade_in_val);
                engine.write_short(fade_out_val);
                engine.write_short(hold_time_val);
                engine.write_short(fx_time_val);
                engine.write_string(full_text);
                engine.message_end();
            } else {
                // Fallback to TE_TEXTMESSAGE if oversized
                engine.message_begin(dest, goldsrc_api::consts::SVC_TEMPENTITY, None, player_idx);
                engine.write_byte(goldsrc_api::consts::TE_TEXTMESSAGE as i32);
                engine.write_byte(4); // channel 4
                engine.write_short(x_val);
                engine.write_short(y_val);
                engine.write_byte(effect_val);
                engine.write_byte(msg.color.r as i32);
                engine.write_byte(msg.color.g as i32);
                engine.write_byte(msg.color.b as i32);
                engine.write_byte(msg.color.a as i32);
                engine.write_byte(msg.color2.r as i32);
                engine.write_byte(msg.color2.g as i32);
                engine.write_byte(msg.color2.b as i32);
                engine.write_byte(msg.color2.a as i32);
                engine.write_short(fade_in_val);
                engine.write_short(fade_out_val);
                engine.write_short(hold_time_val);
                engine.write_short(fx_time_val);
                engine.write_string(full_text);
                engine.message_end();
            }
        }
    }
}
