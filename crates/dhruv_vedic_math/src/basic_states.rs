//! Basic state helpers for grahas and sensitive longitude zones.
//!
//! This module provides:
//! - marankarak-sthana classification
//! - mrityubhaga center/range helpers
//! - pushkaramsha and pushkarabhaga helpers
//!
//! The mrityubhaga and pushkara tables in this implementation follow the
//! project convention selected in user-provided requirements and are encoded
//! here as original constants for open-source use.

use crate::avastha::navamsa_number;
use crate::graha::Graha;
use crate::util::normalize_360;

const MINUTES_PER_DEGREE: f64 = 60.0;

/// Boolean basic-state bundle for a single longitude-bearing entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BasicStates {
    pub exalted: bool,
    pub debilitated: bool,
    pub combust: bool,
    pub retrograde: bool,
    pub moolatrikone: bool,
    pub marankarak_sthana: bool,
    pub mrityubhaga: bool,
    pub pushkaramsha: bool,
    pub pushkarbhaga: bool,
}

/// Minimum distances to sensitive-point definitions, in degrees.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SensitivePointDistances {
    pub mrityubhaga: f64,
    pub pushkarbhaga: f64,
}

/// Subject whose longitude is being evaluated against mrityubhaga.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MrityubhagaSubject {
    Graha(Graha),
    Point,
}

impl MrityubhagaSubject {
    fn center_degree_in_sign(self, rashi_index: u8) -> f64 {
        match self {
            Self::Graha(graha) => mrityubhaga_center_degree_for_graha(graha, rashi_index),
            Self::Point => MRITYUBHAGA_LAGNA[rashi_index as usize],
        }
    }

    fn orb_degrees(self) -> f64 {
        match self {
            Self::Graha(graha) => match graha {
                Graha::Surya => 20.0 / MINUTES_PER_DEGREE,
                Graha::Chandra => 40.0 / MINUTES_PER_DEGREE,
                Graha::Mangal => 15.0 / MINUTES_PER_DEGREE,
                Graha::Buddh => 40.0 / MINUTES_PER_DEGREE,
                Graha::Guru => 15.0 / MINUTES_PER_DEGREE,
                Graha::Shukra => 15.0 / MINUTES_PER_DEGREE,
                Graha::Shani => 15.0 / MINUTES_PER_DEGREE,
                Graha::Rahu => 15.0 / MINUTES_PER_DEGREE,
                Graha::Ketu => 15.0 / MINUTES_PER_DEGREE,
            },
            Self::Point => 40.0 / MINUTES_PER_DEGREE,
        }
    }
}

const MRITYUBHAGA_SUN: [f64; 12] = [
    20.0, 9.0, 12.0, 6.0, 8.0, 24.0, 16.0, 17.0, 22.0, 2.0, 3.0, 23.0,
];
const MRITYUBHAGA_MOON: [f64; 12] = [
    26.0, 12.0, 13.0, 25.0, 24.0, 11.0, 26.0, 14.0, 13.0, 25.0, 5.0, 12.0,
];
const MRITYUBHAGA_MARS: [f64; 12] = [
    19.0, 28.0, 25.0, 23.0, 29.0, 28.0, 14.0, 21.0, 2.0, 15.0, 11.0, 6.0,
];
const MRITYUBHAGA_MERCURY: [f64; 12] = [
    15.0, 14.0, 13.0, 12.0, 8.0, 18.0, 20.0, 10.0, 21.0, 22.0, 7.0, 5.0,
];
const MRITYUBHAGA_JUPITER: [f64; 12] = [
    19.0, 29.0, 12.0, 27.0, 6.0, 4.0, 13.0, 10.0, 17.0, 11.0, 15.0, 28.0,
];
const MRITYUBHAGA_VENUS: [f64; 12] = [
    28.0, 15.0, 11.0, 17.0, 10.0, 13.0, 4.0, 6.0, 27.0, 12.0, 29.0, 19.0,
];
const MRITYUBHAGA_SATURN: [f64; 12] = [
    10.0, 4.0, 7.0, 9.0, 12.0, 16.0, 3.0, 18.0, 28.0, 14.0, 13.0, 15.0,
];
const MRITYUBHAGA_RAHU: [f64; 12] = [
    14.0, 13.0, 12.0, 11.0, 24.0, 23.0, 22.0, 21.0, 10.0, 20.0, 18.0, 8.0,
];
const MRITYUBHAGA_KETU: [f64; 12] = [
    8.0, 18.0, 20.0, 10.0, 21.0, 22.0, 23.0, 24.0, 11.0, 12.0, 13.0, 14.0,
];
const MRITYUBHAGA_LAGNA: [f64; 12] = [
    1.0, 9.0, 22.0, 22.0, 25.0, 2.0, 4.0, 23.0, 18.0, 20.0, 24.0, 10.0,
];

const PUSHKARABHAGA_UPPER_DEGREE: [f64; 12] = [
    21.0, 14.0, 18.0, 8.0, 19.0, 9.0, 24.0, 11.0, 23.0, 14.0, 19.0, 9.0,
];

/// Exact mrityubhaga center degree within the sign for a graha.
pub fn mrityubhaga_center_degree_for_graha(graha: Graha, rashi_index: u8) -> f64 {
    let sign = rashi_index as usize;
    match graha {
        Graha::Surya => MRITYUBHAGA_SUN[sign],
        Graha::Chandra => MRITYUBHAGA_MOON[sign],
        Graha::Mangal => MRITYUBHAGA_MARS[sign],
        Graha::Buddh => MRITYUBHAGA_MERCURY[sign],
        Graha::Guru => MRITYUBHAGA_JUPITER[sign],
        Graha::Shukra => MRITYUBHAGA_VENUS[sign],
        Graha::Shani => MRITYUBHAGA_SATURN[sign],
        Graha::Rahu => MRITYUBHAGA_RAHU[sign],
        Graha::Ketu => MRITYUBHAGA_KETU[sign],
    }
}

/// Exact mrityubhaga center degree within the sign for lagna-style points.
pub fn mrityubhaga_center_degree_for_point(rashi_index: u8) -> f64 {
    MRITYUBHAGA_LAGNA[rashi_index as usize]
}

/// Distance in degrees from the sign-specific mrityubhaga center.
pub fn mrityubhaga_distance_from_center(subject: MrityubhagaSubject, sidereal_lon: f64) -> f64 {
    let lon = normalize_360(sidereal_lon);
    let rashi_index = ((lon / 30.0).floor() as u8).min(11);
    let deg_in_sign = lon % 30.0;
    let center = subject.center_degree_in_sign(rashi_index);
    (deg_in_sign - center).abs()
}

/// Whether the longitude lies within the configured mrityubhaga range.
pub fn is_in_mrityubhaga(subject: MrityubhagaSubject, sidereal_lon: f64) -> bool {
    mrityubhaga_distance_from_center(subject, sidereal_lon) <= subject.orb_degrees()
}

/// Whether a longitude falls in Pushkaramsha using the project convention.
pub fn is_pushkaramsha(sidereal_lon: f64) -> bool {
    let lon = normalize_360(sidereal_lon);
    let rashi_index = ((lon / 30.0).floor() as u8).min(11);
    let navamsa = navamsa_number(lon);
    match rashi_index {
        0 | 4 | 8 => matches!(navamsa, 7 | 9),
        1 | 5 | 9 => matches!(navamsa, 3 | 5),
        2 | 6 | 10 => matches!(navamsa, 6 | 8),
        3 | 7 | 11 => matches!(navamsa, 1 | 3),
        _ => false,
    }
}

/// Exact upper-bound degree for the sign-specific Pushkarabhaga interval.
pub fn pushkarabhaga_upper_degree(rashi_index: u8) -> f64 {
    PUSHKARABHAGA_UPPER_DEGREE[rashi_index as usize]
}

/// Distance in degrees from the sign-specific Pushkarabhaga `n` degree.
pub fn pushkarabhaga_distance_from_degree(sidereal_lon: f64) -> f64 {
    let lon = normalize_360(sidereal_lon);
    let rashi_index = ((lon / 30.0).floor() as u8).min(11);
    let deg_in_sign = lon % 30.0;
    let upper = pushkarabhaga_upper_degree(rashi_index);
    (deg_in_sign - upper).abs()
}

/// Whether the longitude lies in the Pushkarabhaga interval `(n-1, n]`.
pub fn is_pushkarabhaga(sidereal_lon: f64) -> bool {
    let lon = normalize_360(sidereal_lon);
    let rashi_index = ((lon / 30.0).floor() as u8).min(11);
    let deg_in_sign = lon % 30.0;
    let upper = pushkarabhaga_upper_degree(rashi_index);
    deg_in_sign > upper - 1.0 && deg_in_sign <= upper
}

/// Marankarak-sthana classification for classical grahas and nodes.
pub fn is_marankarak_sthana(graha: Graha, bhava_number: u8) -> bool {
    match graha {
        Graha::Surya => bhava_number == 12,
        Graha::Chandra => bhava_number == 8,
        Graha::Mangal => bhava_number == 7,
        Graha::Buddh => matches!(bhava_number, 4 | 7),
        Graha::Guru => bhava_number == 3,
        Graha::Shukra => bhava_number == 6,
        Graha::Shani => bhava_number == 1,
        Graha::Rahu => bhava_number == 9,
        Graha::Ketu => bhava_number == 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushkarabhaga_interval_is_open_closed() {
        assert!(!is_pushkarabhaga(20.0));
        assert!(is_pushkarabhaga(20.5));
        assert!(is_pushkarabhaga(21.0));
    }

    #[test]
    fn pushkarabhaga_distance_uses_upper_degree() {
        assert!((pushkarabhaga_distance_from_degree(20.5) - 0.5).abs() < 1e-10);
        assert!((pushkarabhaga_distance_from_degree(21.0)).abs() < 1e-10);
    }

    #[test]
    fn pushkaramsha_grouping_matches_rule() {
        assert!(is_pushkaramsha(20.1)); // Aries, 7th navamsa
        assert!(is_pushkaramsha(27.0)); // Aries, 9th navamsa
        assert!(!is_pushkaramsha(10.0)); // Aries, 4th navamsa
        assert!(is_pushkaramsha(36.8)); // Taurus, 3rd navamsa
    }

    #[test]
    fn mrityubhaga_distance_uses_center_degree() {
        let d = mrityubhaga_distance_from_center(MrityubhagaSubject::Graha(Graha::Surya), 20.5);
        assert!((d - 0.5).abs() < 1e-10);
    }

    #[test]
    fn mrityubhaga_range_uses_planet_orb() {
        assert!(is_in_mrityubhaga(
            MrityubhagaSubject::Graha(Graha::Surya),
            20.2,
        ));
        assert!(!is_in_mrityubhaga(
            MrityubhagaSubject::Graha(Graha::Surya),
            20.4,
        ));
        assert!(is_in_mrityubhaga(MrityubhagaSubject::Point, 1.5));
    }

    #[test]
    fn marankarak_lookup_matches_requested_table() {
        assert!(is_marankarak_sthana(Graha::Shani, 1));
        assert!(is_marankarak_sthana(Graha::Buddh, 7));
        assert!(!is_marankarak_sthana(Graha::Buddh, 5));
    }
}
