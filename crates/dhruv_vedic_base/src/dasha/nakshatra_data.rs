//! Const configuration data for nakshatra-based dasha systems.
//!
//! Each system defines a graha sequence, period lengths, and a mapping
//! from the 27 nakshatras to graha sequence positions.
//!
//! Provenance: BPHS chapters on dasha systems. See docs/clean_room_dasha.md.

use crate::graha::Graha;

use super::types::{DAYS_PER_YEAR, DashaEntity, DashaSystem};
use super::variation::SubPeriodMethod;

/// Configuration for a nakshatra-based dasha system.
#[derive(Debug, Clone)]
pub struct NakshatraDashaConfig {
    /// Which system this config is for.
    pub system: DashaSystem,
    /// Graha sequence in dasha order.
    pub graha_sequence: Vec<Graha>,
    /// Full-cycle period in days for each graha in sequence.
    pub periods_days: Vec<f64>,
    /// Total period in days (sum of periods_days).
    pub total_period_days: f64,
    /// Nakshatra (0-26) to graha_sequence index mapping.
    pub nakshatra_to_graha_idx: [u8; 27],
    /// Number of cycle repetitions (1-3).
    pub cycle_count: u8,
    /// Default sub-period method.
    pub default_method: SubPeriodMethod,
}

impl NakshatraDashaConfig {
    /// Get the entity/period pairs as a flat sequence suitable for sub-period generation.
    pub fn entity_sequence(&self) -> Vec<(DashaEntity, f64)> {
        self.graha_sequence
            .iter()
            .zip(self.periods_days.iter())
            .map(|(&g, &p)| (DashaEntity::Graha(g), p))
            .collect()
    }

    /// Get the starting graha index for a given nakshatra.
    pub fn starting_graha_idx(&self, nakshatra_index: u8) -> u8 {
        self.nakshatra_to_graha_idx[nakshatra_index.min(26) as usize]
    }

    /// Get the entry period in days for the starting graha of a nakshatra.
    pub fn entry_period_days(&self, nakshatra_index: u8) -> f64 {
        let gi = self.starting_graha_idx(nakshatra_index) as usize;
        self.periods_days[gi]
    }
}

// ---------------------------------------------------------------------------
// Vimshottari Dasha (120 years, 9 grahas)
// ---------------------------------------------------------------------------

/// Vimshottari graha sequence: Ketu, Shukra, Surya, Chandra, Mangal, Rahu, Guru, Shani, Buddh.
const VIMSHOTTARI_GRAHAS: [Graha; 9] = [
    Graha::Ketu, Graha::Shukra, Graha::Surya, Graha::Chandra,
    Graha::Mangal, Graha::Rahu, Graha::Guru, Graha::Shani, Graha::Buddh,
];

/// Vimshottari periods in years.
const VIMSHOTTARI_YEARS: [f64; 9] = [7.0, 20.0, 6.0, 10.0, 7.0, 18.0, 16.0, 19.0, 17.0];

/// Nakshatra-to-graha mapping for Vimshottari (every 3rd nakshatra shares a graha).
const VIMSHOTTARI_NAK_MAP: [u8; 27] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8,  // Ashwini..Ashlesha
    0, 1, 2, 3, 4, 5, 6, 7, 8,  // Magha..Jyeshtha
    0, 1, 2, 3, 4, 5, 6, 7, 8,  // Mula..Revati
];

/// Create the Vimshottari dasha configuration.
pub fn vimshottari_config() -> NakshatraDashaConfig {
    let periods_days: Vec<f64> = VIMSHOTTARI_YEARS.iter().map(|&y| y * DAYS_PER_YEAR).collect();
    let total = periods_days.iter().sum();
    NakshatraDashaConfig {
        system: DashaSystem::Vimshottari,
        graha_sequence: VIMSHOTTARI_GRAHAS.to_vec(),
        periods_days,
        total_period_days: total,
        nakshatra_to_graha_idx: VIMSHOTTARI_NAK_MAP,
        cycle_count: 1,
        default_method: SubPeriodMethod::ProportionalFromParent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vimshottari_total_120_years() {
        let cfg = vimshottari_config();
        let total_years = cfg.total_period_days / DAYS_PER_YEAR;
        assert!((total_years - 120.0).abs() < 1e-10);
    }

    #[test]
    fn vimshottari_9_grahas() {
        let cfg = vimshottari_config();
        assert_eq!(cfg.graha_sequence.len(), 9);
        assert_eq!(cfg.periods_days.len(), 9);
    }

    #[test]
    fn vimshottari_ashwini_starts_ketu() {
        let cfg = vimshottari_config();
        assert_eq!(cfg.starting_graha_idx(0), 0); // Ashwini → Ketu (index 0)
        assert_eq!(cfg.graha_sequence[0], Graha::Ketu);
    }

    #[test]
    fn vimshottari_rohini_starts_chandra() {
        let cfg = vimshottari_config();
        assert_eq!(cfg.starting_graha_idx(3), 3); // Rohini → Chandra (index 3)
        assert_eq!(cfg.graha_sequence[3], Graha::Chandra);
    }

    #[test]
    fn vimshottari_magha_starts_ketu() {
        let cfg = vimshottari_config();
        assert_eq!(cfg.starting_graha_idx(9), 0); // Magha → Ketu (index 0)
    }

    #[test]
    fn vimshottari_ketu_7_years() {
        let cfg = vimshottari_config();
        let ketu_days = cfg.entry_period_days(0);
        let ketu_years = ketu_days / DAYS_PER_YEAR;
        assert!((ketu_years - 7.0).abs() < 1e-10);
    }
}
