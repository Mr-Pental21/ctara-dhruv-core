//! Dasha (planetary period) calculations for Vedic astrology.
//!
//! Implements 23 dasha systems from BPHS across 4 categories:
//! - Nakshatra-based (10 systems): Vimshottari, Ashtottari, etc.
//! - Yogini (1 system)
//! - Rashi-based (10 systems): Chara, Sthira, Kendradi, etc.
//! - Special (2 systems): Kala, Kaal Chakra
//!
//! Each system supports 5 hierarchical levels (Mahadasha through Pranadasha)
//! and 6 computation tiers from simple level-0 generation to efficient
//! snapshot queries.
//!
//! See `docs/clean_room_dasha.md` for algorithm provenance.

pub mod balance;
pub mod nakshatra;
pub mod nakshatra_data;
pub mod query;
pub mod rashi_util;
pub mod subperiod;
pub mod types;
pub mod variation;

pub use balance::nakshatra_birth_balance;
pub use nakshatra::{
    nakshatra_children, nakshatra_child_period, nakshatra_complete_level, nakshatra_hierarchy,
    nakshatra_level0, nakshatra_level0_entity, nakshatra_snapshot,
};
pub use nakshatra_data::{NakshatraDashaConfig, vimshottari_config};
pub use query::{find_active_period, snapshot_from_hierarchy};
pub use rashi_util::{
    SignType, count_signs_forward, count_signs_reverse, is_odd_sign, jump_rashi, next_rashi,
    sign_type,
};
pub use subperiod::{equal_children, generate_children, proportional_children, snap_last_child_end};
pub use types::{
    ALL_DASHA_SYSTEMS, DAYS_PER_YEAR, DEFAULT_DASHA_LEVEL, DashaEntity, DashaHierarchy,
    DashaLevel, DashaPeriod, DashaSnapshot, DashaSystem, MAX_DASHA_LEVEL, MAX_DASHA_SYSTEMS,
    MAX_PERIODS_PER_LEVEL,
};
pub use variation::{
    DashaVariationConfig, SubPeriodMethod, YoginiScheme,
};
