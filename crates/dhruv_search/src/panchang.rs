//! Panchang classification: Masa, Ayana, and Varsha determination.
//!
//! Given a UTC date, these functions determine the current lunar month (Masa),
//! solstice period (Ayana), and 60-year cycle position (Varsha).
//!
//! All functions accept and return UTC times; JD TDB is internal only.
//!
//! Clean-room implementation from standard Vedic panchang conventions.

use dhruv_core::Engine;
use dhruv_time::UtcTime;
use dhruv_vedic_base::{
    Ayana, Rashi, ayana_from_sidereal_longitude, ayanamsha_deg, jd_tdb_to_centuries,
    masa_from_rashi_index, rashi_from_longitude, samvatsara_from_year,
};

use crate::conjunction::body_ecliptic_lon_lat;
use crate::error::SearchError;
use crate::lunar_phase::{next_amavasya, prev_amavasya};
use crate::panchang_types::{AyanaInfo, MasaInfo, VarshaInfo};
use crate::sankranti::{next_specific_sankranti, prev_specific_sankranti};
use crate::sankranti_types::SankrantiConfig;

/// Get Sun's sidereal rashi index at a given JD TDB.
fn sun_sidereal_rashi_index(
    engine: &Engine,
    jd_tdb: f64,
    config: &SankrantiConfig,
) -> Result<u8, SearchError> {
    let (tropical_lon, _lat) = body_ecliptic_lon_lat(engine, dhruv_core::Body::Sun, jd_tdb)?;
    let t = jd_tdb_to_centuries(jd_tdb);
    let aya = ayanamsha_deg(config.ayanamsha_system, t, config.use_nutation);
    let sid = (tropical_lon - aya).rem_euclid(360.0);
    Ok(rashi_from_longitude(sid).rashi_index)
}

/// Get Sun's sidereal longitude at a given JD TDB.
fn sun_sidereal_longitude(
    engine: &Engine,
    jd_tdb: f64,
    config: &SankrantiConfig,
) -> Result<f64, SearchError> {
    let (tropical_lon, _lat) = body_ecliptic_lon_lat(engine, dhruv_core::Body::Sun, jd_tdb)?;
    let t = jd_tdb_to_centuries(jd_tdb);
    let aya = ayanamsha_deg(config.ayanamsha_system, t, config.use_nutation);
    Ok((tropical_lon - aya).rem_euclid(360.0))
}

/// Determine the Masa (lunar month, Amanta system) for a given date.
///
/// Amanta: month runs from new moon to new moon.
/// Month is named after the rashi the Sun is in at the *next* new moon.
/// If the Sun's rashi doesn't change between prev and next new moon, it's an adhika month.
pub fn masa_for_date(
    engine: &Engine,
    utc: &UtcTime,
    config: &SankrantiConfig,
) -> Result<MasaInfo, SearchError> {
    // Find bracketing new moons
    let prev_nm = prev_amavasya(engine, utc)?
        .ok_or(SearchError::NoConvergence("could not find previous new moon"))?;
    let next_nm = next_amavasya(engine, utc)?
        .ok_or(SearchError::NoConvergence("could not find next new moon"))?;

    let prev_nm_jd = prev_nm.utc.to_jd_tdb(engine.lsk());
    let next_nm_jd = next_nm.utc.to_jd_tdb(engine.lsk());

    // Sun's sidereal rashi at each new moon
    let rashi_at_prev = sun_sidereal_rashi_index(engine, prev_nm_jd, config)?;
    let rashi_at_next = sun_sidereal_rashi_index(engine, next_nm_jd, config)?;

    let (masa, adhika) = if rashi_at_prev != rashi_at_next {
        // Normal month: named after rashi at next new moon
        (masa_from_rashi_index(rashi_at_next), false)
    } else {
        // Adhika month: Sun stayed in the same rashi
        // Named after the next rashi (unchanged_rashi + 1)
        (masa_from_rashi_index((rashi_at_prev + 1) % 12), true)
    };

    Ok(MasaInfo {
        masa,
        adhika,
        start: prev_nm.utc,
        end: next_nm.utc,
    })
}

/// Determine the Ayana (solstice period) for a given date.
///
/// Uttarayana starts at Makar Sankranti (Sun enters Makara, sidereal 270 deg).
/// Dakshinayana starts at Karka Sankranti (Sun enters Karka, sidereal 90 deg).
pub fn ayana_for_date(
    engine: &Engine,
    utc: &UtcTime,
    config: &SankrantiConfig,
) -> Result<AyanaInfo, SearchError> {
    let jd = utc.to_jd_tdb(engine.lsk());
    let sid_lon = sun_sidereal_longitude(engine, jd, config)?;
    let current_ayana = ayana_from_sidereal_longitude(sid_lon);

    // Determine start/end based on which ayana we're in
    let (start_rashi, end_rashi) = match current_ayana {
        Ayana::Uttarayana => (Rashi::Makara, Rashi::Karka),
        Ayana::Dakshinayana => (Rashi::Karka, Rashi::Makara),
    };

    // Find the start of this ayana (previous transition)
    let start_event = prev_specific_sankranti(engine, utc, start_rashi, config)?
        .ok_or(SearchError::NoConvergence(
            "could not find ayana start sankranti",
        ))?;

    // Find the end of this ayana (next transition)
    let end_event = next_specific_sankranti(engine, utc, end_rashi, config)?
        .ok_or(SearchError::NoConvergence(
            "could not find ayana end sankranti",
        ))?;

    Ok(AyanaInfo {
        ayana: current_ayana,
        start: start_event.utc,
        end: end_event.utc,
    })
}

/// Determine the Varsha (60-year samvatsara cycle position) for a given date.
///
/// The Vedic year starts at Chaitra Pratipada: the first new moon after Mesha Sankranti
/// (Sun entering sidereal 0 deg / Mesha rashi).
pub fn varsha_for_date(
    engine: &Engine,
    utc: &UtcTime,
    config: &SankrantiConfig,
) -> Result<VarshaInfo, SearchError> {
    // Strategy: find Mesha Sankranti for this year, then next new moon after it = year start.
    // If that year start is after our date, go back one year.

    let year_start = find_chaitra_pratipada_for(engine, utc, config)?;
    let year_start_jd = year_start.to_jd_tdb(engine.lsk());
    let jd = utc.to_jd_tdb(engine.lsk());

    let (actual_start, actual_end) = if year_start_jd > jd {
        // Our date is before this year's Chaitra Pratipada — go back one year
        let prev_year_utc = UtcTime::new(utc.year - 1, 1, 15, 0, 0, 0.0);
        let prev_start = find_chaitra_pratipada_for(engine, &prev_year_utc, config)?;
        (prev_start, year_start)
    } else {
        // Find next year's Chaitra Pratipada
        let next_year_utc = UtcTime::new(utc.year + 1, 1, 15, 0, 0, 0.0);
        let next_start = find_chaitra_pratipada_for(engine, &next_year_utc, config)?;
        let next_start_jd = next_start.to_jd_tdb(engine.lsk());
        if next_start_jd <= jd {
            // Edge case: we're after next year's start too
            let following_year_utc = UtcTime::new(utc.year + 2, 1, 15, 0, 0, 0.0);
            let following_start = find_chaitra_pratipada_for(engine, &following_year_utc, config)?;
            (next_start, following_start)
        } else {
            (year_start, next_start)
        }
    };

    // Use the calendar year of the start to determine the samvatsara
    let (samvatsara, order) = samvatsara_from_year(actual_start.year);

    Ok(VarshaInfo {
        samvatsara,
        order,
        start: actual_start,
        end: actual_end,
    })
}

/// Find Chaitra Pratipada (Vedic new year) near a given date.
///
/// Chaitra Pratipada = first new moon after Mesha Sankranti in the same calendar year.
fn find_chaitra_pratipada_for(
    engine: &Engine,
    utc: &UtcTime,
    config: &SankrantiConfig,
) -> Result<UtcTime, SearchError> {
    // Find Mesha Sankranti (Sun entering sidereal 0 deg) near this date
    let search_start = UtcTime::new(utc.year, 1, 15, 0, 0, 0.0);
    let mesha_sankranti = next_specific_sankranti(engine, &search_start, Rashi::Mesha, config)?
        .ok_or(SearchError::NoConvergence(
            "could not find Mesha Sankranti",
        ))?;

    // Find the next new moon after Mesha Sankranti
    let nm = next_amavasya(engine, &mesha_sankranti.utc)?
        .ok_or(SearchError::NoConvergence(
            "could not find new moon after Mesha Sankranti",
        ))?;

    Ok(nm.utc)
}
