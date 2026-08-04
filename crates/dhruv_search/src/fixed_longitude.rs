//! Fixed-longitude transit search: when a moving body reaches a fixed
//! sidereal longitude, optionally offset by an angle set.
//!
//! Answers "when does `body`'s sidereal longitude next/previously equal
//! `target + angle` (mod 360)" for each angle in a set — the root-find
//! behind gochar transit-to-natal aspect timing, promoted to a public op
//! so timeline consumers do not need windowed sweeps. The longitude model
//! (frame, ayanamsha, node mode) and numerical parameters come from
//! [`SankrantiConfig`], identical to the sankranti / ingress engine.
//! Clean-room implementation from standard astronomical conventions.

use dhruv_core::{Body, Engine};
use dhruv_time::UtcTime;

use crate::conjunction_types::SearchDirection;
use crate::error::SearchError;
use crate::sankranti::{transit_sidereal_longitude, transit_tropical_longitude};
use crate::sankranti_types::SankrantiConfig;
use crate::search_util::{
    BACKWARD_EPSILON_DAYS, FORWARD_EPSILON_DAYS, find_fixed_longitude_event, normalize_to_pm180,
};
use crate::transit_body::TransitBody;

/// A specific fixed longitude can take most of a full zodiac lap to reach
/// (including retrograde loitering); scale the per-rashi ingress scan
/// ceiling accordingly. Same reasoning as the sankranti engine's
/// specific-rashi factor.
const FIXED_TARGET_SCAN_FACTOR: f64 = 13.0;

/// A moving body reaching a fixed sidereal longitude (+ angle offset).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedLongitudeEvent {
    /// UTC time of the event.
    pub utc: UtcTime,
    /// Event time as Julian Date (TDB).
    pub jd_tdb: f64,
    /// The moving body.
    pub body: TransitBody,
    /// Base target sidereal longitude, normalized to [0, 360).
    pub target_longitude_deg: f64,
    /// Angle offset matched by this event, normalized to [0, 360).
    pub angle_deg: f64,
    /// The longitude actually reached: `(target + angle) mod 360`.
    pub matched_longitude_deg: f64,
    /// Body's sidereal longitude at the event (degrees).
    pub sidereal_longitude_deg: f64,
    /// Body's ecliptic tropical longitude at the event (degrees).
    pub tropical_longitude_deg: f64,
    /// Residual |sidereal − matched| at the refined root (degrees).
    pub actual_separation_deg: f64,
}

fn validate_inputs(
    body: TransitBody,
    target_longitude_deg: f64,
    angles_deg: &[f64],
    config: &SankrantiConfig,
) -> Result<(), SearchError> {
    config.validate().map_err(SearchError::InvalidConfig)?;
    if body == TransitBody::Body(Body::Earth) {
        return Err(SearchError::InvalidConfig(
            "Earth has no geocentric longitude to search",
        ));
    }
    if !target_longitude_deg.is_finite() {
        return Err(SearchError::InvalidConfig(
            "target_longitude_deg must be finite",
        ));
    }
    if angles_deg.iter().any(|angle| !angle.is_finite()) {
        return Err(SearchError::InvalidConfig(
            "target_angles_deg must be finite",
        ));
    }
    Ok(())
}

/// Normalize the angle set to [0, 360), defaulting to a single
/// conjunction offset, and drop exact duplicates.
fn resolved_angles(angles_deg: &[f64]) -> Vec<f64> {
    let mut angles: Vec<f64> = if angles_deg.is_empty() {
        vec![0.0]
    } else {
        angles_deg
            .iter()
            .map(|angle| angle.rem_euclid(360.0))
            .collect()
    };
    angles.sort_by(f64::total_cmp);
    angles.dedup();
    angles
}

fn build_event(
    engine: &Engine,
    body: TransitBody,
    event_jd: f64,
    target_longitude_deg: f64,
    angle_deg: f64,
    matched_longitude_deg: f64,
    config: &SankrantiConfig,
) -> Result<FixedLongitudeEvent, SearchError> {
    let sidereal = transit_sidereal_longitude(engine, body, event_jd, config)?;
    let tropical = transit_tropical_longitude(engine, body, event_jd, config)?;
    Ok(FixedLongitudeEvent {
        utc: UtcTime::from_jd_tdb(event_jd, engine.lsk()),
        jd_tdb: event_jd,
        body,
        target_longitude_deg,
        angle_deg,
        matched_longitude_deg,
        sidereal_longitude_deg: sidereal,
        tropical_longitude_deg: tropical,
        actual_separation_deg: normalize_to_pm180(sidereal - matched_longitude_deg).abs(),
    })
}

fn single_scan(
    engine: &Engine,
    body: TransitBody,
    at_jd_tdb: f64,
    target_longitude_deg: f64,
    angles_deg: &[f64],
    config: &SankrantiConfig,
    direction: SearchDirection,
) -> Result<Option<FixedLongitudeEvent>, SearchError> {
    validate_inputs(body, target_longitude_deg, angles_deg, config)?;
    let target = target_longitude_deg.rem_euclid(360.0);
    let scan_days = FIXED_TARGET_SCAN_FACTOR * body.ingress_max_scan_days();
    let (start_jd, end_jd) = match direction {
        SearchDirection::Forward => (at_jd_tdb + FORWARD_EPSILON_DAYS, at_jd_tdb + scan_days),
        SearchDirection::Backward => (at_jd_tdb - BACKWARD_EPSILON_DAYS, at_jd_tdb - scan_days),
    };
    let longitude_fn = |jd: f64| transit_sidereal_longitude(engine, body, jd, config);

    let mut best: Option<FixedLongitudeEvent> = None;
    for &angle in &resolved_angles(angles_deg) {
        let matched = (target + angle).rem_euclid(360.0);
        let Some(event_jd) = find_fixed_longitude_event(
            start_jd,
            end_jd,
            matched,
            config.step_size_days,
            config.max_iterations,
            config.convergence_days,
            &longitude_fn,
            360.0,
            direction,
        )?
        else {
            continue;
        };
        let closer = match (&best, direction) {
            (None, _) => true,
            (Some(b), SearchDirection::Forward) => event_jd < b.jd_tdb,
            (Some(b), SearchDirection::Backward) => event_jd > b.jd_tdb,
        };
        if closer {
            best = Some(build_event(
                engine, body, event_jd, target, angle, matched, config,
            )?);
        }
    }
    Ok(best)
}

/// Find the next time `body` reaches `target + angle` (mod 360) for any
/// angle in `angles_deg` (empty = conjunction only), after `at_jd_tdb`.
///
/// The scan is bounded by a per-body ceiling covering a full zodiac lap
/// including retrograde loitering; hitting the ephemeris coverage edge
/// mid-scan yields `Ok(None)`.
pub fn next_fixed_longitude(
    engine: &Engine,
    body: TransitBody,
    at_jd_tdb: f64,
    target_longitude_deg: f64,
    angles_deg: &[f64],
    config: &SankrantiConfig,
) -> Result<Option<FixedLongitudeEvent>, SearchError> {
    single_scan(
        engine,
        body,
        at_jd_tdb,
        target_longitude_deg,
        angles_deg,
        config,
        SearchDirection::Forward,
    )
}

/// Find the previous time `body` reached `target + angle` (mod 360) for
/// any angle in `angles_deg`, before `at_jd_tdb`.
pub fn prev_fixed_longitude(
    engine: &Engine,
    body: TransitBody,
    at_jd_tdb: f64,
    target_longitude_deg: f64,
    angles_deg: &[f64],
    config: &SankrantiConfig,
) -> Result<Option<FixedLongitudeEvent>, SearchError> {
    single_scan(
        engine,
        body,
        at_jd_tdb,
        target_longitude_deg,
        angles_deg,
        config,
        SearchDirection::Backward,
    )
}

/// Find every time `body` reaches `target + angle` (mod 360) inside
/// `[start_jd_tdb, end_jd_tdb]`, for each angle in `angles_deg`.
///
/// Events are sorted by time, then angle. A range reaching past the
/// loaded ephemeris coverage returns the events found up to the edge
/// rather than erroring.
pub fn search_fixed_longitudes(
    engine: &Engine,
    body: TransitBody,
    start_jd_tdb: f64,
    end_jd_tdb: f64,
    target_longitude_deg: f64,
    angles_deg: &[f64],
    config: &SankrantiConfig,
) -> Result<Vec<FixedLongitudeEvent>, SearchError> {
    validate_inputs(body, target_longitude_deg, angles_deg, config)?;
    if end_jd_tdb <= start_jd_tdb {
        return Err(SearchError::InvalidConfig(
            "end_jd_tdb must be greater than start_jd_tdb",
        ));
    }
    let target = target_longitude_deg.rem_euclid(360.0);
    let longitude_fn = |jd: f64| transit_sidereal_longitude(engine, body, jd, config);

    let mut events = Vec::new();
    for &angle in &resolved_angles(angles_deg) {
        let matched = (target + angle).rem_euclid(360.0);
        let mut cursor = start_jd_tdb;
        while let Some(event_jd) = find_fixed_longitude_event(
            cursor,
            end_jd_tdb,
            matched,
            config.step_size_days,
            config.max_iterations,
            config.convergence_days,
            &longitude_fn,
            360.0,
            SearchDirection::Forward,
        )? {
            if event_jd > end_jd_tdb + FORWARD_EPSILON_DAYS {
                break;
            }
            events.push(build_event(
                engine, body, event_jd, target, angle, matched, config,
            )?);
            cursor = event_jd + FORWARD_EPSILON_DAYS;
        }
    }

    events.sort_by(|a, b| {
        a.jd_tdb
            .total_cmp(&b.jd_tdb)
            .then_with(|| a.angle_deg.total_cmp(&b.angle_deg))
    });
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles_default_to_conjunction() {
        assert_eq!(resolved_angles(&[]), vec![0.0]);
    }

    #[test]
    fn angles_normalize_and_dedup() {
        let angles = resolved_angles(&[-120.0, 240.0, 0.0, 360.0, 180.0]);
        assert_eq!(angles, vec![0.0, 180.0, 240.0]);
    }
}
