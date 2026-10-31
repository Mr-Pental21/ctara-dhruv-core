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
    /// Nakshatra (0-27, 28-scheme with Abhijit at 21) to graha_sequence index
    /// mapping, for systems that honour `DashaVariationConfig::use_abhijit`.
    /// None = the system always uses the 27-scheme map.
    pub nakshatra28_to_graha_idx: Option<[u8; 28]>,
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

    /// Get the starting graha index for a given nakshatra (27-scheme).
    pub fn starting_graha_idx(&self, nakshatra_index: u8) -> u8 {
        self.nakshatra_to_graha_idx[nakshatra_index.min(26) as usize]
    }

    /// Get the starting graha index for a given nakshatra in the 28-scheme
    /// (Abhijit at index 21). Falls back to the 27-scheme map, collapsing
    /// Abhijit onto Uttara Ashadha, for systems without a 28-scheme map.
    pub fn starting_graha_idx_28(&self, nakshatra28_index: u8) -> u8 {
        let idx = nakshatra28_index.min(27) as usize;
        match &self.nakshatra28_to_graha_idx {
            Some(map) => map[idx],
            None => {
                let idx27 = match idx {
                    0..=20 => idx,
                    _ => idx - 1,
                };
                self.nakshatra_to_graha_idx[idx27]
            }
        }
    }

    /// Whether this system honours the 28-nakshatra (Abhijit) scheme.
    pub fn supports_abhijit(&self) -> bool {
        self.nakshatra28_to_graha_idx.is_some()
    }
}

// ---------------------------------------------------------------------------
// Vimshottari Dasha (120 years, 9 grahas)
// ---------------------------------------------------------------------------

/// Vimshottari graha sequence: Ketu, Shukra, Surya, Chandra, Mangal, Rahu, Guru, Shani, Buddh.
const VIMSHOTTARI_GRAHAS: [Graha; 9] = [
    Graha::Ketu,
    Graha::Shukra,
    Graha::Surya,
    Graha::Chandra,
    Graha::Mangal,
    Graha::Rahu,
    Graha::Guru,
    Graha::Shani,
    Graha::Buddh,
];

/// Vimshottari periods in years.
const VIMSHOTTARI_YEARS: [f64; 9] = [7.0, 20.0, 6.0, 10.0, 7.0, 18.0, 16.0, 19.0, 17.0];

/// Nakshatra-to-graha mapping for Vimshottari (every 3rd nakshatra shares a graha).
const VIMSHOTTARI_NAK_MAP: [u8; 27] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, // Ashwini..Ashlesha
    0, 1, 2, 3, 4, 5, 6, 7, 8, // Magha..Jyeshtha
    0, 1, 2, 3, 4, 5, 6, 7, 8, // Mula..Revati
];

/// Create the Vimshottari dasha configuration.
pub fn vimshottari_config() -> NakshatraDashaConfig {
    let periods_days: Vec<f64> = VIMSHOTTARI_YEARS
        .iter()
        .map(|&y| y * DAYS_PER_YEAR)
        .collect();
    let total = periods_days.iter().sum();
    NakshatraDashaConfig {
        system: DashaSystem::Vimshottari,
        graha_sequence: VIMSHOTTARI_GRAHAS.to_vec(),
        periods_days,
        total_period_days: total,
        nakshatra_to_graha_idx: VIMSHOTTARI_NAK_MAP,
        cycle_count: 1,
        default_method: SubPeriodMethod::ProportionalFromParent,
        nakshatra28_to_graha_idx: None,
    }
}

// ---------------------------------------------------------------------------
// Ashtottari Dasha (108 years, 8 grahas — no Ketu)
// BPHS Ch.47. Applicable when Rahu is in Kendra/Trikona from lagna lord.
// Starting nakshatra: Ardra (index 5). Counting from Ardra, the grahas own
// groups of 4, 3, 4, 3, 4, 3, 4, 3 nakshatras in the 28-scheme (Abhijit sits
// in Shani's group between Uttara Ashadha and Shravana). Dropping Abhijit
// leaves Shani with 3 nakshatras in the 27-scheme.
// ---------------------------------------------------------------------------

const ASHTOTTARI_GRAHAS: [Graha; 8] = [
    Graha::Surya,
    Graha::Chandra,
    Graha::Mangal,
    Graha::Buddh,
    Graha::Shani,
    Graha::Guru,
    Graha::Rahu,
    Graha::Shukra,
];

const ASHTOTTARI_YEARS: [f64; 8] = [6.0, 15.0, 8.0, 17.0, 10.0, 19.0, 12.0, 21.0];

/// 27-scheme nakshatra-to-graha mapping (no Abhijit).
/// Surya: Ardra..Ashlesha (4) · Chandra: Magha..U.Phalguni (3) ·
/// Mangal: Hasta..Vishakha (4) · Buddh: Anuradha..Mula (3) ·
/// Shani: P.Ashadha..Shravana (3) · Guru: Dhanishta..P.Bhadra (3) ·
/// Rahu: U.Bhadra..Bharani (4, wraps) · Shukra: Krittika..Mrigashira (3).
const ASHTOTTARI_NAK_MAP: [u8; 27] = [
    6, 6, 7, 7, 7, // Ashwini, Bharani → Rahu; Krittika..Mrigashira → Shukra
    0, 0, 0, 0, // Ardra..Ashlesha → Surya
    1, 1, 1, // Magha..U.Phalguni → Chandra
    2, 2, 2, 2, // Hasta..Vishakha → Mangal
    3, 3, 3, // Anuradha..Mula → Buddh
    4, 4, 4, // P.Ashadha, U.Ashadha, Shravana → Shani
    5, 5, 5, // Dhanishta..P.Bhadra → Guru
    6, 6, // U.Bhadra, Revati → Rahu
];

/// 28-scheme nakshatra-to-graha mapping (Abhijit at index 21 joins Shani's
/// group, giving every graha 4/3 alternation).
const ASHTOTTARI_NAK_MAP_28: [u8; 28] = [
    6, 6, 7, 7, 7, // Ashwini, Bharani → Rahu; Krittika..Mrigashira → Shukra
    0, 0, 0, 0, // Ardra..Ashlesha → Surya
    1, 1, 1, // Magha..U.Phalguni → Chandra
    2, 2, 2, 2, // Hasta..Vishakha → Mangal
    3, 3, 3, // Anuradha..Mula → Buddh
    4, 4, 4, 4, // P.Ashadha, U.Ashadha, Abhijit, Shravana → Shani
    5, 5, 5, // Dhanishta..P.Bhadra → Guru
    6, 6, // U.Bhadra, Revati → Rahu
];

pub fn ashtottari_config() -> NakshatraDashaConfig {
    let periods_days: Vec<f64> = ASHTOTTARI_YEARS
        .iter()
        .map(|&y| y * DAYS_PER_YEAR)
        .collect();
    let total = periods_days.iter().sum();
    NakshatraDashaConfig {
        system: DashaSystem::Ashtottari,
        graha_sequence: ASHTOTTARI_GRAHAS.to_vec(),
        periods_days,
        total_period_days: total,
        nakshatra_to_graha_idx: ASHTOTTARI_NAK_MAP,
        cycle_count: 1,
        default_method: SubPeriodMethod::ProportionalFromParent,
        nakshatra28_to_graha_idx: Some(ASHTOTTARI_NAK_MAP_28),
    }
}

// ---------------------------------------------------------------------------
// Shodsottari Dasha (116 years, 8 grahas)
// BPHS Ch.48. Applicable for strong lagna lord, day birth.
// Starting nakshatra: Pushya (index 7).
// ---------------------------------------------------------------------------

const SHODSOTTARI_GRAHAS: [Graha; 8] = [
    Graha::Surya,
    Graha::Mangal,
    Graha::Guru,
    Graha::Shani,
    Graha::Ketu,
    Graha::Chandra,
    Graha::Buddh,
    Graha::Shukra,
];

const SHODSOTTARI_YEARS: [f64; 8] = [11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];

const SHODSOTTARI_NAK_MAP: [u8; 27] = [
    5, 6, 6, 6, 7, 7, 7, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5,
];

pub fn shodsottari_config() -> NakshatraDashaConfig {
    let periods_days: Vec<f64> = SHODSOTTARI_YEARS
        .iter()
        .map(|&y| y * DAYS_PER_YEAR)
        .collect();
    let total = periods_days.iter().sum();
    NakshatraDashaConfig {
        system: DashaSystem::Shodsottari,
        graha_sequence: SHODSOTTARI_GRAHAS.to_vec(),
        periods_days,
        total_period_days: total,
        nakshatra_to_graha_idx: SHODSOTTARI_NAK_MAP,
        cycle_count: 1,
        default_method: SubPeriodMethod::ProportionalFromParent,
        nakshatra28_to_graha_idx: None,
    }
}

// ---------------------------------------------------------------------------
// Dwadashottari Dasha (112 years, 8 grahas)
// BPHS Ch.49. Applicable when Venus is lagna lord.
// Starting nakshatra: Bharani (index 1).
// ---------------------------------------------------------------------------

const DWADASHOTTARI_GRAHAS: [Graha; 8] = [
    Graha::Surya,
    Graha::Guru,
    Graha::Ketu,
    Graha::Buddh,
    Graha::Rahu,
    Graha::Mangal,
    Graha::Shani,
    Graha::Chandra,
];

const DWADASHOTTARI_YEARS: [f64; 8] = [7.0, 9.0, 11.0, 13.0, 15.0, 17.0, 19.0, 21.0];

const DWADASHOTTARI_NAK_MAP: [u8; 27] = [
    7, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 7, 7,
];

pub fn dwadashottari_config() -> NakshatraDashaConfig {
    let periods_days: Vec<f64> = DWADASHOTTARI_YEARS
        .iter()
        .map(|&y| y * DAYS_PER_YEAR)
        .collect();
    let total = periods_days.iter().sum();
    NakshatraDashaConfig {
        system: DashaSystem::Dwadashottari,
        graha_sequence: DWADASHOTTARI_GRAHAS.to_vec(),
        periods_days,
        total_period_days: total,
        nakshatra_to_graha_idx: DWADASHOTTARI_NAK_MAP,
        cycle_count: 1,
        default_method: SubPeriodMethod::ProportionalFromParent,
        nakshatra28_to_graha_idx: None,
    }
}

// ---------------------------------------------------------------------------
// Panchottari Dasha (105 years, 7 grahas)
// BPHS. Applicable when lagna rises in Karka rashi and Karka dvadamshamsha.
// Starting nakshatra: Anuradha (index 16).
// ---------------------------------------------------------------------------

const PANCHOTTARI_GRAHAS: [Graha; 7] = [
    Graha::Surya,
    Graha::Buddh,
    Graha::Shani,
    Graha::Mangal,
    Graha::Shukra,
    Graha::Chandra,
    Graha::Guru,
];

const PANCHOTTARI_YEARS: [f64; 7] = [12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];

const PANCHOTTARI_NAK_MAP: [u8; 27] = [
    2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2,
];

pub fn panchottari_config() -> NakshatraDashaConfig {
    let periods_days: Vec<f64> = PANCHOTTARI_YEARS
        .iter()
        .map(|&y| y * DAYS_PER_YEAR)
        .collect();
    let total = periods_days.iter().sum();
    NakshatraDashaConfig {
        system: DashaSystem::Panchottari,
        graha_sequence: PANCHOTTARI_GRAHAS.to_vec(),
        periods_days,
        total_period_days: total,
        nakshatra_to_graha_idx: PANCHOTTARI_NAK_MAP,
        cycle_count: 1,
        default_method: SubPeriodMethod::ProportionalFromParent,
        nakshatra28_to_graha_idx: None,
    }
}

// ---------------------------------------------------------------------------
// Shatabdika Dasha (100 years, 7 grahas)
// BPHS. Applicable when lagna is vargottama.
// Starting nakshatra: Revati (index 26).
// ---------------------------------------------------------------------------

const SHATABDIKA_GRAHAS: [Graha; 7] = [
    Graha::Surya,
    Graha::Chandra,
    Graha::Shukra,
    Graha::Buddh,
    Graha::Guru,
    Graha::Mangal,
    Graha::Shani,
];

const SHATABDIKA_YEARS: [f64; 7] = [5.0, 5.0, 10.0, 10.0, 20.0, 20.0, 30.0];

const SHATABDIKA_NAK_MAP: [u8; 27] = [
    0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 0,
];

pub fn shatabdika_config() -> NakshatraDashaConfig {
    let periods_days: Vec<f64> = SHATABDIKA_YEARS
        .iter()
        .map(|&y| y * DAYS_PER_YEAR)
        .collect();
    let total = periods_days.iter().sum();
    NakshatraDashaConfig {
        system: DashaSystem::Shatabdika,
        graha_sequence: SHATABDIKA_GRAHAS.to_vec(),
        periods_days,
        total_period_days: total,
        nakshatra_to_graha_idx: SHATABDIKA_NAK_MAP,
        cycle_count: 1,
        default_method: SubPeriodMethod::ProportionalFromParent,
        nakshatra28_to_graha_idx: None,
    }
}

// ---------------------------------------------------------------------------
// Chaturashiti Sama Dasha (84 years, 7 grahas, each 12y, 2 cycles)
// BPHS. "Sama" = equal periods. Applicable when karmesha in karma bhava.
// Starting nakshatra: Swati (index 14).
// ---------------------------------------------------------------------------

const CHATURASHITI_GRAHAS: [Graha; 7] = [
    Graha::Surya,
    Graha::Chandra,
    Graha::Mangal,
    Graha::Buddh,
    Graha::Guru,
    Graha::Shukra,
    Graha::Shani,
];

const CHATURASHITI_YEARS: [f64; 7] = [12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0];

const CHATURASHITI_NAK_MAP: [u8; 27] = [
    3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3,
];

pub fn chaturashiti_config() -> NakshatraDashaConfig {
    let periods_days: Vec<f64> = CHATURASHITI_YEARS
        .iter()
        .map(|&y| y * DAYS_PER_YEAR)
        .collect();
    let total = periods_days.iter().sum();
    NakshatraDashaConfig {
        system: DashaSystem::Chaturashiti,
        graha_sequence: CHATURASHITI_GRAHAS.to_vec(),
        periods_days,
        total_period_days: total,
        nakshatra_to_graha_idx: CHATURASHITI_NAK_MAP,
        cycle_count: 2,
        default_method: SubPeriodMethod::ProportionalFromParent,
        nakshatra28_to_graha_idx: None,
    }
}

// ---------------------------------------------------------------------------
// Dwisaptati Sama Dasha (72 years, 8 grahas, each 9y, 2 cycles)
// BPHS. "Sama" = equal periods. Applicable when lagnesha in lagna or 7th.
// Starting nakshatra: Mula (index 18).
// ---------------------------------------------------------------------------

const DWISAPTATI_GRAHAS: [Graha; 8] = [
    Graha::Surya,
    Graha::Chandra,
    Graha::Mangal,
    Graha::Buddh,
    Graha::Guru,
    Graha::Shukra,
    Graha::Shani,
    Graha::Rahu,
];

const DWISAPTATI_YEARS: [f64; 8] = [9.0, 9.0, 9.0, 9.0, 9.0, 9.0, 9.0, 9.0];

const DWISAPTATI_NAK_MAP: [u8; 27] = [
    2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 7, 7, 7, 0, 0, 0, 1, 1, 1, 1, 2, 2,
];

pub fn dwisaptati_config() -> NakshatraDashaConfig {
    let periods_days: Vec<f64> = DWISAPTATI_YEARS
        .iter()
        .map(|&y| y * DAYS_PER_YEAR)
        .collect();
    let total = periods_days.iter().sum();
    NakshatraDashaConfig {
        system: DashaSystem::DwisaptatiSama,
        graha_sequence: DWISAPTATI_GRAHAS.to_vec(),
        periods_days,
        total_period_days: total,
        nakshatra_to_graha_idx: DWISAPTATI_NAK_MAP,
        cycle_count: 2,
        default_method: SubPeriodMethod::ProportionalFromParent,
        nakshatra28_to_graha_idx: None,
    }
}

// ---------------------------------------------------------------------------
// Shashtihayani Dasha (60 years, 8 grahas, 2 cycles)
// BPHS. Applicable when Sun occupies the ascendant.
// Starting nakshatra: Ashwini (index 0).
// Special: birth balance uses graha_period / nakshatra_count.
// ---------------------------------------------------------------------------

const SHASHTIHAYANI_GRAHAS: [Graha; 8] = [
    Graha::Guru,
    Graha::Surya,
    Graha::Mangal,
    Graha::Chandra,
    Graha::Buddh,
    Graha::Shukra,
    Graha::Shani,
    Graha::Rahu,
];

const SHASHTIHAYANI_YEARS: [f64; 8] = [10.0, 10.0, 10.0, 6.0, 6.0, 6.0, 6.0, 6.0];

const SHASHTIHAYANI_NAK_MAP: [u8; 27] = [
    0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 7, 7, 7,
];

pub fn shashtihayani_config() -> NakshatraDashaConfig {
    let periods_days: Vec<f64> = SHASHTIHAYANI_YEARS
        .iter()
        .map(|&y| y * DAYS_PER_YEAR)
        .collect();
    let total = periods_days.iter().sum();
    NakshatraDashaConfig {
        system: DashaSystem::Shashtihayani,
        graha_sequence: SHASHTIHAYANI_GRAHAS.to_vec(),
        periods_days,
        total_period_days: total,
        nakshatra_to_graha_idx: SHASHTIHAYANI_NAK_MAP,
        cycle_count: 2,
        default_method: SubPeriodMethod::ProportionalFromParent,
        nakshatra28_to_graha_idx: None,
    }
}

// ---------------------------------------------------------------------------
// Shat-Trimsha Sama Dasha (36 years, 8 grahas, 3 cycles)
// BPHS. "Sama" = arithmetic periods 1-8y. Day birth in Sun Hora or
// night birth in Moon Hora.
// Starting nakshatra: Shravana (index 21).
// ---------------------------------------------------------------------------

const SHAT_TRIMSHA_GRAHAS: [Graha; 8] = [
    Graha::Chandra,
    Graha::Surya,
    Graha::Guru,
    Graha::Mangal,
    Graha::Buddh,
    Graha::Shani,
    Graha::Shukra,
    Graha::Rahu,
];

const SHAT_TRIMSHA_YEARS: [f64; 8] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

const SHAT_TRIMSHA_NAK_MAP: [u8; 27] = [
    1, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 7, 7, 7, 0, 0, 0, 1, 1, 1,
];

pub fn shat_trimsha_config() -> NakshatraDashaConfig {
    let periods_days: Vec<f64> = SHAT_TRIMSHA_YEARS
        .iter()
        .map(|&y| y * DAYS_PER_YEAR)
        .collect();
    let total = periods_days.iter().sum();
    NakshatraDashaConfig {
        system: DashaSystem::ShatTrimshaSama,
        graha_sequence: SHAT_TRIMSHA_GRAHAS.to_vec(),
        periods_days,
        total_period_days: total,
        nakshatra_to_graha_idx: SHAT_TRIMSHA_NAK_MAP,
        cycle_count: 3,
        default_method: SubPeriodMethod::ProportionalFromParent,
        nakshatra28_to_graha_idx: None,
    }
}

// ---------------------------------------------------------------------------
// System lookup
// ---------------------------------------------------------------------------

/// Get the nakshatra dasha config for a given system (nakshatra-based only).
///
/// Returns None for non-nakshatra systems (Yogini, rashi-based, etc.).
pub fn nakshatra_config_for_system(system: DashaSystem) -> Option<NakshatraDashaConfig> {
    match system {
        DashaSystem::Vimshottari => Some(vimshottari_config()),
        DashaSystem::Ashtottari => Some(ashtottari_config()),
        DashaSystem::Shodsottari => Some(shodsottari_config()),
        DashaSystem::Dwadashottari => Some(dwadashottari_config()),
        DashaSystem::Panchottari => Some(panchottari_config()),
        DashaSystem::Shatabdika => Some(shatabdika_config()),
        DashaSystem::Chaturashiti => Some(chaturashiti_config()),
        DashaSystem::DwisaptatiSama => Some(dwisaptati_config()),
        DashaSystem::Shashtihayani => Some(shashtihayani_config()),
        DashaSystem::ShatTrimshaSama => Some(shat_trimsha_config()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: verify total years and nakshatra map coverage (all 27 mapped)
    fn verify_config(cfg: &NakshatraDashaConfig, expected_years: f64, expected_grahas: usize) {
        let total_years = cfg.total_period_days / DAYS_PER_YEAR;
        assert!(
            (total_years - expected_years).abs() < 1e-10,
            "{}: expected {}y total, got {}",
            cfg.system.name(),
            expected_years,
            total_years
        );
        assert_eq!(cfg.graha_sequence.len(), expected_grahas);
        assert_eq!(cfg.periods_days.len(), expected_grahas);
        // Every nakshatra should map to a valid graha index
        for (i, &gi) in cfg.nakshatra_to_graha_idx.iter().enumerate() {
            assert!(
                (gi as usize) < expected_grahas,
                "{}: nakshatra {} maps to graha index {}, but only {} grahas",
                cfg.system.name(),
                i,
                gi,
                expected_grahas
            );
        }
    }

    #[test]
    fn vimshottari_total_120_years() {
        verify_config(&vimshottari_config(), 120.0, 9);
    }

    #[test]
    fn vimshottari_ashwini_starts_ketu() {
        let cfg = vimshottari_config();
        assert_eq!(cfg.starting_graha_idx(0), 0);
        assert_eq!(cfg.graha_sequence[0], Graha::Ketu);
    }

    #[test]
    fn vimshottari_magha_starts_ketu() {
        let cfg = vimshottari_config();
        assert_eq!(cfg.starting_graha_idx(9), 0);
    }

    #[test]
    fn ashtottari_108_years_8_grahas() {
        verify_config(&ashtottari_config(), 108.0, 8);
    }

    #[test]
    fn ashtottari_ardra_starts_surya() {
        let cfg = ashtottari_config();
        assert_eq!(cfg.starting_graha_idx(5), 0); // Ardra → Surya
        assert_eq!(cfg.graha_sequence[0], Graha::Surya);
    }

    #[test]
    fn ashtottari_27_groups_follow_bphs() {
        // Counting from Ardra: 4,3,4,3,3,3,4,3 (Shani loses Abhijit).
        let cfg = ashtottari_config();
        let expect = |naks: &[u8], g: Graha| {
            for &n in naks {
                let gi = cfg.starting_graha_idx(n) as usize;
                assert_eq!(cfg.graha_sequence[gi], g, "nakshatra {n}");
            }
        };
        expect(&[5, 6, 7, 8], Graha::Surya); // Ardra..Ashlesha
        expect(&[9, 10, 11], Graha::Chandra); // Magha..U.Phalguni
        expect(&[12, 13, 14, 15], Graha::Mangal); // Hasta..Vishakha
        expect(&[16, 17, 18], Graha::Buddh); // Anuradha..Mula
        expect(&[19, 20, 21], Graha::Shani); // P.Ashadha..Shravana
        expect(&[22, 23, 24], Graha::Guru); // Dhanishta..P.Bhadra
        expect(&[25, 26, 0, 1], Graha::Rahu); // U.Bhadra..Bharani
        expect(&[2, 3, 4], Graha::Shukra); // Krittika..Mrigashira
    }

    #[test]
    fn ashtottari_28_groups_alternate_4_3() {
        let cfg = ashtottari_config();
        assert!(cfg.supports_abhijit());
        let map = cfg.nakshatra28_to_graha_idx.unwrap();
        // Abhijit (21) belongs to Shani with P.Ashadha, U.Ashadha, Shravana.
        for n in 19..=22u8 {
            assert_eq!(cfg.graha_sequence[map[n as usize] as usize], Graha::Shani);
        }
        // Every graha owns 4 or 3 nakshatras, alternating from Surya.
        let counts: Vec<usize> = (0..8u8)
            .map(|g| map.iter().filter(|&&m| m == g).count())
            .collect();
        assert_eq!(counts, vec![4, 3, 4, 3, 4, 3, 4, 3]);
        // Indices past Abhijit are the 27-scheme indices shifted by one.
        for n in 22..28u8 {
            assert_eq!(cfg.starting_graha_idx_28(n), cfg.starting_graha_idx(n - 1));
        }
    }

    #[test]
    fn systems_without_28_map_collapse_abhijit_onto_uttara_ashadha() {
        let cfg = vimshottari_config();
        assert_eq!(cfg.starting_graha_idx_28(21), cfg.starting_graha_idx(20));
        assert_eq!(cfg.starting_graha_idx_28(22), cfg.starting_graha_idx(21));
        assert_eq!(cfg.starting_graha_idx_28(27), cfg.starting_graha_idx(26));
    }

    #[test]
    fn shodsottari_116_years_8_grahas() {
        verify_config(&shodsottari_config(), 116.0, 8);
    }

    #[test]
    fn dwadashottari_112_years_8_grahas() {
        verify_config(&dwadashottari_config(), 112.0, 8);
    }

    #[test]
    fn panchottari_105_years_7_grahas() {
        verify_config(&panchottari_config(), 105.0, 7);
    }

    #[test]
    fn shatabdika_100_years_7_grahas() {
        verify_config(&shatabdika_config(), 100.0, 7);
    }

    #[test]
    fn chaturashiti_84_years_7_grahas_2_cycles() {
        let cfg = chaturashiti_config();
        verify_config(&cfg, 84.0, 7);
        assert_eq!(cfg.cycle_count, 2);
        // All periods equal (12y)
        for &p in &cfg.periods_days {
            assert!((p - 12.0 * DAYS_PER_YEAR).abs() < 1e-10);
        }
    }

    #[test]
    fn dwisaptati_72_years_8_grahas_2_cycles() {
        let cfg = dwisaptati_config();
        verify_config(&cfg, 72.0, 8);
        assert_eq!(cfg.cycle_count, 2);
        // All periods equal (9y)
        for &p in &cfg.periods_days {
            assert!((p - 9.0 * DAYS_PER_YEAR).abs() < 1e-10);
        }
    }

    #[test]
    fn shashtihayani_60_years_8_grahas_2_cycles() {
        let cfg = shashtihayani_config();
        verify_config(&cfg, 60.0, 8);
        assert_eq!(cfg.cycle_count, 2);
        assert!(!cfg.supports_abhijit());
    }

    #[test]
    fn shat_trimsha_36_years_8_grahas_3_cycles() {
        let cfg = shat_trimsha_config();
        verify_config(&cfg, 36.0, 8);
        assert_eq!(cfg.cycle_count, 3);
    }

    #[test]
    fn all_nakshatra_systems_lookup() {
        let systems = [
            DashaSystem::Vimshottari,
            DashaSystem::Ashtottari,
            DashaSystem::Shodsottari,
            DashaSystem::Dwadashottari,
            DashaSystem::Panchottari,
            DashaSystem::Shatabdika,
            DashaSystem::Chaturashiti,
            DashaSystem::DwisaptatiSama,
            DashaSystem::Shashtihayani,
            DashaSystem::ShatTrimshaSama,
        ];
        for sys in systems {
            assert!(
                nakshatra_config_for_system(sys).is_some(),
                "{:?} should have a config",
                sys
            );
        }
        // Non-nakshatra systems should return None
        assert!(nakshatra_config_for_system(DashaSystem::Yogini).is_none());
        assert!(nakshatra_config_for_system(DashaSystem::Chara).is_none());
    }
}
