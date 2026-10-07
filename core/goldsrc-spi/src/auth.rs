//! Authentication Service Provider Interface (Auth SPI).
//!
//! Implemented by external authenticators (e.g. official Steamworks, ReUnion dual-protocol,
//! Discord OAuth, or custom token authenticators).

use crate::identity::AuthIdentity;
use std::net::IpAddr;

/// Decision returned by an [`AuthProvider`] when evaluating a client connection handshake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeDecision {
    /// Provider does not recognize or handle this client packet/ticket; pass to the next provider in chain.
    Pass,
    /// Client is accepted and authenticated with the given [`AuthIdentity`].
    Accept(AuthIdentity),
    /// Client is rejected (e.g. invalid ticket, ban, fraud, protocol error).
    Reject {
        /// Human-readable rejection reason (e.g. displayed to user in disconnect screen).
        reason: Box<str>,
    },
}

/// Network context provided to an [`AuthProvider`] during connection handshake.
#[derive(Debug, Clone)]
pub struct HandshakeContext<'a> {
    /// Slot index assigned to the connecting client (1..=32).
    pub slot: i32,
    /// Monotonic engine user ID (`pfnGetPlayerUserId`).
    pub user_id: u32,
    /// Remote client IP address, if resolved.
    pub client_ip: Option<IpAddr>,
    /// Raw authentication ticket or packet buffer sent by the client.
    pub raw_ticket: &'a [u8],
    /// Client `userinfo` string containing cvars and connection parameters.
    pub userinfo: &'a str,
}

/// Service Provider Interface for authenticating connecting clients.
pub trait AuthProvider: Send + Sync {
    /// Unique identifier of this provider (e.g. "steam", "reunion", "discord").
    fn name(&self) -> &'static str;

    /// Priority order in the handshake pipeline (lower numbers execute earlier, default: 100).
    fn priority(&self) -> u32 {
        100
    }

    /// Evaluates incoming client connection handshake credentials.
    fn handle_handshake(&self, ctx: &HandshakeContext<'_>) -> HandshakeDecision;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::SteamId;

    struct DummySteamProvider;

    impl AuthProvider for DummySteamProvider {
        fn name(&self) -> &'static str {
            "dummy_steam"
        }

        fn handle_handshake(&self, ctx: &HandshakeContext<'_>) -> HandshakeDecision {
            if ctx.raw_ticket.starts_with(b"VALID_TICKET") {
                HandshakeDecision::Accept(AuthIdentity::steam(SteamId::from_account_id(12345)))
            } else if ctx.raw_ticket.starts_with(b"BANNED") {
                HandshakeDecision::Reject {
                    reason: "Account is banned".into(),
                }
            } else {
                HandshakeDecision::Pass
            }
        }
    }

    #[test]
    fn test_auth_provider_pipeline() {
        let provider = DummySteamProvider;
        assert_eq!(provider.name(), "dummy_steam");
        assert_eq!(provider.priority(), 100);

        let ctx_accept = HandshakeContext {
            slot: 1,
            user_id: 100,
            client_ip: None,
            raw_ticket: b"VALID_TICKET_DATA",
            userinfo: "\\name\\Player\\rate\\25000",
        };
        assert!(matches!(
            provider.handle_handshake(&ctx_accept),
            HandshakeDecision::Accept(_)
        ));

        let ctx_reject = HandshakeContext {
            slot: 2,
            user_id: 101,
            client_ip: None,
            raw_ticket: b"BANNED_USER",
            userinfo: "",
        };
        assert!(matches!(
            provider.handle_handshake(&ctx_reject),
            HandshakeDecision::Reject { .. }
        ));

        let ctx_pass = HandshakeContext {
            slot: 3,
            user_id: 102,
            client_ip: None,
            raw_ticket: b"UNKNOWN",
            userinfo: "",
        };
        assert_eq!(
            provider.handle_handshake(&ctx_pass),
            HandshakeDecision::Pass
        );
    }
}
