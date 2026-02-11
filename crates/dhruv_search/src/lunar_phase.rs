//! Purnima (full moon) and Amavasya (new moon) search.
//!
//! Thin wrappers around the conjunction engine. Purnima is Sun-Moon opposition
//! (180 deg), Amavasya is Sun-Moon conjunction (0 deg).
//!
//! All public functions accept and return UTC times; JD TDB is internal only.

use dhruv_core::{Body, Engine};
use dhruv_time::UtcTime;

use crate::conjunction::{next_conjunction, prev_conjunction, search_conjunctions};
use crate::conjunction_types::ConjunctionConfig;
use crate::error::SearchError;
use crate::lunar_phase_types::{LunarPhase, LunarPhaseEvent};

/// Step size for lunar phase search (days).
/// Half a day gives reliable detection of the ~29.53-day synodic cycle.
const LUNAR_STEP_DAYS: f64 = 0.5;

fn make_config(target_deg: f64) -> ConjunctionConfig {
    ConjunctionConfig {
        target_separation_deg: target_deg,
        step_size_days: LUNAR_STEP_DAYS,
        max_iterations: 50,
        convergence_days: 1e-8,
    }
}

fn conjunction_to_phase(
    event: &crate::conjunction_types::ConjunctionEvent,
    phase: LunarPhase,
    lsk: &dhruv_time::LeapSecondKernel,
) -> LunarPhaseEvent {
    LunarPhaseEvent {
        utc: UtcTime::from_jd_tdb(event.jd_tdb, lsk),
        phase,
        // body1=Sun, body2=Moon
        sun_longitude_deg: event.body1_longitude_deg,
        moon_longitude_deg: event.body2_longitude_deg,
    }
}

/// Find the next Purnima (full moon) after the given UTC time.
pub fn next_purnima(
    engine: &Engine,
    utc: &UtcTime,
) -> Result<Option<LunarPhaseEvent>, SearchError> {
    let jd = utc.to_jd_tdb(engine.lsk());
    let config = make_config(180.0);
    let result = next_conjunction(engine, Body::Sun, Body::Moon, jd, &config)?;
    Ok(result.map(|e| conjunction_to_phase(&e, LunarPhase::FullMoon, engine.lsk())))
}

/// Find the previous Purnima (full moon) before the given UTC time.
pub fn prev_purnima(
    engine: &Engine,
    utc: &UtcTime,
) -> Result<Option<LunarPhaseEvent>, SearchError> {
    let jd = utc.to_jd_tdb(engine.lsk());
    let config = make_config(180.0);
    let result = prev_conjunction(engine, Body::Sun, Body::Moon, jd, &config)?;
    Ok(result.map(|e| conjunction_to_phase(&e, LunarPhase::FullMoon, engine.lsk())))
}

/// Find the next Amavasya (new moon) after the given UTC time.
pub fn next_amavasya(
    engine: &Engine,
    utc: &UtcTime,
) -> Result<Option<LunarPhaseEvent>, SearchError> {
    let jd = utc.to_jd_tdb(engine.lsk());
    let config = make_config(0.0);
    let result = next_conjunction(engine, Body::Sun, Body::Moon, jd, &config)?;
    Ok(result.map(|e| conjunction_to_phase(&e, LunarPhase::NewMoon, engine.lsk())))
}

/// Find the previous Amavasya (new moon) before the given UTC time.
pub fn prev_amavasya(
    engine: &Engine,
    utc: &UtcTime,
) -> Result<Option<LunarPhaseEvent>, SearchError> {
    let jd = utc.to_jd_tdb(engine.lsk());
    let config = make_config(0.0);
    let result = prev_conjunction(engine, Body::Sun, Body::Moon, jd, &config)?;
    Ok(result.map(|e| conjunction_to_phase(&e, LunarPhase::NewMoon, engine.lsk())))
}

/// Search for all Purnimas (full moons) in a UTC time range.
pub fn search_purnimas(
    engine: &Engine,
    start: &UtcTime,
    end: &UtcTime,
) -> Result<Vec<LunarPhaseEvent>, SearchError> {
    let jd_start = start.to_jd_tdb(engine.lsk());
    let jd_end = end.to_jd_tdb(engine.lsk());
    let config = make_config(180.0);
    let events = search_conjunctions(engine, Body::Sun, Body::Moon, jd_start, jd_end, &config)?;
    Ok(events
        .iter()
        .map(|e| conjunction_to_phase(e, LunarPhase::FullMoon, engine.lsk()))
        .collect())
}

/// Search for all Amavasyas (new moons) in a UTC time range.
pub fn search_amavasyas(
    engine: &Engine,
    start: &UtcTime,
    end: &UtcTime,
) -> Result<Vec<LunarPhaseEvent>, SearchError> {
    let jd_start = start.to_jd_tdb(engine.lsk());
    let jd_end = end.to_jd_tdb(engine.lsk());
    let config = make_config(0.0);
    let events = search_conjunctions(engine, Body::Sun, Body::Moon, jd_start, jd_end, &config)?;
    Ok(events
        .iter()
        .map(|e| conjunction_to_phase(e, LunarPhase::NewMoon, engine.lsk()))
        .collect())
}
