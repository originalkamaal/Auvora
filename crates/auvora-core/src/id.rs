//! External identifiers ([ADR-0018]).
//!
//! Internally every entity is keyed by a UUIDv7. Externally it is shown as
//! `<prefix>_<body>`, where `body` is the 128-bit UUID in lowercase Crockford
//! base32 (26 characters), e.g. `usr_01jbqz6v0f8m3h2k4n5p7r9s1t`.
//!
//! - The prefix list is **closed**: adding a prefix requires amending ADR-0018.
//! - Parsing is **strict**: there is exactly one canonical spelling per ID, and
//!   no case folding or Crockford aliases. This avoids aliasing bugs and oracles.
//! - The prefix is a type hint only. It is never trusted for authorization.
//! - IDs from the same generator sort lexicographically in creation order
//!   (millisecond precision).
//!
//! [ADR-0018]: https://github.com/originalkamaal/Auvora/blob/main/docs/adr/0018-identifier-format-uuidv7-prefixed.md

use std::fmt;
use std::str::FromStr;

use uuid::Uuid;

use crate::time::Clock;

macro_rules! kinds {
    ($($variant:ident => $prefix:literal, $doc:literal;)*) => {
        /// The entity type an identifier refers to (closed list, ADR-0018 §3).
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[non_exhaustive]
        pub enum Kind {
            $(#[doc = $doc] $variant,)*
        }

        impl Kind {
            /// Every registered kind, in ADR-0018 table order.
            pub const ALL: &'static [Kind] = &[$(Kind::$variant,)*];

            /// The external prefix, without the trailing underscore.
            #[must_use]
            pub const fn prefix(self) -> &'static str {
                match self {
                    $(Kind::$variant => $prefix,)*
                }
            }

            /// Looks up a kind by its exact prefix.
            #[must_use]
            pub fn from_prefix(prefix: &str) -> Option<Self> {
                match prefix {
                    $($prefix => Some(Kind::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

kinds! {
    Tenant => "ten", "Tenant (Auvora customer account).";
    Project => "prj", "Project (issuer + user pool).";
    Organization => "org", "Organization (B2B customer inside a project).";
    Membership => "mem", "Principal ↔ organization membership.";
    User => "usr", "Human user.";
    ServiceAccount => "sa", "Service account (Auvora-managed non-human principal).";
    Workload => "wl", "Externally attested workload.";
    Agent => "agt", "AI agent principal.";
    AnonymousUser => "anon", "Anonymous (guest) user.";
    Identifier => "idn", "Login identifier (email, phone, username).";
    Credential => "cred", "Credential (password verifier, passkey, TOTP, …).";
    LinkedIdentity => "lid", "Linked upstream identity.";
    Group => "grp", "Group.";
    Role => "rol", "Role.";
    Application => "app", "Application.";
    OAuthClient => "cli", "OAuth client / protocol client.";
    ResourceServer => "rs", "Resource server (API).";
    Connection => "con", "Upstream IdP or directory connection.";
    Session => "ses", "Session.";
    Device => "dev", "Device.";
    Grant => "gnt", "Consent grant.";
    Delegation => "dlg", "Delegation (agent, support or token exchange).";
    SupportSession => "sup", "Supervised support session.";
    Hook => "hook", "Hook or action.";
    Policy => "pol", "Policy.";
    Key => "key", "Cryptographic key.";
    Flow => "flw", "Authentication flow instance.";
    Event => "evt", "Domain or audit event.";
    Invitation => "inv", "Invitation.";
    Domain => "dom", "Verified domain.";
    WebhookEndpoint => "whk", "Webhook endpoint.";
    Job => "job", "Background job.";
    ApiKey => "ak", "API key (identifier part only; never the secret).";
    EnvironmentLink => "env", "Environment link between projects.";
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.prefix())
    }
}

/// Errors from generating or parsing identifiers.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdError {
    /// The string isn't `<prefix>_<26 lowercase base32 chars>`.
    #[error("malformed identifier")]
    Malformed,
    /// The prefix isn't in the ADR-0018 registry.
    #[error("unknown identifier prefix")]
    UnknownPrefix,
    /// The ID is well-formed but of a different kind than required.
    #[error("identifier kind mismatch: expected {expected}, found {found}")]
    WrongKind {
        /// Kind the caller required.
        expected: Kind,
        /// Kind encoded in the identifier.
        found: Kind,
    },
    /// The operating system CSPRNG failed.
    #[error("entropy source unavailable")]
    Entropy,
}

/// A typed external identifier: an entity [`Kind`] plus its UUIDv7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id {
    kind: Kind,
    uuid: Uuid,
}

impl Id {
    /// Generates a new UUIDv7 identifier using `clock` for the timestamp and the
    /// OS CSPRNG for the 74 random bits.
    ///
    /// # Errors
    /// [`IdError::Entropy`] if the CSPRNG is unavailable.
    pub fn generate(kind: Kind, clock: &dyn Clock) -> Result<Self, IdError> {
        let mut random = [0u8; 10];
        getrandom::fill(&mut random).map_err(|_| IdError::Entropy)?;
        let uuid =
            uuid::Builder::from_unix_timestamp_millis(clock.unix_millis(), &random).into_uuid();
        Ok(Self { kind, uuid })
    }

    /// Wraps an existing UUID (e.g. loaded from the database).
    #[must_use]
    pub const fn from_uuid(kind: Kind, uuid: Uuid) -> Self {
        Self { kind, uuid }
    }

    /// The entity kind.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// The internal UUID, as stored in PostgreSQL.
    #[must_use]
    pub const fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Parses an identifier and requires it to be of `expected` kind.
    ///
    /// API handlers use this so that an ID of the wrong type is rejected
    /// (ADR-0018 §4: the API answers 404, not 400).
    ///
    /// # Errors
    /// Any [`IdError`] from parsing, or [`IdError::WrongKind`].
    pub fn parse_as(expected: Kind, s: &str) -> Result<Self, IdError> {
        let id: Self = s.parse()?;
        if id.kind == expected {
            Ok(id)
        } else {
            Err(IdError::WrongKind {
                expected,
                found: id.kind,
            })
        }
    }
}

impl FromStr for Id {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prefix, body) = s.split_once('_').ok_or(IdError::Malformed)?;
        let kind = Kind::from_prefix(prefix).ok_or(IdError::UnknownPrefix)?;
        let value = base32::decode(body).ok_or(IdError::Malformed)?;
        Ok(Self {
            kind,
            uuid: Uuid::from_u128(value),
        })
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let body = base32::encode(self.uuid.as_u128());
        // The alphabet is ASCII, so this conversion can't fail.
        let body = std::str::from_utf8(&body).map_err(|_| fmt::Error)?;
        write!(f, "{}_{}", self.kind.prefix(), body)
    }
}

/// Lowercase Crockford base32 for exactly 128 bits (26 characters).
mod base32 {
    const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
    pub(super) const LEN: usize = 26;

    const INVALID: u8 = 0xFF;
    const DECODE: [u8; 256] = {
        let mut table = [INVALID; 256];
        let mut i = 0;
        while i < ALPHABET.len() {
            #[allow(clippy::cast_possible_truncation)] // i < 32
            {
                table[ALPHABET[i] as usize] = i as u8;
            }
            i += 1;
        }
        table
    };

    pub(super) fn encode(value: u128) -> [u8; LEN] {
        let mut out = [0u8; LEN];
        // 26 × 5 = 130 bits; the first character carries the top 3 bits.
        let mut shift: u32 = 125;
        for slot in &mut out {
            #[allow(clippy::cast_possible_truncation)] // masked to 5 bits
            let index = ((value >> shift) & 0x1F) as usize;
            *slot = ALPHABET[index];
            shift = shift.saturating_sub(5);
        }
        out
    }

    pub(super) fn decode(s: &str) -> Option<u128> {
        let bytes = s.as_bytes();
        if bytes.len() != LEN {
            return None;
        }
        let mut value: u128 = 0;
        for (position, &byte) in bytes.iter().enumerate() {
            let digit = DECODE[usize::from(byte)];
            if digit == INVALID || (position == 0 && digit > 7) {
                return None;
            }
            value = (value << 5) | u128::from(digit);
        }
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::time::FixedClock;

    fn clock(ms: u64) -> FixedClock {
        FixedClock::at_unix_millis(ms)
    }

    #[test]
    fn round_trips_through_display_and_parse() {
        for &kind in Kind::ALL {
            let id = Id::generate(kind, &clock(1_762_000_000_000)).unwrap();
            let text = id.to_string();
            assert!(text.starts_with(&format!("{}_", kind.prefix())));
            assert_eq!(text.len(), kind.prefix().len() + 1 + base32::LEN);
            assert_eq!(text.parse::<Id>().unwrap(), id);
        }
    }

    #[test]
    fn generates_uuid_v7() {
        let id = Id::generate(Kind::User, &clock(1_762_000_000_000)).unwrap();
        assert_eq!(id.uuid().get_version_num(), 7);
    }

    #[test]
    fn extreme_values_round_trip() {
        for value in [0u128, 1, u128::MAX, u128::MAX >> 1, 1 << 127] {
            let id = Id::from_uuid(Kind::Project, Uuid::from_u128(value));
            assert_eq!(id.to_string().parse::<Id>().unwrap(), id);
        }
        assert_eq!(
            Id::from_uuid(Kind::User, Uuid::from_u128(0)).to_string(),
            "usr_00000000000000000000000000"
        );
        assert_eq!(
            Id::from_uuid(Kind::User, Uuid::from_u128(u128::MAX)).to_string(),
            "usr_7zzzzzzzzzzzzzzzzzzzzzzzzz"
        );
    }

    #[test]
    fn later_ids_sort_after_earlier_ids() {
        let earlier = Id::generate(Kind::User, &clock(1_762_000_000_000))
            .unwrap()
            .to_string();
        let later = Id::generate(Kind::User, &clock(1_762_000_000_001))
            .unwrap()
            .to_string();
        assert!(earlier < later);
    }

    #[test]
    fn rejects_non_canonical_spellings() {
        let valid = "usr_01jbqz6v0f8m3h2k4n5p7r9s1t";
        assert!(valid.parse::<Id>().is_ok());
        for bad in [
            "USR_01jbqz6v0f8m3h2k4n5p7r9s1t",  // uppercase prefix
            "usr_01JBQZ6V0F8M3H2K4N5P7R9S1T",  // uppercase body
            "usr_01jbqz6v0f8m3h2k4n5p7r9s1",   // too short
            "usr_01jbqz6v0f8m3h2k4n5p7r9s1tt", // too long
            "usr_81jbqz6v0f8m3h2k4n5p7r9s1t",  // overflows 128 bits
            "usr_01jbqz6v0f8m3h2k4n5p7r9s1i",  // 'i' isn't in the alphabet
            "usr_01jbqz6v0f8m3h2k4n5p7r9s1u",  // 'u' isn't in the alphabet
            "usr-01jbqz6v0f8m3h2k4n5p7r9s1t",  // wrong separator
            "",
        ] {
            assert!(bad.parse::<Id>().is_err(), "accepted {bad:?}");
        }
        assert_eq!(
            "xyz_01jbqz6v0f8m3h2k4n5p7r9s1t".parse::<Id>(),
            Err(IdError::UnknownPrefix)
        );
    }

    #[test]
    fn parse_as_enforces_kind() {
        let id = Id::generate(Kind::Organization, &clock(1))
            .unwrap()
            .to_string();
        assert!(Id::parse_as(Kind::Organization, &id).is_ok());
        assert_eq!(
            Id::parse_as(Kind::User, &id),
            Err(IdError::WrongKind {
                expected: Kind::User,
                found: Kind::Organization
            })
        );
    }

    #[test]
    fn prefixes_are_unique_and_lowercase() {
        let mut seen = HashSet::new();
        for &kind in Kind::ALL {
            let prefix = kind.prefix();
            assert!(seen.insert(prefix), "duplicate prefix {prefix}");
            assert!(
                prefix.bytes().all(|b| b.is_ascii_lowercase()),
                "bad prefix {prefix}"
            );
            assert_eq!(Kind::from_prefix(prefix), Some(kind));
        }
    }

    #[test]
    fn generated_ids_are_unique_within_one_millisecond() {
        let clock = clock(1_762_000_000_000);
        let ids: HashSet<Id> = (0..10_000)
            .map(|_| Id::generate(Kind::Session, &clock).unwrap())
            .collect();
        assert_eq!(ids.len(), 10_000);
    }
}
