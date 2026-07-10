//! Exact varga-lagna transition events over a time range.
//!
//! For each requested amsha, returns the stream of varga-lagna rashi
//! segments overlapping `[from, to]` with root-found boundary times —
//! no sampling grid, so fast vargas (e.g. D60) cannot alias between
//! samples and segment boundaries are astronomically exact.
//!
//! The varga rashi of the ascendant changes only when the D1 ascendant
//! crosses a fixed division-boundary longitude (pure zodiac geometry), and
//! the ascendant advances monotonically (~360 degrees/day), so each
//! transition is a single seeded root-find on the ascendant longitude.

use dhruv_core::Engine;
use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::amsha::{amsha_rashi_info, next_amsha_boundary_longitude};
use dhruv_vedic_base::riseset_types::GeoLocation;
use dhruv_vedic_base::{Amsha, AmshaRequest, Rashi};

use crate::error::SearchError;
use crate::jyotish::{
    sidereal_lagna_for_date, unique_amsha_requests_for_compute, validate_amsha_requests,
};
use crate::sankranti_types::SankrantiConfig;
use crate::search_util::{find_zero_crossing, normalize_to_pm180, utc_to_jd_tdb_with_eop};

/// Hard ceiling on segments returned by a single `amsha_lagna_events` call.
pub const MAX_AMSHA_LAGNA_SEGMENTS: u32 = 50_000;

/// Offset (days, ~0.4 s) past a found boundary used to classify the next
/// segment.
const BOUNDARY_PROBE_DAYS: f64 = 5e-6;

/// One varga-lagna rashi segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmshaLagnaSegment {
    pub rashi: Rashi,
    /// 0-based rashi index (0-11).
    pub rashi_index: u8,
    /// Segment start (UTC). The first segment of a sweep starts at the
    /// requested `from`; later segments start at the exact transition.
    pub start: UtcTime,
    /// Segment end (UTC): the exact transition time. The last segment's end
    /// is the first transition at or after `to`.
    pub end: UtcTime,
}

/// Segment stream for one requested amsha.
#[derive(Debug, Clone)]
pub struct AmshaLagnaEvents {
    pub amsha: Amsha,
    pub variation_code: u8,
    pub segments: Vec<AmshaLagnaSegment>,
}

/// Result of an `amsha_lagna_events` sweep.
///
/// Entries are in request order with duplicate requests collapsed.
#[derive(Debug, Clone)]
pub struct AmshaLagnaEventsResult {
    pub entries: Vec<AmshaLagnaEvents>,
    /// True when the segment budget was exhausted before `to_utc`.
    pub truncated: bool,
    /// Resume point when truncated: re-invoke with `from_utc =
    /// next_from_utc`. The resumed sweep re-yields, per amsha, the segment
    /// containing the resume point (with its start clipped to it).
    pub next_from_utc: Option<UtcTime>,
}

/// Sweep cursor for one amsha request.
struct RequestSweep {
    request: AmshaRequest,
    /// Classified rashi of the pending (unemitted) segment.
    rashi: Rashi,
    rashi_index: u8,
    seg_start_utc: UtcTime,
    seg_start_jd: f64,
    /// Root-found end of the pending segment.
    end_jd: f64,
    end_utc: UtcTime,
    active: bool,
}

/// Find the time the sidereal ascendant reaches `delta_deg` degrees ahead
/// of its position at `t_jd` (`lagna_deg`), seeded by the nominal rate of
/// 360 degrees/day.
fn lagna_crossing_time(
    engine: &Engine,
    eop: &EopKernel,
    location: &GeoLocation,
    aya_config: &SankrantiConfig,
    t_jd: f64,
    lagna_deg: f64,
    delta_deg: f64,
) -> Result<f64, SearchError> {
    let target = (lagna_deg + delta_deg).rem_euclid(360.0);
    let f = |t: f64| -> Result<f64, SearchError> {
        let utc = UtcTime::from_jd_tdb(t, engine.lsk());
        let lagna = sidereal_lagna_for_date(engine, eop, &utc, location, aya_config)?;
        Ok(normalize_to_pm180(lagna - target))
    };
    // Nominal estimate; the true ascendant rate varies with latitude and
    // obliquity, so the scan window is an order of magnitude wider.
    let est_days = delta_deg / 360.0;
    let step = (est_days / 4.0).max(30.0 / 86_400.0);
    find_zero_crossing(&f, t_jd, step, 48, 60, 1e-8)?.ok_or(SearchError::NoConvergence(
        "could not bracket varga lagna transition (non-monotonic ascendant?)",
    ))
}

fn init_sweep(
    engine: &Engine,
    eop: &EopKernel,
    location: &GeoLocation,
    aya_config: &SankrantiConfig,
    request: AmshaRequest,
    from_utc: &UtcTime,
    from_jd: f64,
) -> Result<RequestSweep, SearchError> {
    let lagna = sidereal_lagna_for_date(engine, eop, from_utc, location, aya_config)?;
    let info = amsha_rashi_info(lagna, request.amsha, request.variation);
    let boundary = next_amsha_boundary_longitude(lagna, request.amsha, request.variation);
    let end_jd = lagna_crossing_time(
        engine,
        eop,
        location,
        aya_config,
        from_jd,
        lagna,
        boundary - lagna,
    )?;
    Ok(RequestSweep {
        request,
        rashi: info.rashi,
        rashi_index: info.rashi_index,
        seg_start_utc: *from_utc,
        seg_start_jd: from_jd,
        end_jd,
        end_utc: UtcTime::from_jd_tdb(end_jd, engine.lsk()),
        active: true,
    })
}

impl RequestSweep {
    /// Advance past the pending segment's end boundary: classify the next
    /// segment and root-find its end.
    fn advance(
        &mut self,
        engine: &Engine,
        eop: &EopKernel,
        location: &GeoLocation,
        aya_config: &SankrantiConfig,
    ) -> Result<(), SearchError> {
        let prev_index = self.rashi_index;
        let mut probe_jd = self.end_jd + BOUNDARY_PROBE_DAYS;
        let mut lagna = 0.0;
        let mut info = None;
        // The root-find converges to ~1 ms of the true crossing; step the
        // probe outward until the classification actually flips.
        for widen in 0..4 {
            let utc = UtcTime::from_jd_tdb(probe_jd, engine.lsk());
            lagna = sidereal_lagna_for_date(engine, eop, &utc, location, aya_config)?;
            let candidate = amsha_rashi_info(lagna, self.request.amsha, self.request.variation);
            if candidate.rashi_index != prev_index {
                info = Some(candidate);
                break;
            }
            probe_jd += BOUNDARY_PROBE_DAYS * 10f64.powi(widen + 1);
        }
        let info = info.ok_or(SearchError::NoConvergence(
            "varga lagna did not change past a computed boundary",
        ))?;

        let boundary = next_amsha_boundary_longitude(lagna, self.request.amsha, self.request.variation);
        let next_end_jd = lagna_crossing_time(
            engine,
            eop,
            location,
            aya_config,
            probe_jd,
            lagna,
            boundary - lagna,
        )?;

        self.seg_start_utc = self.end_utc;
        self.seg_start_jd = self.end_jd;
        self.rashi = info.rashi;
        self.rashi_index = info.rashi_index;
        self.end_jd = next_end_jd;
        self.end_utc = UtcTime::from_jd_tdb(next_end_jd, engine.lsk());
        Ok(())
    }
}

/// Stream exact varga-lagna rashi segments overlapping `[from_utc, to_utc]`
/// for each requested amsha.
///
/// `max_segments` caps the total segments across all amshas (`0` selects
/// [`MAX_AMSHA_LAGNA_SEGMENTS`]); on truncation `next_from_utc` gives the
/// resume point. Requests follow the usual amsha batch rules (at most 40,
/// valid variations; duplicates collapsed).
pub fn amsha_lagna_events(
    engine: &Engine,
    eop: &EopKernel,
    from_utc: &UtcTime,
    to_utc: &UtcTime,
    location: &GeoLocation,
    aya_config: &SankrantiConfig,
    requests: &[AmshaRequest],
    max_segments: u32,
) -> Result<AmshaLagnaEventsResult, SearchError> {
    if requests.is_empty() {
        return Err(SearchError::InvalidConfig(
            "amsha_requests must be non-empty",
        ));
    }
    validate_amsha_requests(requests)?;
    let (unique_requests, _) = unique_amsha_requests_for_compute(requests);

    let from_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), from_utc);
    let to_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), to_utc);
    if to_jd <= from_jd {
        return Err(SearchError::InvalidConfig(
            "to_utc must be after from_utc",
        ));
    }
    let cap = if max_segments == 0 {
        MAX_AMSHA_LAGNA_SEGMENTS
    } else {
        max_segments.min(MAX_AMSHA_LAGNA_SEGMENTS)
    };

    let mut sweeps: Vec<RequestSweep> = Vec::with_capacity(unique_requests.len());
    for request in &unique_requests {
        sweeps.push(init_sweep(
            engine, eop, location, aya_config, *request, from_utc, from_jd,
        )?);
    }
    let mut entries: Vec<AmshaLagnaEvents> = unique_requests
        .iter()
        .map(|request| AmshaLagnaEvents {
            amsha: request.amsha,
            variation_code: request.effective_variation(),
            segments: Vec::new(),
        })
        .collect();

    let mut truncated = false;
    let mut next_from_utc = None;
    let mut total: u32 = 0;

    loop {
        // Emit in global boundary-time order so all amshas progress
        // together; truncation then leaves at most one unemitted segment
        // per amsha.
        let next = sweeps
            .iter()
            .enumerate()
            .filter(|(_, s)| s.active)
            .min_by(|a, b| a.1.end_jd.total_cmp(&b.1.end_jd))
            .map(|(i, _)| i);
        let Some(i) = next else { break };

        if total >= cap {
            truncated = true;
            let min_start = sweeps
                .iter()
                .filter(|s| s.active)
                .map(|s| s.seg_start_jd)
                .fold(f64::INFINITY, f64::min);
            next_from_utc = Some(UtcTime::from_jd_tdb(min_start, engine.lsk()));
            break;
        }

        let sweep = &sweeps[i];
        entries[i].segments.push(AmshaLagnaSegment {
            rashi: sweep.rashi,
            rashi_index: sweep.rashi_index,
            start: sweep.seg_start_utc,
            end: sweep.end_utc,
        });
        total += 1;

        if sweeps[i].end_jd >= to_jd {
            sweeps[i].active = false;
        } else {
            sweeps[i].advance(engine, eop, location, aya_config)?;
        }
    }

    Ok(AmshaLagnaEventsResult {
        entries,
        truncated,
        next_from_utc,
    })
}
