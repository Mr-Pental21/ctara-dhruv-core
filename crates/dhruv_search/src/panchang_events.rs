//! Range sweep over panchang element boundaries.
//!
//! Produces the full stream of element segments overlapping a UTC range in
//! a single call, instead of one per-moment call per day. Each consecutive
//! boundary search is seeded from the previous boundary, so a sweep costs
//! roughly one root-find per emitted segment; the sunrise-anchored elements
//! (vaar, hora, ghatika) cost one sunrise search per Vedic day and pure
//! arithmetic for the subdivisions.
//!
//! A location is required only when a location-dependent element is
//! selected; location-independent selections take no location at all.

use dhruv_core::Engine;
use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::riseset_types::{GeoLocation, RiseSetConfig};
use dhruv_vedic_base::{
    HORA_COUNT, KARANA_SEGMENT_DEG, NAKSHATRA_SPAN_27, TITHI_SEGMENT_DEG, YOGA_SEGMENT_DEG,
    karana_from_elongation, nakshatra_from_longitude, tithi_from_elongation, yoga_from_sum,
};

use crate::error::SearchError;
use crate::operations::{
    PANCHANG_INCLUDE_ALL, PANCHANG_INCLUDE_AYANA, PANCHANG_INCLUDE_GHATIKA, PANCHANG_INCLUDE_HORA,
    PANCHANG_INCLUDE_KARANA, PANCHANG_INCLUDE_LOCATION_DEPENDENT, PANCHANG_INCLUDE_MASA,
    PANCHANG_INCLUDE_NAKSHATRA, PANCHANG_INCLUDE_TITHI, PANCHANG_INCLUDE_VAAR,
    PANCHANG_INCLUDE_VARSHA, PANCHANG_INCLUDE_YOGA,
};
use crate::panchang::{
    ayana_for_date_with_eop, elongation_at, find_angle_boundary, ghatika_from_sunrises,
    hora_from_sunrises, karana_at, masa_for_date_with_eop, moon_sidereal_longitude_at,
    nakshatra_at, sidereal_sum_at, tithi_at, vaar_from_sunrises, varsha_for_date_with_eop,
    vedic_day_sunrises, yoga_at,
};
use crate::panchang_types::{
    AyanaInfo, GhatikaInfo, HoraInfo, KaranaInfo, MasaInfo, PanchangNakshatraInfo, TithiInfo,
    VaarInfo, VarshaInfo, YogaInfo,
};
use crate::sankranti_types::SankrantiConfig;
use crate::search_util::utc_to_jd_tdb_with_eop;

/// Hard ceiling on events returned by a single `panchang_events` call.
pub const MAX_PANCHANG_EVENTS: u32 = 50_000;

/// Offset (days) past a calendar-element boundary used to classify the next
/// segment (~30 minutes; well inside any masa/ayana/varsha).
const CALENDAR_ADVANCE_DAYS: f64 = 0.02;

/// Tolerance (days, ~1 minute) within which a calendar segment's
/// independently root-found start is snapped to the previous segment's end,
/// so consecutive segments chain exactly instead of differing by root-find
/// jitter (sub-millisecond).
const CALENDAR_SNAP_DAYS: f64 = 7e-4;

/// Panchang boundary events over a range, one vector per selected element.
///
/// Every entry reuses the per-moment `*Info` shape: `start`/`end` are the
/// exact segment boundaries. The first segment of each kind may start
/// before `from_utc` and the last may end after `to_utc` (segments overlap
/// the range; they are not clipped). For nakshatra events the `pada` field
/// refers to the segment start and is always 1; use the per-moment API for
/// the pada of a specific instant.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PanchangEventsResult {
    pub tithi: Vec<TithiInfo>,
    pub karana: Vec<KaranaInfo>,
    pub yoga: Vec<YogaInfo>,
    pub nakshatra: Vec<PanchangNakshatraInfo>,
    pub vaar: Vec<VaarInfo>,
    pub hora: Vec<HoraInfo>,
    pub ghatika: Vec<GhatikaInfo>,
    pub masa: Vec<MasaInfo>,
    pub ayana: Vec<AyanaInfo>,
    pub varsha: Vec<VarshaInfo>,
    /// True when the event budget was exhausted before `to_utc`.
    pub truncated: bool,
    /// Resume point when truncated: re-invoke with `from_utc = next_from_utc`.
    /// Resuming may re-emit, per kind, the one segment containing the resume
    /// point; callers should deduplicate on `(kind, start)`.
    pub next_from_utc: Option<UtcTime>,
}

/// Angular element selector for the shared sweep logic.
#[derive(Debug, Clone, Copy, PartialEq)]
enum AngularKind {
    Tithi,
    Karana,
    Yoga,
    Nakshatra,
}

impl AngularKind {
    fn segment_deg(self) -> f64 {
        match self {
            Self::Tithi => TITHI_SEGMENT_DEG,
            Self::Karana => KARANA_SEGMENT_DEG,
            Self::Yoga => YOGA_SEGMENT_DEG,
            Self::Nakshatra => NAKSHATRA_SPAN_27,
        }
    }

    fn segment_count(self) -> u16 {
        match self {
            Self::Tithi => 30,
            Self::Karana => 60,
            Self::Yoga => 27,
            Self::Nakshatra => 27,
        }
    }

    /// Coarse scan step (days) for the forward boundary search; sized well
    /// under the shortest possible segment duration.
    fn scan_step_days(self) -> f64 {
        match self {
            Self::Karana => 0.2,
            _ => 0.25,
        }
    }

    fn value_at(
        self,
        engine: &Engine,
        jd_tdb: f64,
        config: &SankrantiConfig,
    ) -> Result<f64, SearchError> {
        match self {
            Self::Tithi | Self::Karana => elongation_at(engine, jd_tdb),
            Self::Yoga => sidereal_sum_at(engine, jd_tdb, config),
            Self::Nakshatra => moon_sidereal_longitude_at(engine, jd_tdb, config),
        }
    }
}

/// Sweep cursor for one selected element.
enum Sweeper {
    Angular {
        kind: AngularKind,
        index: u16,
        start_jd: f64,
        end_jd: f64,
        end_utc: UtcTime,
        pending: PendingAngular,
    },
    Masa {
        current: MasaInfo,
        start_jd: f64,
        end_jd: f64,
    },
    Ayana {
        current: AyanaInfo,
        start_jd: f64,
        end_jd: f64,
    },
    Varsha {
        current: VarshaInfo,
        start_jd: f64,
        end_jd: f64,
    },
    Vaar {
        current: VaarInfo,
        day: VedicDayCursor,
    },
    Hora {
        current: HoraInfo,
        day: VedicDayCursor,
        /// 0-based hora index of the pending segment within the Vedic day.
        index: u16,
        start_jd: f64,
        end_jd: f64,
    },
    Ghatika {
        current: GhatikaInfo,
        day: VedicDayCursor,
        /// 0-based ghatika index of the pending segment within the Vedic day.
        index: u16,
        start_jd: f64,
        end_jd: f64,
    },
}

/// Sunrise-to-sunrise bracket (JD TDB) shared by the sunrise-anchored
/// sweepers; one sunrise search per Vedic day, subdivisions are arithmetic.
#[derive(Debug, Clone, Copy)]
struct VedicDayCursor {
    start_jd: f64,
    end_jd: f64,
    start_utc: UtcTime,
    end_utc: UtcTime,
}

impl VedicDayCursor {
    fn at(
        engine: &Engine,
        eop: &EopKernel,
        utc: &UtcTime,
        location: &GeoLocation,
        riseset_config: &RiseSetConfig,
    ) -> Result<Self, SearchError> {
        let (start_jd, end_jd) = vedic_day_sunrises(engine, eop, utc, location, riseset_config)?;
        Ok(Self {
            start_jd,
            end_jd,
            start_utc: UtcTime::from_jd_tdb(start_jd, engine.lsk()),
            end_utc: UtcTime::from_jd_tdb(end_jd, engine.lsk()),
        })
    }

    /// Bracket of the following Vedic day; the recomputed shared sunrise is
    /// snapped to this bracket's end so consecutive days chain exactly.
    fn next(
        &self,
        engine: &Engine,
        eop: &EopKernel,
        location: &GeoLocation,
        riseset_config: &RiseSetConfig,
    ) -> Result<Self, SearchError> {
        let probe = UtcTime::from_jd_tdb(self.end_jd + 0.5, engine.lsk());
        let mut next = Self::at(engine, eop, &probe, location, riseset_config)?;
        if (next.start_jd - self.end_jd).abs() < CALENDAR_SNAP_DAYS {
            next.start_jd = self.end_jd;
            next.start_utc = self.end_utc;
        }
        Ok(next)
    }

    /// [start, end) (JD TDB) of the `index`-th of `count` equal divisions.
    fn division(&self, index: u16, count: u16) -> (f64, f64) {
        let len = (self.end_jd - self.start_jd) / count as f64;
        let start = self.start_jd + index as f64 * len;
        (start, start + len)
    }

    /// Moment safely inside the `index`-th of `count` equal divisions.
    fn division_probe(&self, index: u16, count: u16) -> f64 {
        let (start, end) = self.division(index, count);
        0.5 * (start + end)
    }
}

/// The classified-but-unemitted angular segment held by an angular sweeper.
enum PendingAngular {
    Tithi(TithiInfo),
    Karana(KaranaInfo),
    Yoga(YogaInfo),
    Nakshatra(PanchangNakshatraInfo),
}

impl Sweeper {
    /// End (JD TDB) of the pending segment; drives the interleaved emission
    /// order.
    fn end_jd(&self) -> f64 {
        match self {
            Self::Angular { end_jd, .. }
            | Self::Masa { end_jd, .. }
            | Self::Ayana { end_jd, .. }
            | Self::Varsha { end_jd, .. }
            | Self::Hora { end_jd, .. }
            | Self::Ghatika { end_jd, .. } => *end_jd,
            Self::Vaar { day, .. } => day.end_jd,
        }
    }

    /// Start (JD TDB) of the pending segment; used for the resume point.
    fn start_jd(&self) -> f64 {
        match self {
            Self::Angular { start_jd, .. }
            | Self::Masa { start_jd, .. }
            | Self::Ayana { start_jd, .. }
            | Self::Varsha { start_jd, .. }
            | Self::Hora { start_jd, .. }
            | Self::Ghatika { start_jd, .. } => *start_jd,
            Self::Vaar { day, .. } => day.start_jd,
        }
    }

    /// Append the pending segment to the result.
    fn emit(&self, result: &mut PanchangEventsResult) {
        match self {
            Self::Angular { pending, .. } => match pending {
                PendingAngular::Tithi(info) => result.tithi.push(*info),
                PendingAngular::Karana(info) => result.karana.push(*info),
                PendingAngular::Yoga(info) => result.yoga.push(*info),
                PendingAngular::Nakshatra(info) => result.nakshatra.push(*info),
            },
            Self::Masa { current, .. } => result.masa.push(*current),
            Self::Ayana { current, .. } => result.ayana.push(*current),
            Self::Varsha { current, .. } => result.varsha.push(*current),
            Self::Vaar { current, .. } => result.vaar.push(*current),
            Self::Hora { current, .. } => result.hora.push(*current),
            Self::Ghatika { current, .. } => result.ghatika.push(*current),
        }
    }

    /// Replace the pending segment with the next one, seeding the boundary
    /// search from the previous boundary.
    fn advance(
        &mut self,
        engine: &Engine,
        eop: &EopKernel,
        config: &SankrantiConfig,
        location: Option<&GeoLocation>,
        riseset_config: &RiseSetConfig,
    ) -> Result<(), SearchError> {
        match self {
            Self::Angular {
                kind,
                index,
                start_jd,
                end_jd,
                end_utc,
                pending,
            } => {
                let next_index = (*index + 1) % kind.segment_count();
                let segment_deg = kind.segment_deg();
                let target = ((next_index as f64 + 1.0) * segment_deg) % 360.0;
                let f = |t: f64| kind.value_at(engine, t, config);
                let next_end_jd =
                    find_angle_boundary(&f, *end_jd, target, kind.scan_step_days(), 24)?.ok_or(
                        SearchError::NoConvergence("could not find next panchang boundary"),
                    )?;
                let start_utc = *end_utc;
                let next_end_utc = UtcTime::from_jd_tdb(next_end_jd, engine.lsk());
                let mid_deg = (next_index as f64 + 0.5) * segment_deg;
                *pending = match kind {
                    AngularKind::Tithi => {
                        let pos = tithi_from_elongation(mid_deg);
                        PendingAngular::Tithi(TithiInfo {
                            tithi: pos.tithi,
                            tithi_index: pos.tithi_index,
                            paksha: pos.paksha,
                            tithi_in_paksha: pos.tithi_in_paksha,
                            start: start_utc,
                            end: next_end_utc,
                        })
                    }
                    AngularKind::Karana => {
                        let pos = karana_from_elongation(mid_deg);
                        PendingAngular::Karana(KaranaInfo {
                            karana: pos.karana,
                            karana_index: pos.karana_index,
                            start: start_utc,
                            end: next_end_utc,
                        })
                    }
                    AngularKind::Yoga => {
                        let pos = yoga_from_sum(mid_deg);
                        PendingAngular::Yoga(YogaInfo {
                            yoga: pos.yoga,
                            yoga_index: pos.yoga_index,
                            start: start_utc,
                            end: next_end_utc,
                        })
                    }
                    AngularKind::Nakshatra => {
                        // Classify at the segment start so `pada` is the
                        // segment-start pada (1); the nakshatra itself is
                        // constant across the segment.
                        let pos =
                            nakshatra_from_longitude(next_index as f64 * segment_deg + 1e-9);
                        PendingAngular::Nakshatra(PanchangNakshatraInfo {
                            nakshatra: pos.nakshatra,
                            nakshatra_index: pos.nakshatra_index,
                            pada: pos.pada,
                            start: start_utc,
                            end: next_end_utc,
                        })
                    }
                };
                *index = next_index;
                *start_jd = *end_jd;
                *end_jd = next_end_jd;
                *end_utc = next_end_utc;
                Ok(())
            }
            Self::Masa {
                current,
                start_jd,
                end_jd,
            } => {
                let probe = UtcTime::from_jd_tdb(*end_jd + CALENDAR_ADVANCE_DAYS, engine.lsk());
                let mut next = masa_for_date_with_eop(engine, Some(eop), &probe, config)?;
                snap_start(engine, eop, &mut next.start, current.end, *end_jd);
                *start_jd = *end_jd;
                *end_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), &next.end);
                *current = next;
                Ok(())
            }
            Self::Ayana {
                current,
                start_jd,
                end_jd,
            } => {
                let probe = UtcTime::from_jd_tdb(*end_jd + CALENDAR_ADVANCE_DAYS, engine.lsk());
                let mut next = ayana_for_date_with_eop(engine, Some(eop), &probe, config)?;
                snap_start(engine, eop, &mut next.start, current.end, *end_jd);
                *start_jd = *end_jd;
                *end_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), &next.end);
                *current = next;
                Ok(())
            }
            Self::Varsha {
                current,
                start_jd,
                end_jd,
            } => {
                let probe = UtcTime::from_jd_tdb(*end_jd + CALENDAR_ADVANCE_DAYS, engine.lsk());
                let mut next = varsha_for_date_with_eop(engine, Some(eop), &probe, config)?;
                snap_start(engine, eop, &mut next.start, current.end, *end_jd);
                *start_jd = *end_jd;
                *end_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), &next.end);
                *current = next;
                Ok(())
            }
            Self::Vaar { current, day } => {
                let location = location.expect("validated: location present");
                let next_day = day.next(engine, eop, location, riseset_config)?;
                *current = vaar_from_sunrises(next_day.start_jd, next_day.end_jd, engine.lsk());
                *day = next_day;
                Ok(())
            }
            Self::Hora {
                current,
                day,
                index,
                start_jd,
                end_jd,
            } => {
                let count = HORA_COUNT as u16;
                let next_index = if *index + 1 < count {
                    *index + 1
                } else {
                    let location = location.expect("validated: location present");
                    *day = day.next(engine, eop, location, riseset_config)?;
                    0
                };
                let prev_end = current.end;
                let probe = day.division_probe(next_index, count);
                let mut next = hora_from_sunrises(probe, day.start_jd, day.end_jd, engine.lsk());
                // Chain exactly: subdivision arithmetic can differ from the
                // previous end by one ulp.
                next.start = prev_end;
                let (s, e) = day.division(next_index, count);
                *current = next;
                *index = next_index;
                *start_jd = s;
                *end_jd = e;
                Ok(())
            }
            Self::Ghatika {
                current,
                day,
                index,
                start_jd,
                end_jd,
            } => {
                let count: u16 = 60;
                let next_index = if *index + 1 < count {
                    *index + 1
                } else {
                    let location = location.expect("validated: location present");
                    *day = day.next(engine, eop, location, riseset_config)?;
                    0
                };
                let prev_end = current.end;
                let probe = day.division_probe(next_index, count);
                let mut next =
                    ghatika_from_sunrises(probe, day.start_jd, day.end_jd, engine.lsk());
                next.start = prev_end;
                let (s, e) = day.division(next_index, count);
                *current = next;
                *index = next_index;
                *start_jd = s;
                *end_jd = e;
                Ok(())
            }
        }
    }
}

/// Snap `start` to `prev_end` when they re-found the same boundary (within
/// [`CALENDAR_SNAP_DAYS`]); leaves it untouched otherwise.
fn snap_start(
    engine: &Engine,
    eop: &EopKernel,
    start: &mut UtcTime,
    prev_end: UtcTime,
    prev_end_jd: f64,
) {
    let start_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), start);
    if (start_jd - prev_end_jd).abs() < CALENDAR_SNAP_DAYS {
        *start = prev_end;
    }
}

fn angular_sweeper(
    engine: &Engine,
    eop: &EopKernel,
    kind: AngularKind,
    from_jd: f64,
    config: &SankrantiConfig,
) -> Result<Sweeper, SearchError> {
    let value = kind.value_at(engine, from_jd, config)?;
    let (index, pending, start, end) = match kind {
        AngularKind::Tithi => {
            let info = tithi_at(engine, from_jd, value)?;
            (
                info.tithi_index as u16,
                PendingAngular::Tithi(info),
                info.start,
                info.end,
            )
        }
        AngularKind::Karana => {
            let info = karana_at(engine, from_jd, value)?;
            (
                info.karana_index as u16,
                PendingAngular::Karana(info),
                info.start,
                info.end,
            )
        }
        AngularKind::Yoga => {
            let info = yoga_at(engine, from_jd, value, config)?;
            (
                info.yoga_index as u16,
                PendingAngular::Yoga(info),
                info.start,
                info.end,
            )
        }
        AngularKind::Nakshatra => {
            let info = nakshatra_at(engine, from_jd, value, config)?;
            (
                info.nakshatra_index as u16,
                PendingAngular::Nakshatra(info),
                info.start,
                info.end,
            )
        }
    };
    Ok(Sweeper::Angular {
        kind,
        index,
        start_jd: utc_to_jd_tdb_with_eop(engine, Some(eop), &start),
        end_jd: utc_to_jd_tdb_with_eop(engine, Some(eop), &end),
        end_utc: end,
        pending,
    })
}

/// Stream panchang element segments overlapping `[from_utc, to_utc]`.
///
/// `include_mask` selects elements with the usual `PANCHANG_INCLUDE_*`
/// bits. A `location` (with `riseset_config`) is required only when a
/// location-dependent element (vaar, hora, ghatika) is selected; it may be
/// `None` otherwise. `max_events` caps the total number of returned
/// segments across all kinds (`0` selects the hard ceiling
/// [`MAX_PANCHANG_EVENTS`]); when the cap is reached the result is marked
/// `truncated` and `next_from_utc` gives the resume point.
///
/// Segments are exact: consecutive segments of one kind share a boundary
/// (`end == next.start`), and boundary times match the per-moment API.
/// Sunrise-anchored kinds cost one sunrise search per Vedic day; hora and
/// ghatika subdivisions are arithmetic.
#[allow(clippy::too_many_arguments)]
pub fn panchang_events(
    engine: &Engine,
    eop: &EopKernel,
    from_utc: &UtcTime,
    to_utc: &UtcTime,
    include_mask: u32,
    location: Option<&GeoLocation>,
    riseset_config: &RiseSetConfig,
    config: &SankrantiConfig,
    max_events: u32,
) -> Result<PanchangEventsResult, SearchError> {
    if include_mask & PANCHANG_INCLUDE_ALL == 0 {
        return Err(SearchError::InvalidConfig(
            "include_mask must select at least one element",
        ));
    }
    if include_mask & PANCHANG_INCLUDE_LOCATION_DEPENDENT != 0 && location.is_none() {
        return Err(SearchError::InvalidConfig(
            "location required for vaar/hora/ghatika",
        ));
    }
    let from_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), from_utc);
    let to_jd = utc_to_jd_tdb_with_eop(engine, Some(eop), to_utc);
    if to_jd <= from_jd {
        return Err(SearchError::InvalidConfig(
            "to_utc must be after from_utc",
        ));
    }
    let cap = if max_events == 0 {
        MAX_PANCHANG_EVENTS
    } else {
        max_events.min(MAX_PANCHANG_EVENTS)
    };

    let include = |bit: u32| include_mask & bit != 0;
    let mut sweepers: Vec<Sweeper> = Vec::new();
    if include(PANCHANG_INCLUDE_TITHI) {
        sweepers.push(angular_sweeper(
            engine,
            eop,
            AngularKind::Tithi,
            from_jd,
            config,
        )?);
    }
    if include(PANCHANG_INCLUDE_KARANA) {
        sweepers.push(angular_sweeper(
            engine,
            eop,
            AngularKind::Karana,
            from_jd,
            config,
        )?);
    }
    if include(PANCHANG_INCLUDE_YOGA) {
        sweepers.push(angular_sweeper(
            engine,
            eop,
            AngularKind::Yoga,
            from_jd,
            config,
        )?);
    }
    if include(PANCHANG_INCLUDE_NAKSHATRA) {
        sweepers.push(angular_sweeper(
            engine,
            eop,
            AngularKind::Nakshatra,
            from_jd,
            config,
        )?);
    }
    if include(PANCHANG_INCLUDE_MASA) {
        let current = masa_for_date_with_eop(engine, Some(eop), from_utc, config)?;
        sweepers.push(Sweeper::Masa {
            start_jd: utc_to_jd_tdb_with_eop(engine, Some(eop), &current.start),
            end_jd: utc_to_jd_tdb_with_eop(engine, Some(eop), &current.end),
            current,
        });
    }
    if include(PANCHANG_INCLUDE_AYANA) {
        let current = ayana_for_date_with_eop(engine, Some(eop), from_utc, config)?;
        sweepers.push(Sweeper::Ayana {
            start_jd: utc_to_jd_tdb_with_eop(engine, Some(eop), &current.start),
            end_jd: utc_to_jd_tdb_with_eop(engine, Some(eop), &current.end),
            current,
        });
    }
    if include(PANCHANG_INCLUDE_VARSHA) {
        let current = varsha_for_date_with_eop(engine, Some(eop), from_utc, config)?;
        sweepers.push(Sweeper::Varsha {
            start_jd: utc_to_jd_tdb_with_eop(engine, Some(eop), &current.start),
            end_jd: utc_to_jd_tdb_with_eop(engine, Some(eop), &current.end),
            current,
        });
    }
    if include_mask & PANCHANG_INCLUDE_LOCATION_DEPENDENT != 0 {
        let location = location.expect("validated: location present");
        let day = VedicDayCursor::at(engine, eop, from_utc, location, riseset_config)?;
        if include(PANCHANG_INCLUDE_VAAR) {
            sweepers.push(Sweeper::Vaar {
                current: vaar_from_sunrises(day.start_jd, day.end_jd, engine.lsk()),
                day,
            });
        }
        if include(PANCHANG_INCLUDE_HORA) {
            let current = hora_from_sunrises(from_jd, day.start_jd, day.end_jd, engine.lsk());
            let index = current.hora_index as u16;
            let (start_jd, end_jd) = day.division(index, HORA_COUNT as u16);
            sweepers.push(Sweeper::Hora {
                current,
                day,
                index,
                start_jd,
                end_jd,
            });
        }
        if include(PANCHANG_INCLUDE_GHATIKA) {
            let current = ghatika_from_sunrises(from_jd, day.start_jd, day.end_jd, engine.lsk());
            let index = (current.value - 1) as u16;
            let (start_jd, end_jd) = day.division(index, 60);
            sweepers.push(Sweeper::Ghatika {
                current,
                day,
                index,
                start_jd,
                end_jd,
            });
        }
    }

    let mut result = PanchangEventsResult::default();
    let mut active: Vec<bool> = vec![true; sweepers.len()];
    let mut total: u32 = 0;

    loop {
        // Emit in global boundary-time order so all kinds progress together
        // and truncation leaves at most one unemitted segment per kind.
        let next = sweepers
            .iter()
            .enumerate()
            .filter(|(i, _)| active[*i])
            .min_by(|a, b| a.1.end_jd().total_cmp(&b.1.end_jd()))
            .map(|(i, _)| i);
        let Some(i) = next else { break };

        if total >= cap {
            result.truncated = true;
            let min_start = sweepers
                .iter()
                .enumerate()
                .filter(|(j, _)| active[*j])
                .map(|(_, s)| s.start_jd())
                .fold(f64::INFINITY, f64::min);
            result.next_from_utc = Some(UtcTime::from_jd_tdb(min_start, engine.lsk()));
            break;
        }

        sweepers[i].emit(&mut result);
        total += 1;
        if sweepers[i].end_jd() >= to_jd {
            active[i] = false;
        } else {
            sweepers[i].advance(engine, eop, config, location, riseset_config)?;
        }
    }

    Ok(result)
}
