//! `gochar_events`: grouped return-chart and transit-conjunction search.

use dhruv_core::{Body, Engine};
use dhruv_time::UtcTime;
use dhruv_vedic_base::{BhavaConfig, GeoLocation, RiseSetConfig, jd_tdb_to_centuries};

use crate::conjunction::body_ecliptic_lon_lat;
use crate::error::SearchError;
use crate::gochar_events_types::{
    EventWindow, GocharEventsConfig, GocharEventsOperation, GocharEventsResult, GocharReference,
    TajakaReturnBasis, TajakaReturnEvent, TithiPraveshaEvent, TransitAspectKind,
    TransitAspectOwner, TransitToNatalAspectEvent,
};
use crate::jyotish::full_kundali_for_date;
use crate::panchang::{elongation_at, masa_for_date_with_eop};
use crate::sankranti_types::SankrantiConfig;
use crate::search_util::{normalize_to_pm180, utc_to_jd_tdb_with_eop};

const FORWARD_EPSILON_DAYS: f64 = 1e-6;
const BACKWARD_EPSILON_DAYS: f64 = 1e-6;
const MONTHLY_SOLAR_MAX_SCAN_DAYS: f64 = 45.0;
const YEARLY_SOLAR_MAX_SCAN_DAYS: f64 = 410.0;
const MONTHLY_LUNAR_MAX_SCAN_DAYS: f64 = 40.0;
const SAME_MASA_MAX_ATTEMPTS: usize = 18;

/// Compute grouped Tajaka, Tithi Pravesha, and gochara conjunction events.
pub fn gochar_events(
    engine: &Engine,
    op: &GocharEventsOperation<'_>,
) -> Result<GocharEventsResult, SearchError> {
    validate_config(&op.config)?;

    let birth_jd = utc_to_jd_tdb_with_eop(engine, Some(op.eop), &op.birth_utc);
    let at_jd = utc_to_jd_tdb_with_eop(engine, Some(op.eop), &op.at_utc);

    let natal_tropical_solar_longitude_deg = tropical_solar_longitude(engine, birth_jd)?;
    let natal_sidereal_solar_longitude_deg =
        sidereal_solar_longitude(engine, birth_jd, &op.sankranti_config)?;
    let natal_elongation_deg = elongation_at(engine, birth_jd)?;
    let natal_masa =
        masa_for_date_with_eop(engine, Some(op.eop), &op.birth_utc, &op.sankranti_config)?;

    let reference = GocharReference {
        natal_tropical_solar_longitude_deg,
        natal_sidereal_solar_longitude_deg,
        natal_elongation_deg,
        natal_masa,
    };

    let yearly_tajaka = collect_tajaka_window(
        engine,
        at_jd,
        &op.location,
        &op.bhava_config,
        &op.riseset_config,
        &op.sankranti_config,
        &op.kundali_config,
        &op.config,
        natal_solar_target(&reference, op.config.tajaka_return_basis),
        360.0,
        YEARLY_SOLAR_MAX_SCAN_DAYS,
        op.eop,
    )?;
    let monthly_tajaka = collect_tajaka_window(
        engine,
        at_jd,
        &op.location,
        &op.bhava_config,
        &op.riseset_config,
        &op.sankranti_config,
        &op.kundali_config,
        &op.config,
        natal_solar_target(&reference, op.config.tajaka_return_basis),
        30.0,
        MONTHLY_SOLAR_MAX_SCAN_DAYS,
        op.eop,
    )?;

    let yearly_tithi_pravesha = collect_tithi_window(
        engine,
        at_jd,
        &op.location,
        &op.bhava_config,
        &op.riseset_config,
        &op.sankranti_config,
        &op.kundali_config,
        &op.config,
        &reference,
        true,
        op.eop,
    )?;
    let monthly_tithi_pravesha = collect_tithi_window(
        engine,
        at_jd,
        &op.location,
        &op.bhava_config,
        &op.riseset_config,
        &op.sankranti_config,
        &op.kundali_config,
        &op.config,
        &reference,
        false,
        op.eop,
    )?;

    let transit_events = collect_transit_events(engine, at_jd, op)?;

    Ok(GocharEventsResult {
        birth_utc: op.birth_utc,
        at_utc: op.at_utc,
        reference,
        yearly_tajaka,
        yearly_tithi_pravesha,
        monthly_tajaka,
        monthly_tithi_pravesha,
        transit_events,
    })
}

fn validate_config(config: &GocharEventsConfig) -> Result<(), SearchError> {
    if config.yearly_count == 0 {
        return Err(SearchError::InvalidConfig("yearly_count must be > 0"));
    }
    if config.monthly_count == 0 {
        return Err(SearchError::InvalidConfig("monthly_count must be > 0"));
    }
    if !config.transit_window_days.is_finite() || config.transit_window_days <= 0.0 {
        return Err(SearchError::InvalidConfig(
            "transit_window_days must be positive",
        ));
    }
    if !config.solar_step_size_days.is_finite() || config.solar_step_size_days <= 0.0 {
        return Err(SearchError::InvalidConfig(
            "solar_step_size_days must be positive",
        ));
    }
    if !config.lunar_step_size_days.is_finite() || config.lunar_step_size_days <= 0.0 {
        return Err(SearchError::InvalidConfig(
            "lunar_step_size_days must be positive",
        ));
    }
    if !config.solar_convergence_days.is_finite() || config.solar_convergence_days <= 0.0 {
        return Err(SearchError::InvalidConfig(
            "solar_convergence_days must be positive",
        ));
    }
    if !config.lunar_convergence_days.is_finite() || config.lunar_convergence_days <= 0.0 {
        return Err(SearchError::InvalidConfig(
            "lunar_convergence_days must be positive",
        ));
    }
    if config.max_iterations == 0 {
        return Err(SearchError::InvalidConfig("max_iterations must be > 0"));
    }
    Ok(())
}

fn natal_solar_target(reference: &GocharReference, basis: TajakaReturnBasis) -> f64 {
    match basis {
        TajakaReturnBasis::TropicalSolar => reference.natal_tropical_solar_longitude_deg,
        TajakaReturnBasis::SiderealSolar => reference.natal_sidereal_solar_longitude_deg,
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_tajaka_window(
    engine: &Engine,
    at_jd: f64,
    location: &GeoLocation,
    bhava_config: &BhavaConfig,
    riseset_config: &RiseSetConfig,
    sankranti_config: &SankrantiConfig,
    kundali_config: &crate::jyotish_types::FullKundaliConfig,
    config: &GocharEventsConfig,
    natal_target_deg: f64,
    period_deg: f64,
    max_scan_days: f64,
    eop: &dhruv_time::EopKernel,
) -> Result<EventWindow<TajakaReturnEvent>, SearchError> {
    let count = if period_deg == 360.0 {
        config.yearly_count
    } else {
        config.monthly_count
    };
    let mut before = Vec::with_capacity(count);
    let mut after = Vec::with_capacity(count);

    let mut cursor = at_jd - BACKWARD_EPSILON_DAYS;
    for _ in 0..count {
        let Some(event_jd) = find_tajaka_return(
            engine,
            cursor,
            natal_target_deg,
            period_deg,
            config,
            max_scan_days,
            SearchDirection::Backward,
            sankranti_config,
        )?
        else {
            break;
        };
        before.push(build_tajaka_event(
            engine,
            event_jd,
            location,
            bhava_config,
            riseset_config,
            sankranti_config,
            kundali_config,
            config,
            natal_target_deg,
            eop,
        )?);
        cursor = event_jd - BACKWARD_EPSILON_DAYS;
    }
    before.reverse();

    cursor = at_jd + FORWARD_EPSILON_DAYS;
    for _ in 0..count {
        let Some(event_jd) = find_tajaka_return(
            engine,
            cursor,
            natal_target_deg,
            period_deg,
            config,
            max_scan_days,
            SearchDirection::Forward,
            sankranti_config,
        )?
        else {
            break;
        };
        after.push(build_tajaka_event(
            engine,
            event_jd,
            location,
            bhava_config,
            riseset_config,
            sankranti_config,
            kundali_config,
            config,
            natal_target_deg,
            eop,
        )?);
        cursor = event_jd + FORWARD_EPSILON_DAYS;
    }

    Ok(EventWindow { before, after })
}

#[allow(clippy::too_many_arguments)]
fn collect_tithi_window(
    engine: &Engine,
    at_jd: f64,
    location: &GeoLocation,
    bhava_config: &BhavaConfig,
    riseset_config: &RiseSetConfig,
    sankranti_config: &SankrantiConfig,
    kundali_config: &crate::jyotish_types::FullKundaliConfig,
    config: &GocharEventsConfig,
    reference: &GocharReference,
    require_same_masa: bool,
    eop: &dhruv_time::EopKernel,
) -> Result<EventWindow<TithiPraveshaEvent>, SearchError> {
    let count = if require_same_masa {
        config.yearly_count
    } else {
        config.monthly_count
    };
    let mut before = Vec::with_capacity(count);
    let mut after = Vec::with_capacity(count);

    let mut cursor = at_jd - BACKWARD_EPSILON_DAYS;
    for _ in 0..count {
        let Some((event_jd, masa)) = find_tithi_pravesha_return(
            engine,
            cursor,
            reference,
            config,
            sankranti_config,
            SearchDirection::Backward,
            require_same_masa,
            eop,
        )?
        else {
            break;
        };
        before.push(build_tithi_event(
            engine,
            event_jd,
            masa,
            location,
            bhava_config,
            riseset_config,
            sankranti_config,
            kundali_config,
            config,
            reference.natal_elongation_deg,
            eop,
        )?);
        cursor = event_jd - BACKWARD_EPSILON_DAYS;
    }
    before.reverse();

    cursor = at_jd + FORWARD_EPSILON_DAYS;
    for _ in 0..count {
        let Some((event_jd, masa)) = find_tithi_pravesha_return(
            engine,
            cursor,
            reference,
            config,
            sankranti_config,
            SearchDirection::Forward,
            require_same_masa,
            eop,
        )?
        else {
            break;
        };
        after.push(build_tithi_event(
            engine,
            event_jd,
            masa,
            location,
            bhava_config,
            riseset_config,
            sankranti_config,
            kundali_config,
            config,
            reference.natal_elongation_deg,
            eop,
        )?);
        cursor = event_jd + FORWARD_EPSILON_DAYS;
    }

    Ok(EventWindow { before, after })
}

fn collect_transit_events(
    engine: &Engine,
    at_jd: f64,
    op: &GocharEventsOperation<'_>,
) -> Result<Vec<TransitToNatalAspectEvent>, SearchError> {
    let start_jd = at_jd - op.config.transit_window_days;
    let end_jd = at_jd + op.config.transit_window_days;
    let mut events = Vec::new();

    for &body in &op.transit_bodies {
        let step_size_days = transit_step_size_days(body);
        for target in &op.natal_targets {
            for spec in aspect_specs_for_pair(body, target) {
                let mut cursor = start_jd;
                let fixed_longitude = match spec.owner {
                    TransitAspectOwner::GocharBody => {
                        (target.longitude_deg - spec.angle_deg).rem_euclid(360.0)
                    }
                    TransitAspectOwner::NatalTarget => {
                        (target.longitude_deg + spec.angle_deg).rem_euclid(360.0)
                    }
                };
                while let Some(event_jd) = find_fixed_longitude_event(
                    engine,
                    cursor,
                    end_jd,
                    fixed_longitude,
                    step_size_days,
                    op.config.max_iterations,
                    op.config.solar_convergence_days,
                    &|jd| sidereal_body_longitude(engine, body, jd, &op.sankranti_config),
                    360.0,
                    SearchDirection::Forward,
                )? {
                    if event_jd > end_jd + FORWARD_EPSILON_DAYS {
                        break;
                    }
                    let transit_longitude_deg =
                        sidereal_body_longitude(engine, body, event_jd, &op.sankranti_config)?;
                    events.push(TransitToNatalAspectEvent {
                        transit_body: body,
                        target: target.clone(),
                        target_name: target.display_name().to_string(),
                        aspect_kind: spec.kind,
                        aspect_owner: spec.owner,
                        aspect_angle_deg: spec.angle_deg,
                        utc: UtcTime::from_jd_tdb(event_jd, engine.lsk()),
                        jd_tdb: event_jd,
                        transit_longitude_deg,
                        target_longitude_deg: target.longitude_deg,
                        actual_separation_deg: normalize_to_pm180(
                            transit_longitude_deg - fixed_longitude,
                        )
                        .abs(),
                    });
                    cursor = event_jd + FORWARD_EPSILON_DAYS;
                }
            }
        }
    }

    events.sort_by(|a, b| {
        a.jd_tdb
            .total_cmp(&b.jd_tdb)
            .then_with(|| a.aspect_angle_deg.total_cmp(&b.aspect_angle_deg))
    });
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
fn build_tajaka_event(
    engine: &Engine,
    event_jd: f64,
    location: &GeoLocation,
    bhava_config: &BhavaConfig,
    riseset_config: &RiseSetConfig,
    sankranti_config: &SankrantiConfig,
    kundali_config: &crate::jyotish_types::FullKundaliConfig,
    config: &GocharEventsConfig,
    natal_target_deg: f64,
    eop: &dhruv_time::EopKernel,
) -> Result<TajakaReturnEvent, SearchError> {
    let utc = UtcTime::from_jd_tdb(event_jd, engine.lsk());
    let event_solar_longitude_deg = match config.tajaka_return_basis {
        TajakaReturnBasis::TropicalSolar => tropical_solar_longitude(engine, event_jd)?,
        TajakaReturnBasis::SiderealSolar => {
            sidereal_solar_longitude(engine, event_jd, sankranti_config)?
        }
    };
    let chart = if config.include_return_charts {
        Some(full_kundali_for_date(
            engine,
            eop,
            &utc,
            location,
            bhava_config,
            riseset_config,
            sankranti_config,
            kundali_config,
        )?)
    } else {
        None
    };

    Ok(TajakaReturnEvent {
        utc,
        jd_tdb: event_jd,
        basis: config.tajaka_return_basis,
        target_solar_longitude_deg: natal_target_deg,
        event_solar_longitude_deg,
        chart,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_tithi_event(
    engine: &Engine,
    event_jd: f64,
    masa: crate::panchang_types::MasaInfo,
    location: &GeoLocation,
    bhava_config: &BhavaConfig,
    riseset_config: &RiseSetConfig,
    sankranti_config: &SankrantiConfig,
    kundali_config: &crate::jyotish_types::FullKundaliConfig,
    config: &GocharEventsConfig,
    natal_elongation_deg: f64,
    eop: &dhruv_time::EopKernel,
) -> Result<TithiPraveshaEvent, SearchError> {
    let utc = UtcTime::from_jd_tdb(event_jd, engine.lsk());
    let event_elongation_deg = elongation_at(engine, event_jd)?;
    let chart = if config.include_return_charts {
        Some(full_kundali_for_date(
            engine,
            eop,
            &utc,
            location,
            bhava_config,
            riseset_config,
            sankranti_config,
            kundali_config,
        )?)
    } else {
        None
    };
    Ok(TithiPraveshaEvent {
        utc,
        jd_tdb: event_jd,
        target_elongation_deg: natal_elongation_deg,
        event_elongation_deg,
        masa,
        chart,
    })
}

fn find_tajaka_return(
    engine: &Engine,
    start_jd: f64,
    natal_target_deg: f64,
    period_deg: f64,
    config: &GocharEventsConfig,
    max_scan_days: f64,
    direction: SearchDirection,
    sankranti_config: &SankrantiConfig,
) -> Result<Option<f64>, SearchError> {
    let step = match direction {
        SearchDirection::Forward => config.solar_step_size_days,
        SearchDirection::Backward => -config.solar_step_size_days,
    };
    let max_steps = (max_scan_days / config.solar_step_size_days).ceil() as usize;
    find_periodic_return(
        start_jd,
        step,
        max_steps,
        config.max_iterations,
        config.solar_convergence_days,
        period_deg,
        &|jd| {
            let lon = match config.tajaka_return_basis {
                TajakaReturnBasis::TropicalSolar => tropical_solar_longitude(engine, jd)?,
                TajakaReturnBasis::SiderealSolar => {
                    sidereal_solar_longitude(engine, jd, sankranti_config)?
                }
            };
            Ok(normalize_to_half_period(lon - natal_target_deg, period_deg))
        },
    )
}

fn find_tithi_pravesha_return(
    engine: &Engine,
    start_jd: f64,
    reference: &GocharReference,
    config: &GocharEventsConfig,
    sankranti_config: &SankrantiConfig,
    direction: SearchDirection,
    require_same_masa: bool,
    eop: &dhruv_time::EopKernel,
) -> Result<Option<(f64, crate::panchang_types::MasaInfo)>, SearchError> {
    let mut cursor = start_jd;
    for _ in 0..SAME_MASA_MAX_ATTEMPTS {
        let step = match direction {
            SearchDirection::Forward => config.lunar_step_size_days,
            SearchDirection::Backward => -config.lunar_step_size_days,
        };
        let max_steps = (MONTHLY_LUNAR_MAX_SCAN_DAYS / config.lunar_step_size_days).ceil() as usize;
        let Some(event_jd) = find_periodic_return(
            cursor,
            step,
            max_steps,
            config.max_iterations,
            config.lunar_convergence_days,
            360.0,
            &|jd| {
                let elong = elongation_at(engine, jd)?;
                Ok(normalize_to_half_period(
                    elong - reference.natal_elongation_deg,
                    360.0,
                ))
            },
        )?
        else {
            return Ok(None);
        };

        let event_utc = UtcTime::from_jd_tdb(event_jd, engine.lsk());
        let masa = masa_for_date_with_eop(engine, Some(eop), &event_utc, sankranti_config)?;
        let same_masa =
            masa.masa == reference.natal_masa.masa && masa.adhika == reference.natal_masa.adhika;
        if !require_same_masa || same_masa {
            return Ok(Some((event_jd, masa)));
        }
        cursor = match direction {
            SearchDirection::Forward => event_jd + FORWARD_EPSILON_DAYS,
            SearchDirection::Backward => event_jd - BACKWARD_EPSILON_DAYS,
        };
    }
    Ok(None)
}

#[allow(clippy::too_many_arguments)]
fn find_fixed_longitude_event(
    _engine: &Engine,
    start_jd: f64,
    end_jd: f64,
    target_deg: f64,
    step_size_days: f64,
    max_iterations: u32,
    convergence_days: f64,
    longitude_fn: &dyn Fn(f64) -> Result<f64, SearchError>,
    period_deg: f64,
    direction: SearchDirection,
) -> Result<Option<f64>, SearchError> {
    let step = match direction {
        SearchDirection::Forward => step_size_days,
        SearchDirection::Backward => -step_size_days,
    };
    let total_days = (end_jd - start_jd).abs();
    let max_steps = (total_days / step_size_days).ceil() as usize + 1;
    find_periodic_return(
        start_jd,
        step,
        max_steps,
        max_iterations,
        convergence_days,
        period_deg,
        &|jd| {
            if matches!(direction, SearchDirection::Forward) && jd > end_jd + FORWARD_EPSILON_DAYS {
                return Ok(1.0);
            }
            if matches!(direction, SearchDirection::Backward) && jd < end_jd - BACKWARD_EPSILON_DAYS
            {
                return Ok(1.0);
            }
            let longitude = longitude_fn(jd)?;
            Ok(normalize_to_half_period(longitude - target_deg, period_deg))
        },
    )
}

fn find_periodic_return(
    jd_start: f64,
    step: f64,
    max_steps: usize,
    max_iterations: u32,
    convergence_days: f64,
    period_deg: f64,
    f: &dyn Fn(f64) -> Result<f64, SearchError>,
) -> Result<Option<f64>, SearchError> {
    let mut f_prev = f(jd_start)?;
    let mut t_prev = jd_start;
    for _ in 0..max_steps {
        let t_curr = t_prev + step;
        let f_curr = f(t_curr)?;
        if is_periodic_crossing(f_prev, f_curr, period_deg) {
            let (mut t_a, mut f_a, mut t_b) = if t_prev < t_curr {
                (t_prev, f_prev, t_curr)
            } else {
                (t_curr, f_curr, t_prev)
            };
            for _ in 0..max_iterations {
                let t_mid = 0.5 * (t_a + t_b);
                let f_mid = f(t_mid)?;
                if f_a * f_mid <= 0.0 {
                    t_b = t_mid;
                } else {
                    t_a = t_mid;
                    f_a = f_mid;
                }
                if (t_b - t_a).abs() < convergence_days {
                    break;
                }
            }
            return Ok(Some(0.5 * (t_a + t_b)));
        }
        t_prev = t_curr;
        f_prev = f_curr;
    }
    Ok(None)
}

fn is_periodic_crossing(f_a: f64, f_b: f64, period_deg: f64) -> bool {
    f_a * f_b < 0.0 && (f_a - f_b).abs() < period_deg * 0.75
}

fn normalize_to_half_period(deg: f64, period_deg: f64) -> f64 {
    let mut d = deg % period_deg;
    let half = period_deg / 2.0;
    if d > half {
        d -= period_deg;
    } else if d <= -half {
        d += period_deg;
    }
    d
}

fn tropical_solar_longitude(engine: &Engine, jd_tdb: f64) -> Result<f64, SearchError> {
    let (longitude_deg, _latitude_deg) = body_ecliptic_lon_lat(engine, Body::Sun, jd_tdb)?;
    Ok(longitude_deg)
}

fn sidereal_solar_longitude(
    engine: &Engine,
    jd_tdb: f64,
    sankranti_config: &SankrantiConfig,
) -> Result<f64, SearchError> {
    sidereal_body_longitude(engine, Body::Sun, jd_tdb, sankranti_config)
}

fn sidereal_body_longitude(
    engine: &Engine,
    body: Body,
    jd_tdb: f64,
    sankranti_config: &SankrantiConfig,
) -> Result<f64, SearchError> {
    let (lon, _lat) = crate::conjunction::body_lon_lat_on_plane(
        engine,
        body,
        jd_tdb,
        sankranti_config.precession_model,
        sankranti_config.reference_plane,
    )?;
    let t = jd_tdb_to_centuries(jd_tdb);
    let ayanamsha_deg = sankranti_config.ayanamsha_deg_at_centuries(t);
    Ok((lon - ayanamsha_deg).rem_euclid(360.0))
}

fn transit_step_size_days(body: Body) -> f64 {
    match body {
        Body::Moon => 0.25,
        Body::Mercury | Body::Venus => 0.5,
        Body::Sun | Body::Mars => 1.0,
        Body::Jupiter | Body::Saturn => 2.0,
        Body::Uranus | Body::Neptune | Body::Pluto => 5.0,
        _ => 1.0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy)]
struct AspectSearchSpec {
    kind: TransitAspectKind,
    owner: TransitAspectOwner,
    angle_deg: f64,
}

fn aspect_specs_for_pair(
    transit_body: Body,
    target: &crate::gochar_events_types::NatalTargetLongitude,
) -> Vec<AspectSearchSpec> {
    let mut specs = vec![
        AspectSearchSpec {
            kind: TransitAspectKind::Conjunction,
            owner: TransitAspectOwner::GocharBody,
            angle_deg: 0.0,
        },
        AspectSearchSpec {
            kind: TransitAspectKind::Opposition,
            owner: TransitAspectOwner::GocharBody,
            angle_deg: 180.0,
        },
    ];

    for angle_deg in special_angles_for_body(transit_body) {
        specs.push(AspectSearchSpec {
            kind: TransitAspectKind::Special,
            owner: TransitAspectOwner::GocharBody,
            angle_deg: *angle_deg,
        });
    }

    if let Some(natal_body) = natal_graha_body(target) {
        for angle_deg in special_angles_for_body(natal_body) {
            specs.push(AspectSearchSpec {
                kind: TransitAspectKind::Special,
                owner: TransitAspectOwner::NatalTarget,
                angle_deg: *angle_deg,
            });
        }
    }

    specs
}

fn natal_graha_body(target: &crate::gochar_events_types::NatalTargetLongitude) -> Option<Body> {
    use crate::gochar_events_types::NatalTargetKind;
    match target.kind {
        NatalTargetKind::Graha => match target.index {
            0 => Some(Body::Sun),
            1 => Some(Body::Moon),
            2 => Some(Body::Mars),
            3 => Some(Body::Mercury),
            4 => Some(Body::Jupiter),
            5 => Some(Body::Venus),
            6 => Some(Body::Saturn),
            _ => None,
        },
        _ => None,
    }
}

fn special_angles_for_body(body: Body) -> &'static [f64] {
    match body {
        Body::Jupiter => &[120.0, 240.0],
        Body::Saturn => &[90.0, 270.0],
        Body::Mars => &[90.0, 210.0],
        _ => &[],
    }
}
