//! Sankranti / rashi-ingress search engine.
//!
//! Finds when a body's sidereal longitude crosses a rashi boundary
//! (multiples of 30 deg). Works for any [`TransitBody`]: the Sun (classical
//! sankranti), Moon through Pluto, and Rahu/Ketu via the lunar-node model.
//!
//! Algorithm: coarse scan of the sidereal rashi index; when the index changes
//! between consecutive samples the crossed cusp is refined by bisection on
//! f(t) = normalize(sid(t) - boundary). Detecting index *changes* (rather
//! than tracking a single target boundary) makes retrograde re-ingresses
//! first-class events. Clean-room implementation from standard astronomical
//! conventions.

use dhruv_core::{Body, Engine};
use dhruv_frames::DEFAULT_PRECESSION_MODEL;
use dhruv_time::UtcTime;
use dhruv_vedic_base::{
    ALL_RASHIS, Rashi, jd_tdb_to_centuries, lunar_node_deg_for_epoch_on_plane,
    lunar_node_deg_for_epoch_with_model,
};

use crate::conjunction::{body_ecliptic_lon_lat, body_lon_lat_on_plane};
use crate::error::SearchError;
use crate::sankranti_types::{SankrantiConfig, SankrantiEvent};
use crate::search_util::{is_coverage_edge, normalize_to_pm180};
use crate::transit_body::TransitBody;

/// Cursor advance past a found crossing when scanning for a specific rashi
/// (~8.6 s; safely past the cusp at any supported body speed).
const SPECIFIC_RESUME_EPSILON_DAYS: f64 = 1e-4;

/// Specific-rashi searches may need most of a full zodiac lap; scale the
/// per-rashi scan ceiling accordingly. The Sun keeps its legacy 400-day
/// window (which already covers a full lap).
const SPECIFIC_TARGET_SCAN_FACTOR: f64 = 13.0;

/// Get a body's sidereal longitude at a given JD TDB.
///
/// Uses the reference plane configured in `config` for both the body
/// longitude and the ayanamsha, ensuring frame consistency. Rahu/Ketu use the
/// lunar-node model selected by `config.node_mode`.
pub(crate) fn transit_sidereal_longitude(
    engine: &Engine,
    body: TransitBody,
    jd_tdb: f64,
    config: &SankrantiConfig,
) -> Result<f64, SearchError> {
    let lon = match body {
        TransitBody::Body(b) => {
            let (lon, _lat) = body_lon_lat_on_plane(
                engine,
                b,
                jd_tdb,
                config.precession_model,
                config.reference_plane,
            )?;
            lon
        }
        TransitBody::Rahu | TransitBody::Ketu => {
            let node = body.lunar_node().expect("node variants carry a node");
            lunar_node_deg_for_epoch_on_plane(
                engine,
                node,
                jd_tdb,
                config.node_mode,
                config.precession_model,
                config.reference_plane,
            )?
        }
    };
    let t = jd_tdb_to_centuries(jd_tdb);
    let aya = config.ayanamsha_deg_at_centuries(t);
    Ok((lon - aya).rem_euclid(360.0))
}

/// Body's ecliptic tropical longitude at the event (existing sankranti
/// semantics: always ecliptic-of-date with the default precession model).
pub(crate) fn transit_tropical_longitude(
    engine: &Engine,
    body: TransitBody,
    jd_tdb: f64,
    config: &SankrantiConfig,
) -> Result<f64, SearchError> {
    match body {
        TransitBody::Body(b) => Ok(body_ecliptic_lon_lat(engine, b, jd_tdb)?.0),
        TransitBody::Rahu | TransitBody::Ketu => {
            let node = body.lunar_node().expect("node variants carry a node");
            Ok(lunar_node_deg_for_epoch_with_model(
                engine,
                node,
                jd_tdb,
                config.node_mode,
                DEFAULT_PRECESSION_MODEL,
            )?)
        }
    }
}

fn validate_ingress_body(body: TransitBody) -> Result<(), SearchError> {
    if body == TransitBody::Body(Body::Earth) {
        return Err(SearchError::InvalidConfig(
            "Earth has no geocentric longitude to search",
        ));
    }
    Ok(())
}

/// 0-based rashi index of a longitude in [0, 360).
fn rashi_index_of(sidereal_lon: f64) -> u8 {
    ((sidereal_lon.rem_euclid(360.0)) / 30.0) as u8 % 12
}

/// A refined rashi-boundary crossing.
#[derive(Debug, Clone, Copy)]
struct IngressCrossing {
    jd: f64,
    entered_index: u8,
    is_retrograde: bool,
}

/// Collect every cusp crossing inside the time-ordered interval
/// `(t_a, idx_a) -> (t_b, idx_b)`, in time order.
///
/// Intervals wider than the body's safe span (motion could exceed one rashi,
/// or wrap) are subdivided at their midpoint until each piece is narrow
/// enough that a rashi-index delta of +1/-1 is unambiguous. This keeps
/// over-large caller step sizes correct: every crossing is found and none is
/// fabricated from aliased motion.
#[allow(clippy::too_many_arguments)]
fn collect_crossings(
    engine: &Engine,
    body: TransitBody,
    config: &SankrantiConfig,
    t_a: f64,
    idx_a: u8,
    t_b: f64,
    idx_b: u8,
    out: &mut Vec<IngressCrossing>,
) -> Result<(), SearchError> {
    if idx_a == idx_b {
        // No net index change; an even number of crossings inside is below
        // this scan's resolution (same limitation as any coarse scan).
        return Ok(());
    }

    let safe_span = body
        .default_ingress_step_days()
        .max(config.convergence_days);
    if (t_b - t_a) > safe_span {
        let t_mid = 0.5 * (t_a + t_b);
        let idx_mid = rashi_index_of(transit_sidereal_longitude(engine, body, t_mid, config)?);
        collect_crossings(engine, body, config, t_a, idx_a, t_mid, idx_mid, out)?;
        collect_crossings(engine, body, config, t_mid, idx_mid, t_b, idx_b, out)?;
        return Ok(());
    }

    // Within the safe span the body moves well under 30 deg, so the index
    // delta is a truthful single-cusp crossing direction.
    let delta = (i32::from(idx_b) - i32::from(idx_a)).rem_euclid(12);
    let (boundary_deg, entered_index, is_retrograde) = if delta == 11 {
        // Stepped back one rashi: retrograde re-ingress across idx_a's cusp.
        (f64::from(idx_a) * 30.0, idx_b, true)
    } else {
        let entered = (idx_a + 1) % 12;
        (f64::from(entered) * 30.0 % 360.0, entered, false)
    };

    let f = |t: f64| -> Result<f64, SearchError> {
        let sid = transit_sidereal_longitude(engine, body, t, config)?;
        Ok(normalize_to_pm180(sid - boundary_deg))
    };

    let mut lo = t_a;
    let mut hi = t_b;
    let mut f_lo = f(lo)?;
    let f_hi = f(hi)?;
    if f_lo * f_hi > 0.0 {
        // The bracket does not straddle the cusp (numerical fluke); skip
        // rather than fabricate an event.
        return Ok(());
    }
    for _ in 0..config.max_iterations {
        let mid = 0.5 * (lo + hi);
        let f_mid = f(mid)?;
        if f_lo * f_mid <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
            f_lo = f_mid;
        }
        if (hi - lo).abs() < config.convergence_days {
            break;
        }
    }

    out.push(IngressCrossing {
        jd: 0.5 * (lo + hi),
        entered_index,
        is_retrograde,
    });
    Ok(())
}

/// Scan for the first rashi-boundary crossing from `jd_start` in the
/// direction of `step` (signed), up to `max_steps` samples.
///
/// The first sample's failure propagates; hitting the ephemeris coverage
/// edge mid-scan ends the scan with `Ok(None)`. Other engine errors
/// propagate.
fn scan_ingress(
    engine: &Engine,
    body: TransitBody,
    config: &SankrantiConfig,
    jd_start: f64,
    step: f64,
    max_steps: usize,
) -> Result<Option<IngressCrossing>, SearchError> {
    let mut t_prev = jd_start;
    let mut idx_prev = rashi_index_of(transit_sidereal_longitude(engine, body, t_prev, config)?);

    for _ in 0..max_steps {
        let t_curr = t_prev + step;
        let idx_curr = match transit_sidereal_longitude(engine, body, t_curr, config) {
            Ok(lon) => rashi_index_of(lon),
            Err(e) if is_coverage_edge(&e) => return Ok(None),
            Err(e) => return Err(e),
        };

        if idx_curr != idx_prev {
            let (t_a, idx_a, t_b, idx_b) = if t_prev < t_curr {
                (t_prev, idx_prev, t_curr, idx_curr)
            } else {
                (t_curr, idx_curr, t_prev, idx_prev)
            };
            let mut crossings = Vec::new();
            collect_crossings(engine, body, config, t_a, idx_a, t_b, idx_b, &mut crossings)?;
            // Forward scans want the earliest crossing in the interval,
            // backward scans the latest (nearest to the query time).
            let picked = if step > 0.0 {
                crossings.first()
            } else {
                crossings.last()
            };
            if let Some(&crossing) = picked {
                return Ok(Some(crossing));
            }
            // Aliased interval resolved to no crossing: keep scanning.
        }

        t_prev = t_curr;
        idx_prev = idx_curr;
    }

    Ok(None)
}

fn build_ingress_event(
    engine: &Engine,
    body: TransitBody,
    crossing: IngressCrossing,
    config: &SankrantiConfig,
) -> Result<SankrantiEvent, SearchError> {
    let tropical = transit_tropical_longitude(engine, body, crossing.jd, config)?;
    let sidereal = transit_sidereal_longitude(engine, body, crossing.jd, config)?;
    let rashi_index = crossing.entered_index % 12;
    Ok(SankrantiEvent {
        utc: UtcTime::from_jd_tdb(crossing.jd, engine.lsk()),
        body,
        rashi: ALL_RASHIS[rashi_index as usize],
        rashi_index,
        sidereal_longitude_deg: sidereal,
        tropical_longitude_deg: tropical,
        is_retrograde: crossing.is_retrograde,
    })
}

fn any_target_max_steps(body: TransitBody, config: &SankrantiConfig) -> usize {
    (body.ingress_max_scan_days() / config.step_size_days).ceil() as usize
}

/// Find the next rashi ingress of `body` after the given UTC time.
pub fn next_ingress(
    engine: &Engine,
    body: TransitBody,
    utc: &UtcTime,
    config: &SankrantiConfig,
) -> Result<Option<SankrantiEvent>, SearchError> {
    config.validate().map_err(SearchError::InvalidConfig)?;
    validate_ingress_body(body)?;

    let jd = crate::search_util::utc_to_jd_tdb(engine, utc);
    let crossing = scan_ingress(
        engine,
        body,
        config,
        jd,
        config.step_size_days,
        any_target_max_steps(body, config),
    )?;
    crossing
        .map(|c| build_ingress_event(engine, body, c, config))
        .transpose()
}

/// Find the previous rashi ingress of `body` before the given UTC time.
pub fn prev_ingress(
    engine: &Engine,
    body: TransitBody,
    utc: &UtcTime,
    config: &SankrantiConfig,
) -> Result<Option<SankrantiEvent>, SearchError> {
    config.validate().map_err(SearchError::InvalidConfig)?;
    validate_ingress_body(body)?;

    let jd = crate::search_util::utc_to_jd_tdb(engine, utc);
    let crossing = scan_ingress(
        engine,
        body,
        config,
        jd,
        -config.step_size_days,
        any_target_max_steps(body, config),
    )?;
    crossing
        .map(|c| build_ingress_event(engine, body, c, config))
        .transpose()
}

/// Search for all rashi ingresses of `body` in a UTC time range.
///
/// The scan is bounded by the range itself (no overshoot past `end`). A
/// range extending beyond the loaded ephemeris coverage errors, exactly as
/// the classical sankranti range search did.
pub fn search_ingresses(
    engine: &Engine,
    body: TransitBody,
    start: &UtcTime,
    end: &UtcTime,
    config: &SankrantiConfig,
) -> Result<Vec<SankrantiEvent>, SearchError> {
    config.validate().map_err(SearchError::InvalidConfig)?;
    validate_ingress_body(body)?;

    let jd_start = crate::search_util::utc_to_jd_tdb(engine, start);
    let jd_end = crate::search_util::utc_to_jd_tdb(engine, end);
    if jd_end <= jd_start {
        return Err(SearchError::InvalidConfig("end must be after start"));
    }

    let step = config.step_size_days;
    let mut events = Vec::new();

    let mut t_prev = jd_start;
    let mut idx_prev = rashi_index_of(transit_sidereal_longitude(engine, body, t_prev, config)?);

    loop {
        let t_curr = (t_prev + step).min(jd_end);
        let idx_curr = rashi_index_of(transit_sidereal_longitude(engine, body, t_curr, config)?);

        if idx_curr != idx_prev {
            let mut crossings = Vec::new();
            collect_crossings(
                engine,
                body,
                config,
                t_prev,
                idx_prev,
                t_curr,
                idx_curr,
                &mut crossings,
            )?;
            for crossing in crossings {
                if crossing.jd >= jd_start && crossing.jd <= jd_end {
                    events.push(build_ingress_event(engine, body, crossing, config)?);
                }
            }
        }

        if t_curr >= jd_end {
            break;
        }
        t_prev = t_curr;
        idx_prev = idx_curr;
    }

    Ok(events)
}

/// Find the next time `body` enters a specific rashi.
pub fn next_specific_ingress(
    engine: &Engine,
    body: TransitBody,
    utc: &UtcTime,
    rashi: Rashi,
    config: &SankrantiConfig,
) -> Result<Option<SankrantiEvent>, SearchError> {
    specific_ingress(engine, body, utc, rashi, config, 1.0)
}

/// Find the previous time `body` entered a specific rashi.
pub fn prev_specific_ingress(
    engine: &Engine,
    body: TransitBody,
    utc: &UtcTime,
    rashi: Rashi,
    config: &SankrantiConfig,
) -> Result<Option<SankrantiEvent>, SearchError> {
    specific_ingress(engine, body, utc, rashi, config, -1.0)
}

fn specific_ingress(
    engine: &Engine,
    body: TransitBody,
    utc: &UtcTime,
    rashi: Rashi,
    config: &SankrantiConfig,
    direction: f64,
) -> Result<Option<SankrantiEvent>, SearchError> {
    config.validate().map_err(SearchError::InvalidConfig)?;
    validate_ingress_body(body)?;

    let jd = crate::search_util::utc_to_jd_tdb(engine, utc);
    let step = direction * config.step_size_days;
    let window_days = SPECIFIC_TARGET_SCAN_FACTOR * body.ingress_max_scan_days();
    let window_end = jd + direction * window_days;

    let mut cursor = jd;
    loop {
        let remaining_days = (window_end - cursor) * direction;
        if remaining_days <= 0.0 {
            return Ok(None);
        }
        let max_steps = (remaining_days / config.step_size_days).ceil() as usize;
        let Some(crossing) = scan_ingress(engine, body, config, cursor, step, max_steps)? else {
            return Ok(None);
        };
        if crossing.entered_index == rashi.index() {
            return Ok(Some(build_ingress_event(engine, body, crossing, config)?));
        }
        cursor = crossing.jd + direction * SPECIFIC_RESUME_EPSILON_DAYS;
    }
}

// ---------------------------------------------------------------------------
// Classical Sun-based sankranti wrappers (existing public surface)
// ---------------------------------------------------------------------------

/// Find the next Sankranti (Sun entering any rashi) after the given UTC time.
pub fn next_sankranti(
    engine: &Engine,
    utc: &UtcTime,
    config: &SankrantiConfig,
) -> Result<Option<SankrantiEvent>, SearchError> {
    next_ingress(engine, TransitBody::Body(Body::Sun), utc, config)
}

/// Find the previous Sankranti (Sun entering any rashi) before the given UTC time.
pub fn prev_sankranti(
    engine: &Engine,
    utc: &UtcTime,
    config: &SankrantiConfig,
) -> Result<Option<SankrantiEvent>, SearchError> {
    prev_ingress(engine, TransitBody::Body(Body::Sun), utc, config)
}

/// Search for all Sankrantis in a UTC time range.
pub fn search_sankrantis(
    engine: &Engine,
    start: &UtcTime,
    end: &UtcTime,
    config: &SankrantiConfig,
) -> Result<Vec<SankrantiEvent>, SearchError> {
    search_ingresses(engine, TransitBody::Body(Body::Sun), start, end, config)
}

/// Find the next time the Sun enters a specific rashi.
pub fn next_specific_sankranti(
    engine: &Engine,
    utc: &UtcTime,
    rashi: Rashi,
    config: &SankrantiConfig,
) -> Result<Option<SankrantiEvent>, SearchError> {
    next_specific_ingress(engine, TransitBody::Body(Body::Sun), utc, rashi, config)
}

/// Find the previous time the Sun entered a specific rashi.
pub fn prev_specific_sankranti(
    engine: &Engine,
    utc: &UtcTime,
    rashi: Rashi,
    config: &SankrantiConfig,
) -> Result<Option<SankrantiEvent>, SearchError> {
    prev_specific_ingress(engine, TransitBody::Body(Body::Sun), utc, rashi, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dhruv_frames::DEFAULT_PRECESSION_MODEL;
    use dhruv_vedic_base::{AyanamshaSystem, NodeMode};

    #[test]
    fn rashi_index_basic() {
        assert_eq!(rashi_index_of(0.0), 0);
        assert_eq!(rashi_index_of(29.999), 0);
        assert_eq!(rashi_index_of(30.0), 1);
        assert_eq!(rashi_index_of(359.999), 11);
        assert_eq!(rashi_index_of(360.0), 0);
        assert_eq!(rashi_index_of(-10.0), 11);
    }

    #[test]
    fn config_validates() {
        let c = SankrantiConfig::default_lahiri();
        assert!(c.validate().is_ok());
    }

    #[test]
    fn config_rejects_zero_step() {
        let mut c = SankrantiConfig::default_lahiri();
        c.step_size_days = 0.0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn config_rejects_zero_iterations() {
        let mut c = SankrantiConfig::default_lahiri();
        c.max_iterations = 0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn default_lahiri_config() {
        let c = SankrantiConfig::default_lahiri();
        assert_eq!(c.ayanamsha_system, AyanamshaSystem::Lahiri);
        assert!(!c.use_nutation);
        assert_eq!(c.precession_model, DEFAULT_PRECESSION_MODEL);
        assert_eq!(c.node_mode, NodeMode::True);
    }

    #[test]
    fn for_body_uses_per_body_step() {
        let c = SankrantiConfig::for_body(
            AyanamshaSystem::Lahiri,
            false,
            TransitBody::Body(Body::Moon),
        );
        assert!((c.step_size_days - 0.25).abs() < 1e-12);
        let c = SankrantiConfig::for_body(AyanamshaSystem::Lahiri, false, TransitBody::Rahu);
        assert!((c.step_size_days - 1.0).abs() < 1e-12);
    }

    #[test]
    fn earth_rejected_for_ingress() {
        assert!(validate_ingress_body(TransitBody::Body(Body::Earth)).is_err());
        assert!(validate_ingress_body(TransitBody::Body(Body::Moon)).is_ok());
        assert!(validate_ingress_body(TransitBody::Rahu).is_ok());
    }
}
