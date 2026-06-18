/// Number of ledgers creator-associated persistent storage should be kept alive.
///
/// This is the target TTL used whenever creator storage is created or refreshed
/// after successful market activity. It is intentionally centralized so TTL
/// policy can change without touching buy/sell trade logic.
pub const CREATOR_TTL_LEDGERS: u32 = 518_400;

/// Threshold below which creator-associated storage is refreshed up to
/// [`CREATOR_TTL_LEDGERS`].
pub const CREATOR_TTL_THRESHOLD_LEDGERS: u32 = 17_280;
