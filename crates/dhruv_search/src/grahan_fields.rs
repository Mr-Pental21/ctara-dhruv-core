//! Surya grahan field products: the per-event local-circumstance grid,
//! visibility/duration/magnitude isolines, and the swept central corridor.
//!
//! All topocentric evaluation follows the same conventions as
//! `grahan::local_disk_geometry` (true equatorial-of-date vectors, oblate
//! observer, standard-refraction Sun-up threshold of -0.833 degrees),
//! restructured so the ephemeris is queried once per time sample and every
//! per-location evaluation is pure math.
//!
//! Isolines are extracted with marching squares over a node grid and every
//! ring vertex is refined by bisection against the exact continuous field.
//! The isoline grid is a global latitude/longitude grid with the poles as
//! degenerate rows and longitude wraparound. The corridor grid is
//! track-aligned (along-track x cross-track), because the umbral/antumbral
//! band is kilometers thin near hybrid transitions and contacts — far below
//! any affordable uniform geographic resolution.
//!
//! Sources: standard spherical astronomy (Meeus Ch. 54 shadow geometry, IAU
//! 2015 nominal radii) and the classical marching-squares contouring
//! algorithm (public literature). See
//! docs/clean_room_solar_eclipse_visibility.md.

use dhruv_core::Engine;
use dhruv_time::{EopKernel, UtcTime};

use crate::error::SearchError;
use crate::grahan::{
    EARTH_RADIUS_KM, MOON_RADIUS_KM, SUN_RADIUS_KM, disk_magnitude, disk_obscuration, dot,
    gast_rad_for, geodetic_to_ecef, norm, rotate_z, scale, sub, sun_moon_true_vectors,
};
use crate::grahan_types::{
    EclipseGeoPoint, GeoLocation, GrahanConfig, PoleSide, SuryaCentralCorridor,
    SuryaCorridorSegment, SuryaDurationIsoline, SuryaGrahanType, SuryaIsolineRing, SuryaIsolines,
    SuryaLocalGridSample, SuryaMagnitudeIsoline,
};

/// Sun-up altitude threshold in degrees (standard refraction + semidiameter),
/// identical to the convention used by `local` circumstances.
const SUN_UP_ALTITUDE_DEG: f64 = -0.833;

/// Scale that maps the Sun-up altitude margin (radians) into magnitude units
/// for the visible-magnitude field, keeping the field continuous across the
/// terminator while leaving interior values exact.
const MAGNITUDE_ALTITUDE_SCALE: f64 = 50.0;

/// Along-track node spacing for the corridor grid, km of ground distance.
const CORRIDOR_ALONG_STEP_KM: f64 = 25.0;

/// Cross-track node spacing for the corridor grid, degrees.
const CORRIDOR_CROSS_STEP_DEG: f64 = 0.02;

/// Per-node time-search half window around the shadow's passage, days.
const CORRIDOR_TIME_HALF_WINDOW_DAYS: f64 = 15.0 / 1440.0;

// ---------------------------------------------------------------------------
// Dense time table: ephemeris queried once per sample, then pure math
// ---------------------------------------------------------------------------

pub(crate) struct FieldTable {
    start_jd: f64,
    step_days: f64,
    sun: Vec<[f64; 3]>,
    moon: Vec<[f64; 3]>,
    /// Unwrapped (monotonically increasing) apparent sidereal time, radians.
    gast: Vec<f64>,
}

impl FieldTable {
    pub(crate) fn build(
        engine: &Engine,
        eop: Option<&EopKernel>,
        start_jd: f64,
        end_jd: f64,
        step_days: f64,
    ) -> Result<Self, SearchError> {
        let count = (((end_jd - start_jd) / step_days).ceil() as usize).max(1) + 1;
        let mut sun = Vec::with_capacity(count);
        let mut moon = Vec::with_capacity(count);
        let mut gast = Vec::with_capacity(count);
        let mut previous_gast = f64::NEG_INFINITY;
        let mut turns = 0.0;
        for index in 0..count {
            let jd = start_jd + index as f64 * step_days;
            let (s, m) = sun_moon_true_vectors(engine, jd)?;
            let mut g = gast_rad_for(engine, eop, jd) + turns;
            while g < previous_gast {
                g += std::f64::consts::TAU;
                turns += std::f64::consts::TAU;
            }
            previous_gast = g;
            sun.push(s);
            moon.push(m);
            gast.push(g);
        }
        Ok(Self {
            start_jd,
            step_days,
            sun,
            moon,
            gast,
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.gast.len()
    }

    pub(crate) fn jd_at(&self, index: usize) -> f64 {
        self.start_jd + index as f64 * self.step_days
    }

    pub(crate) fn start_jd(&self) -> f64 {
        self.start_jd
    }

    pub(crate) fn end_jd(&self) -> f64 {
        self.jd_at(self.len() - 1)
    }

    fn index_range(&self, jd_from: f64, jd_to: f64) -> (usize, usize) {
        let first = (((jd_from - self.start_jd) / self.step_days).floor() as isize)
            .clamp(0, self.len() as isize - 1) as usize;
        let last = (((jd_to - self.start_jd) / self.step_days).ceil() as isize)
            .clamp(0, self.len() as isize - 1) as usize;
        (first, last)
    }

    fn eval_index(&self, index: usize, observer: &ObserverPoint) -> PointEval {
        eval_with(self.sun[index], self.moon[index], self.gast[index], observer)
    }

    /// Evaluate at an arbitrary epoch by linear interpolation of the Sun and
    /// Moon vectors and sidereal angle (sub-km accurate over a two-minute
    /// sample interval).
    fn eval_jd(&self, jd: f64, observer: &ObserverPoint) -> PointEval {
        let position =
            ((jd - self.start_jd) / self.step_days).clamp(0.0, (self.len() - 1) as f64 - 1.0e-9);
        let index = position.floor() as usize;
        let t = position - index as f64;
        let lerp3 = |a: [f64; 3], b: [f64; 3]| {
            [
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
            ]
        };
        let sun = lerp3(self.sun[index], self.sun[index + 1]);
        let moon = lerp3(self.moon[index], self.moon[index + 1]);
        let gast = self.gast[index] + (self.gast[index + 1] - self.gast[index]) * t;
        eval_with(sun, moon, gast, observer)
    }
}

/// Precomputed observer geometry for one ground location (ellipsoid surface).
pub(crate) struct ObserverPoint {
    ecef: [f64; 3],
    cos_lat: f64,
    sin_lat: f64,
    cos_lon: f64,
    sin_lon: f64,
}

impl ObserverPoint {
    pub(crate) fn new(latitude_deg: f64, longitude_deg: f64) -> Self {
        let location = GeoLocation::new(latitude_deg, longitude_deg, 0.0);
        let lat = latitude_deg.to_radians();
        let lon = longitude_deg.to_radians();
        Self {
            ecef: geodetic_to_ecef(&location),
            cos_lat: lat.cos(),
            sin_lat: lat.sin(),
            cos_lon: lon.cos(),
            sin_lon: lon.sin(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PointEval {
    pub separation_rad: f64,
    pub sun_radius_rad: f64,
    pub moon_radius_rad: f64,
    pub sun_altitude_deg: f64,
}

impl PointEval {
    /// Partial-phase margin: positive while any partial phase is in progress.
    fn penumbral_margin(&self) -> f64 {
        self.sun_radius_rad + self.moon_radius_rad - self.separation_rad
    }

    /// Sun-up margin in radians (positive when the Sun counts as risen).
    fn altitude_margin(&self) -> f64 {
        (self.sun_altitude_deg - SUN_UP_ALTITUDE_DEG).to_radians()
    }

    /// Continuous visibility margin: positive iff a partial phase is in
    /// progress while the Sun is up.
    fn visibility_margin(&self) -> f64 {
        self.penumbral_margin().min(self.altitude_margin())
    }

    /// Unclamped instantaneous magnitude (can be negative outside contact).
    fn magnitude_raw(&self) -> f64 {
        self.penumbral_margin() / (2.0 * self.sun_radius_rad)
    }

    /// Magnitude clipped continuously to zero across the terminator.
    fn visible_magnitude(&self) -> f64 {
        self.magnitude_raw()
            .min(MAGNITUDE_ALTITUDE_SCALE * self.altitude_margin())
    }

    /// Total (umbral) margin: positive while the total phase is in progress.
    fn total_margin(&self) -> f64 {
        (self.moon_radius_rad - self.sun_radius_rad) - self.separation_rad
    }

    /// Annular (antumbral) margin: positive during the annular phase.
    fn annular_margin(&self) -> f64 {
        (self.sun_radius_rad - self.moon_radius_rad) - self.separation_rad
    }
}

fn eval_with(sun: [f64; 3], moon: [f64; 3], gast: f64, observer: &ObserverPoint) -> PointEval {
    let observer_eq = rotate_z(observer.ecef, gast);
    let sun_topo = sub(sun, observer_eq);
    let moon_topo = sub(moon, observer_eq);
    let sun_distance = norm(sun_topo);
    let moon_distance = norm(moon_topo);
    let sun_u = scale(sun_topo, 1.0 / sun_distance);
    let moon_u = scale(moon_topo, 1.0 / moon_distance);
    let separation_rad = dot(sun_u, moon_u).clamp(-1.0, 1.0).acos();
    let sun_ecef = rotate_z(sun_u, -gast);
    let up = observer.cos_lat * observer.cos_lon * sun_ecef[0]
        + observer.cos_lat * observer.sin_lon * sun_ecef[1]
        + observer.sin_lat * sun_ecef[2];
    PointEval {
        separation_rad,
        sun_radius_rad: (SUN_RADIUS_KM / sun_distance).asin(),
        moon_radius_rad: (MOON_RADIUS_KM / moon_distance).asin(),
        sun_altitude_deg: up.clamp(-1.0, 1.0).asin().to_degrees(),
    }
}

// ---------------------------------------------------------------------------
// Scalar refinement helpers (pure)
// ---------------------------------------------------------------------------

/// Golden-section maximum of a continuous scalar over [left, right].
fn golden_max<F: Fn(f64) -> f64>(mut left: f64, mut right: f64, f: F) -> (f64, f64) {
    let phi = (5.0_f64.sqrt() - 1.0) * 0.5;
    let mut c = right - phi * (right - left);
    let mut d = left + phi * (right - left);
    let mut fc = f(c);
    let mut fd = f(d);
    for _ in 0..48 {
        if fc >= fd {
            right = d;
            d = c;
            fd = fc;
            c = right - phi * (right - left);
            fc = f(c);
        } else {
            left = c;
            c = d;
            fc = fd;
            d = left + phi * (right - left);
            fd = f(d);
        }
    }
    let mid = (left + right) * 0.5;
    (mid, f(mid))
}

/// Bisection root of a continuous scalar with opposite signs at the ends.
fn bisect_zero<F: Fn(f64) -> f64>(mut left: f64, mut right: f64, f: F) -> f64 {
    let mut f_left = f(left);
    if f_left == 0.0 {
        return left;
    }
    for _ in 0..40 {
        let mid = (left + right) * 0.5;
        let f_mid = f(mid);
        if f_left * f_mid <= 0.0 {
            right = mid;
        } else {
            left = mid;
            f_left = f_mid;
        }
    }
    (left + right) * 0.5
}

// ---------------------------------------------------------------------------
// Per-point summary over the event window
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub(crate) struct PointSummary {
    /// Refined maximum visibility margin; positive iff any Sun-up partial
    /// phase occurs (the visibility-boundary field).
    pub f_vis: f64,
    /// Refined maximum Sun-up-clipped magnitude (the magnitude field).
    pub f_mag: f64,
    /// Summed Sun-up partial-phase duration in days (the duration field).
    pub duration_days: f64,
    /// Outer edges of the visible intervals (valid when duration > 0).
    pub first_contact_jd: f64,
    pub last_contact_jd: f64,
    /// Unclipped local maximum (minimum separation), matching the
    /// per-location `local` convention.
    pub maximum_jd: f64,
    pub magnitude: f64,
    pub obscuration: f64,
}

pub(crate) fn summarize_point(table: &FieldTable, observer: &ObserverPoint) -> PointSummary {
    let count = table.len();
    let mut best_sep = f64::INFINITY;
    let mut best_sep_index = 0usize;
    let mut best_vis = f64::NEG_INFINITY;
    let mut best_vis_index = 0usize;
    let mut best_mag = f64::NEG_INFINITY;
    let mut best_mag_index = 0usize;
    let mut margins: Vec<f64> = Vec::with_capacity(count);
    for index in 0..count {
        let eval = table.eval_index(index, observer);
        if eval.separation_rad < best_sep {
            best_sep = eval.separation_rad;
            best_sep_index = index;
        }
        let vis = eval.visibility_margin();
        if vis > best_vis {
            best_vis = vis;
            best_vis_index = index;
        }
        let mag = eval.visible_magnitude();
        if mag > best_mag {
            best_mag = mag;
            best_mag_index = index;
        }
        margins.push(vis);
    }

    let bracket = |index: usize| -> (f64, f64) {
        (
            table.jd_at(index.saturating_sub(1)),
            table.jd_at((index + 1).min(count - 1)),
        )
    };

    // Unclipped local maximum: minimum separation, per-location convention.
    let (sep_left, sep_right) = bracket(best_sep_index);
    let (maximum_jd, _) = golden_max(sep_left, sep_right, |jd| {
        -table.eval_jd(jd, observer).separation_rad
    });
    let at_maximum = table.eval_jd(maximum_jd, observer);
    let magnitude = disk_magnitude(
        at_maximum.separation_rad,
        at_maximum.sun_radius_rad,
        at_maximum.moon_radius_rad,
    );
    let obscuration = disk_obscuration(
        at_maximum.separation_rad,
        at_maximum.sun_radius_rad,
        at_maximum.moon_radius_rad,
    );

    let (vis_left, vis_right) = bracket(best_vis_index);
    let (f_vis_jd, f_vis) = golden_max(vis_left, vis_right, |jd| {
        table.eval_jd(jd, observer).visibility_margin()
    });
    let (mag_left, mag_right) = bracket(best_mag_index);
    let (_, f_mag) = golden_max(mag_left, mag_right, |jd| {
        table.eval_jd(jd, observer).visible_magnitude()
    });

    // Visible intervals: sign transitions of the visibility margin, refined
    // by bisection on the continuous margin.
    let margin_at = |jd: f64| table.eval_jd(jd, observer).visibility_margin();
    let mut intervals: Vec<(f64, f64)> = Vec::new();
    let mut open_start: Option<f64> = if margins[0] > 0.0 {
        Some(table.jd_at(0))
    } else {
        None
    };
    for index in 1..count {
        let previous = margins[index - 1];
        let current = margins[index];
        if previous <= 0.0 && current > 0.0 {
            let jd = bisect_zero(table.jd_at(index - 1), table.jd_at(index), margin_at);
            open_start = Some(jd);
        } else if previous > 0.0
            && current <= 0.0
            && let Some(start) = open_start.take()
        {
            let jd = bisect_zero(table.jd_at(index), table.jd_at(index - 1), margin_at);
            intervals.push((start, jd));
        }
    }
    if let Some(start) = open_start.take() {
        intervals.push((start, table.jd_at(count - 1)));
    }
    // A refined-positive point whose samples were all non-positive: a brief
    // visible window between two samples. Recover it around the refined max.
    if intervals.is_empty() && f_vis > 0.0 {
        let left_seed = (f_vis_jd - table.step_days).max(table.start_jd());
        let right_seed = (f_vis_jd + table.step_days).min(table.end_jd());
        let start = if margin_at(left_seed) <= 0.0 {
            bisect_zero(left_seed, f_vis_jd, margin_at)
        } else {
            left_seed
        };
        let end = if margin_at(right_seed) <= 0.0 {
            bisect_zero(right_seed, f_vis_jd, margin_at)
        } else {
            right_seed
        };
        intervals.push((start, end));
    }

    let duration_days: f64 = intervals.iter().map(|(start, end)| end - start).sum();
    let (first_contact_jd, last_contact_jd) = match (intervals.first(), intervals.last()) {
        (Some((first, _)), Some((_, last))) => (*first, *last),
        _ => (maximum_jd, maximum_jd),
    };

    PointSummary {
        f_vis,
        f_mag,
        duration_days,
        first_contact_jd,
        last_contact_jd,
        maximum_jd,
        magnitude,
        obscuration,
    }
}

// ---------------------------------------------------------------------------
// Marching squares over a generic node grid
// ---------------------------------------------------------------------------

/// Contour extraction grid. Node values live on an `n_rows` x `n_cols`
/// index grid (row-major); `point` maps fractional grid coordinates to
/// geographic coordinates and `field` evaluates the exact continuous field
/// there (used for edge refinement and saddle disambiguation). When `wraps`
/// is true the columns are cyclic and the mapping must stay continuous
/// across the seam (column `n_cols` = column 0 one turn later).
pub(crate) struct ContourGrid<'a> {
    pub n_rows: usize,
    pub n_cols: usize,
    pub wraps: bool,
    pub values: &'a [f64],
    pub point: &'a dyn Fn(f64, f64) -> EclipseGeoPoint,
    pub field: &'a dyn Fn(f64, f64) -> f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgeKey {
    row: u32,
    col: u32,
    horizontal: bool,
}

#[derive(Debug, Clone, Copy)]
enum CellEdge {
    S,
    E,
    N,
    W,
}

impl<'a> ContourGrid<'a> {
    fn value(&self, row: usize, col: usize) -> f64 {
        self.values[row * self.n_cols + col % self.n_cols]
    }

    fn cell_edge_key(&self, row: usize, col: usize, edge: CellEdge) -> EdgeKey {
        match edge {
            CellEdge::S => EdgeKey {
                row: row as u32,
                col: (col % self.n_cols) as u32,
                horizontal: true,
            },
            CellEdge::N => EdgeKey {
                row: (row + 1) as u32,
                col: (col % self.n_cols) as u32,
                horizontal: true,
            },
            CellEdge::W => EdgeKey {
                row: row as u32,
                col: (col % self.n_cols) as u32,
                horizontal: false,
            },
            CellEdge::E => EdgeKey {
                row: row as u32,
                col: ((col + 1) % self.n_cols) as u32,
                horizontal: false,
            },
        }
    }

    /// Crossing point on a cell edge, refined by bisection against the
    /// exact field in fractional grid coordinates.
    fn crossing(
        &self,
        row: usize,
        col: usize,
        edge: CellEdge,
        level: f64,
    ) -> EclipseGeoPoint {
        let ((row_a, col_a), (row_b, col_b)) = match edge {
            CellEdge::S => ((row, col), (row, col + 1)),
            CellEdge::N => ((row + 1, col), (row + 1, col + 1)),
            CellEdge::W => ((row, col), (row + 1, col)),
            CellEdge::E => ((row, col + 1), (row + 1, col + 1)),
        };
        let value_a = self.value(row_a, col_a) - level;
        let value_b = self.value(row_b, col_b) - level;
        let coords = |t: f64| -> (f64, f64) {
            (
                row_a as f64 + (row_b as f64 - row_a as f64) * t,
                col_a as f64 + (col_b as f64 - col_a as f64) * t,
            )
        };
        if value_a * value_b > 0.0 {
            // Degenerate (touching) edge: midpoint.
            let (row_f, col_f) = coords(0.5);
            return (self.point)(row_f, col_f);
        }
        let mut left = 0.0_f64;
        let mut right = 1.0_f64;
        let mut f_left = value_a;
        for _ in 0..14 {
            let mid = (left + right) * 0.5;
            let (row_f, col_f) = coords(mid);
            let f_mid = (self.field)(row_f, col_f) - level;
            if f_left * f_mid <= 0.0 {
                right = mid;
            } else {
                left = mid;
                f_left = f_mid;
            }
        }
        let (row_f, col_f) = coords((left + right) * 0.5);
        (self.point)(row_f, col_f)
    }
}

/// Extract closed level-set rings (`value > level` is the inside) with the
/// inside on the left of the direction of travel. Chains that reach the
/// domain boundary without closing are dropped; callers size their grids so
/// the region lies strictly inside (poles capped, seam wrapped, padding
/// verified).
pub(crate) fn extract_rings(grid: &ContourGrid<'_>, level: f64) -> Vec<SuryaIsolineRing> {
    use std::collections::HashMap;

    let n_rows = grid.n_rows;
    if n_rows < 2 || grid.n_cols < 2 {
        return Vec::new();
    }
    let n_cell_cols = if grid.wraps {
        grid.n_cols
    } else {
        grid.n_cols - 1
    };

    let mut next: HashMap<EdgeKey, EdgeKey> = HashMap::new();
    let mut points: HashMap<EdgeKey, EclipseGeoPoint> = HashMap::new();
    let mut starts: Vec<EdgeKey> = Vec::new();

    for row in 0..n_rows - 1 {
        for col in 0..n_cell_cols {
            let inside_a = grid.value(row, col) > level;
            let inside_b = grid.value(row, col + 1) > level;
            let inside_c = grid.value(row + 1, col + 1) > level;
            let inside_d = grid.value(row + 1, col) > level;
            let mask = (inside_a as u8)
                | ((inside_b as u8) << 1)
                | ((inside_c as u8) << 2)
                | ((inside_d as u8) << 3);
            if mask == 0 || mask == 15 {
                continue;
            }
            // Segments as (entry edge, exit edge), inside on the left.
            let segments: &[(CellEdge, CellEdge)] = match mask {
                1 => &[(CellEdge::S, CellEdge::W)],
                2 => &[(CellEdge::E, CellEdge::S)],
                3 => &[(CellEdge::E, CellEdge::W)],
                4 => &[(CellEdge::N, CellEdge::E)],
                5 => {
                    if (grid.field)(row as f64 + 0.5, col as f64 + 0.5) > level {
                        &[(CellEdge::S, CellEdge::E), (CellEdge::N, CellEdge::W)]
                    } else {
                        &[(CellEdge::S, CellEdge::W), (CellEdge::N, CellEdge::E)]
                    }
                }
                6 => &[(CellEdge::N, CellEdge::S)],
                7 => &[(CellEdge::N, CellEdge::W)],
                8 => &[(CellEdge::W, CellEdge::N)],
                9 => &[(CellEdge::S, CellEdge::N)],
                10 => {
                    if (grid.field)(row as f64 + 0.5, col as f64 + 0.5) > level {
                        &[(CellEdge::W, CellEdge::S), (CellEdge::E, CellEdge::N)]
                    } else {
                        &[(CellEdge::E, CellEdge::S), (CellEdge::W, CellEdge::N)]
                    }
                }
                11 => &[(CellEdge::E, CellEdge::N)],
                12 => &[(CellEdge::W, CellEdge::E)],
                13 => &[(CellEdge::S, CellEdge::E)],
                14 => &[(CellEdge::W, CellEdge::S)],
                _ => &[],
            };
            for (entry, exit) in segments {
                let entry_key = grid.cell_edge_key(row, col, *entry);
                let exit_key = grid.cell_edge_key(row, col, *exit);
                points
                    .entry(entry_key)
                    .or_insert_with(|| grid.crossing(row, col, *entry, level));
                points
                    .entry(exit_key)
                    .or_insert_with(|| grid.crossing(row, col, *exit, level));
                next.insert(entry_key, exit_key);
                starts.push(entry_key);
            }
        }
    }

    let mut visited: std::collections::HashSet<EdgeKey> = std::collections::HashSet::new();
    let mut rings = Vec::new();
    for start in starts {
        if visited.contains(&start) {
            continue;
        }
        let mut chain: Vec<EdgeKey> = Vec::new();
        let mut edge = start;
        let mut closed = false;
        loop {
            if !visited.insert(edge) {
                break;
            }
            chain.push(edge);
            match next.get(&edge) {
                Some(next_edge) => {
                    if *next_edge == start {
                        closed = true;
                        break;
                    }
                    edge = *next_edge;
                }
                None => break,
            }
        }
        if !closed || chain.len() < 3 {
            continue;
        }
        let raw: Vec<EclipseGeoPoint> = chain.iter().map(|key| points[key]).collect();
        rings.push(finalize_ring(&raw));
    }
    rings
}

/// Decide pole containment from winding, rotate the ring to a deterministic
/// start, and close it.
fn finalize_ring(raw: &[EclipseGeoPoint]) -> SuryaIsolineRing {
    let mut winding = 0.0_f64;
    for pair in raw.windows(2) {
        winding += wrap_delta(pair[1].longitude_deg - pair[0].longitude_deg);
    }
    winding += wrap_delta(raw[0].longitude_deg - raw[raw.len() - 1].longitude_deg);
    let contains_pole = if winding > 180.0 {
        Some(PoleSide::North)
    } else if winding < -180.0 {
        Some(PoleSide::South)
    } else {
        None
    };

    let mut start_index = 0usize;
    for (index, point) in raw.iter().enumerate() {
        let best = &raw[start_index];
        if (point.latitude_deg, point.longitude_deg) < (best.latitude_deg, best.longitude_deg) {
            start_index = index;
        }
    }

    let mut boundary: Vec<EclipseGeoPoint> = (0..raw.len())
        .map(|offset| raw[(start_index + offset) % raw.len()])
        .collect();
    boundary.push(boundary[0]);
    SuryaIsolineRing {
        boundary,
        contains_pole,
    }
}

pub(crate) fn wrap_delta(delta_deg: f64) -> f64 {
    let mut d = delta_deg % 360.0;
    if d > 180.0 {
        d -= 360.0;
    } else if d < -180.0 {
        d += 360.0;
    }
    d
}

fn normalize_lon(lon: f64) -> f64 {
    let wrapped = (lon + 180.0).rem_euclid(360.0) - 180.0;
    if wrapped == -180.0 && lon > 0.0 { 180.0 } else { wrapped }
}

// ---------------------------------------------------------------------------
// 5a + 5b: global grid and isolines
// ---------------------------------------------------------------------------

pub(crate) struct FieldProducts {
    pub local_grid: Vec<SuryaLocalGridSample>,
    pub isolines: Option<SuryaIsolines>,
}

struct NodeFields {
    f_vis: f64,
    f_dur_days: f64,
    f_mag: f64,
}

/// Compute the local-circumstance grid and/or isolines for one Surya event.
///
/// `window` is the sampled event span (C1..C4 with fallbacks); `span_days`
/// is the C1–C4 duration used to normalize duration fractions.
pub(crate) fn grid_and_isolines(
    engine: &Engine,
    eop: Option<&EopKernel>,
    window_start_jd: f64,
    window_end_jd: f64,
    span_days: f64,
    config: &GrahanConfig,
) -> Result<FieldProducts, SearchError> {
    let step_deg = config.effective_local_grid_step_deg();
    let n_lat = ((180.0 / step_deg).round() as usize).max(2);
    let n_lon = ((360.0 / step_deg).round() as usize).max(4);
    let lat_step = 180.0 / n_lat as f64;
    let lon_step = 360.0 / n_lon as f64;

    let window_days = window_end_jd - window_start_jd;
    let cadence_days = (window_days / 360.0).clamp(30.0 / 86_400.0, 120.0 / 86_400.0);
    let table = FieldTable::build(engine, eop, window_start_jd, window_end_jd, cadence_days)?;

    // Node rows: south pole, cell-centered rows, north pole.
    let mut lats: Vec<f64> = Vec::with_capacity(n_lat + 2);
    lats.push(-90.0);
    for i in 0..n_lat {
        lats.push(-90.0 + (i as f64 + 0.5) * lat_step);
    }
    lats.push(90.0);
    let lon0 = -180.0 + 0.5 * lon_step;

    let n_rows = lats.len();
    let mut node_fields: Vec<NodeFields> = Vec::with_capacity(n_rows * n_lon);
    let mut raw_samples: Vec<PointSummaryAt> = Vec::new();

    // Parallel over rows: workers are pure math against the shared table.
    let worker_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(n_rows.max(1));
    let rows_per_worker = n_rows.div_ceil(worker_count);
    let row_results: Vec<Vec<(Vec<NodeFields>, Vec<PointSummaryAt>)>> =
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for worker in 0..worker_count {
                let row_start = worker * rows_per_worker;
                let row_end = ((worker + 1) * rows_per_worker).min(n_rows);
                let table_ref = &table;
                let lats_ref = &lats;
                handles.push(scope.spawn(move || {
                    let mut rows = Vec::new();
                    for row in row_start..row_end {
                        let lat = lats_ref[row];
                        let pole_row = row == 0 || row + 1 == n_rows;
                        let mut fields = Vec::with_capacity(n_lon);
                        let mut samples = Vec::new();
                        if pole_row {
                            // Longitude-independent geometry at the exact pole.
                            let summary = summarize_point(table_ref, &ObserverPoint::new(lat, 0.0));
                            for _ in 0..n_lon {
                                fields.push(NodeFields {
                                    f_vis: summary.f_vis,
                                    f_dur_days: summary.duration_days,
                                    f_mag: summary.f_mag,
                                });
                            }
                        } else {
                            for col in 0..n_lon {
                                let lon = normalize_lon(lon0 + col as f64 * lon_step);
                                let summary =
                                    summarize_point(table_ref, &ObserverPoint::new(lat, lon));
                                if summary.duration_days > 0.0 {
                                    samples.push(PointSummaryAt {
                                        latitude_deg: lat,
                                        longitude_deg: lon,
                                        summary,
                                    });
                                }
                                fields.push(NodeFields {
                                    f_vis: summary.f_vis,
                                    f_dur_days: summary.duration_days,
                                    f_mag: summary.f_mag,
                                });
                            }
                        }
                        rows.push((fields, samples));
                    }
                    rows
                }));
            }
            handles
                .into_iter()
                .map(|handle| handle.join().expect("grid worker panicked"))
                .collect()
        });
    for worker_rows in row_results {
        for (fields, samples) in worker_rows {
            node_fields.extend(fields);
            raw_samples.extend(samples);
        }
    }

    let local_grid = if config.include_local_grid {
        raw_samples
            .iter()
            .map(|raw| raw.to_sample(engine))
            .collect()
    } else {
        Vec::new()
    };

    let isolines = if config.include_isolines {
        let f_vis: Vec<f64> = node_fields.iter().map(|f| f.f_vis).collect();
        let f_dur: Vec<f64> = node_fields.iter().map(|f| f.f_dur_days).collect();
        let f_mag: Vec<f64> = node_fields.iter().map(|f| f.f_mag).collect();

        let point = |row_f: f64, col_f: f64| -> EclipseGeoPoint {
            let row = (row_f.floor() as usize).min(lats.len() - 2);
            let t = row_f - row as f64;
            EclipseGeoPoint {
                latitude_deg: lats[row] + (lats[row + 1] - lats[row]) * t,
                longitude_deg: normalize_lon(lon0 + col_f * lon_step),
            }
        };
        let summarize_at = |row_f: f64, col_f: f64| -> PointSummary {
            let geo = point(row_f, col_f);
            summarize_point(
                &table,
                &ObserverPoint::new(geo.latitude_deg, geo.longitude_deg),
            )
        };
        let field_vis = |row_f: f64, col_f: f64| -> f64 { summarize_at(row_f, col_f).f_vis };
        let field_dur =
            |row_f: f64, col_f: f64| -> f64 { summarize_at(row_f, col_f).duration_days };
        let field_mag = |row_f: f64, col_f: f64| -> f64 { summarize_at(row_f, col_f).f_mag };

        let grid = |values: &[f64], field: &dyn Fn(f64, f64) -> f64, level: f64| {
            extract_rings(
                &ContourGrid {
                    n_rows,
                    n_cols: n_lon,
                    wraps: true,
                    values,
                    point: &point,
                    field,
                },
                level,
            )
        };

        let visibility_boundary = grid(&f_vis, &field_vis, 0.0);
        let duration_isolines = config
            .effective_duration_isoline_fractions()
            .into_iter()
            .map(|fraction| SuryaDurationIsoline {
                fraction,
                rings: grid(&f_dur, &field_dur, fraction * span_days),
            })
            .collect();
        let magnitude_isolines = config
            .effective_magnitude_isoline_levels()
            .into_iter()
            .map(|level| SuryaMagnitudeIsoline {
                level,
                rings: grid(&f_mag, &field_mag, level),
            })
            .collect();
        Some(SuryaIsolines {
            visibility_boundary,
            duration_isolines,
            magnitude_isolines,
        })
    } else {
        None
    };

    Ok(FieldProducts {
        local_grid,
        isolines,
    })
}

struct PointSummaryAt {
    latitude_deg: f64,
    longitude_deg: f64,
    summary: PointSummary,
}

impl PointSummaryAt {
    fn to_sample(&self, engine: &Engine) -> SuryaLocalGridSample {
        let s = &self.summary;
        SuryaLocalGridSample {
            latitude_deg: self.latitude_deg,
            longitude_deg: self.longitude_deg,
            magnitude: s.magnitude,
            obscuration: s.obscuration,
            maximum_jd: s.maximum_jd,
            maximum_utc: UtcTime::from_jd_tdb(s.maximum_jd, engine.lsk()),
            first_contact_jd: s.first_contact_jd,
            first_contact_utc: UtcTime::from_jd_tdb(s.first_contact_jd, engine.lsk()),
            last_contact_jd: s.last_contact_jd,
            last_contact_utc: UtcTime::from_jd_tdb(s.last_contact_jd, engine.lsk()),
            visible_duration_seconds: s.duration_days * 86_400.0,
        }
    }
}

// ---------------------------------------------------------------------------
// 6a: instantaneous visibility ring at one moment
// ---------------------------------------------------------------------------

/// The instantaneous penumbral visibility region at one moment: the closed
/// ring enclosing every location with a partial phase in progress and the
/// Sun up (same clip convention as the Change 5 visibility boundary, so the
/// ring always lies inside it). Returns None when the region is empty or
/// smaller than the sampling grid — at exact C1/C4 tangency the region
/// degenerates toward a point.
///
/// This deliberately differs from the sampled `footprints` rings, which
/// keep the raw cone-ellipsoid intersection: near the contacts a grazing
/// cone's ring includes a night-side sliver past the terminator that never
/// sees the eclipse.
pub(crate) fn instantaneous_visibility_ring(
    engine: &Engine,
    eop: Option<&EopKernel>,
    jd_tdb: f64,
) -> Result<Option<SuryaIsolineRing>, SearchError> {
    let (sun, moon) = sun_moon_true_vectors(engine, jd_tdb)?;
    let gast = gast_rad_for(engine, eop, jd_tdb);
    let field = move |lat: f64, lon: f64| -> f64 {
        eval_with(sun, moon, gast, &ObserverPoint::new(lat, lon)).visibility_margin()
    };

    const STEP_DEG: f64 = 1.0;
    let n_lat = (180.0 / STEP_DEG) as usize;
    let n_lon = (360.0 / STEP_DEG) as usize;
    let mut lats: Vec<f64> = Vec::with_capacity(n_lat + 2);
    lats.push(-90.0);
    for i in 0..n_lat {
        lats.push(-90.0 + (i as f64 + 0.5) * STEP_DEG);
    }
    lats.push(90.0);
    let lon0 = -180.0 + 0.5 * STEP_DEG;

    let mut values = Vec::with_capacity(lats.len() * n_lon);
    for lat in &lats {
        if *lat <= -90.0 + 1.0e-9 || *lat >= 90.0 - 1.0e-9 {
            let pole = field(*lat, 0.0);
            values.extend(std::iter::repeat_n(pole, n_lon));
        } else {
            for col in 0..n_lon {
                values.push(field(*lat, normalize_lon(lon0 + col as f64 * STEP_DEG)));
            }
        }
    }

    let lats_ref = &lats;
    let point = move |row_f: f64, col_f: f64| -> EclipseGeoPoint {
        let row = (row_f.floor() as usize).min(lats_ref.len() - 2);
        let t = row_f - row as f64;
        EclipseGeoPoint {
            latitude_deg: lats_ref[row] + (lats_ref[row + 1] - lats_ref[row]) * t,
            longitude_deg: normalize_lon(lon0 + col_f * STEP_DEG),
        }
    };
    let field_at = |row_f: f64, col_f: f64| -> f64 {
        let geo = point(row_f, col_f);
        field(geo.latitude_deg, geo.longitude_deg)
    };
    let rings = extract_rings(
        &ContourGrid {
            n_rows: lats.len(),
            n_cols: n_lon,
            wraps: true,
            values: &values,
            point: &point,
            field: &field_at,
        },
        0.0,
    );
    Ok(rings
        .into_iter()
        .max_by_key(|ring| ring.boundary.len()))
}

// ---------------------------------------------------------------------------
// 5c: swept central corridor over a track-aligned grid
// ---------------------------------------------------------------------------

/// Central ground track used to align the corridor grid: timestamped points
/// at or near the shadow axis.
pub(crate) struct CorridorTrack {
    /// (jd_tdb, point) samples ordered by time.
    pub points: Vec<(f64, EclipseGeoPoint)>,
    /// Additional geographic points (path limits, end outlines) that the
    /// corridor grid must cover.
    pub extra: Vec<EclipseGeoPoint>,
}

fn latlon_to_unit(latitude_deg: f64, longitude_deg: f64) -> [f64; 3] {
    let lat = latitude_deg.to_radians();
    let lon = longitude_deg.to_radians();
    [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
}

fn unit_to_latlon(v: [f64; 3]) -> (f64, f64) {
    (
        v[2].clamp(-1.0, 1.0).asin().to_degrees(),
        v[1].atan2(v[0]).to_degrees(),
    )
}

fn unit_normalize(v: [f64; 3]) -> [f64; 3] {
    let n = norm(v);
    if n <= f64::EPSILON {
        [0.0, 0.0, 1.0]
    } else {
        scale(v, 1.0 / n)
    }
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Densified, extended central track with a left-normal frame per node.
struct TrackFrame {
    positions: Vec<[f64; 3]>,
    normals: Vec<[f64; 3]>,
    jds: Vec<f64>,
}

impl TrackFrame {
    /// Map fractional along-track coordinate and cross-track offset
    /// (degrees, positive to the left of travel) to a point on the sphere.
    fn offset_point(&self, s_f: f64, cross_deg: f64) -> EclipseGeoPoint {
        let index = (s_f.floor() as usize).min(self.positions.len() - 2);
        let t = s_f - index as f64;
        let lerp = |a: [f64; 3], b: [f64; 3]| {
            unit_normalize([
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
            ])
        };
        let position = lerp(self.positions[index], self.positions[index + 1]);
        let normal = lerp(self.normals[index], self.normals[index + 1]);
        let d = cross_deg.to_radians();
        let offset = unit_normalize([
            position[0] * d.cos() + normal[0] * d.sin(),
            position[1] * d.cos() + normal[1] * d.sin(),
            position[2] * d.cos() + normal[2] * d.sin(),
        ]);
        let (latitude_deg, longitude_deg) = unit_to_latlon(offset);
        EclipseGeoPoint {
            latitude_deg,
            longitude_deg,
        }
    }

    fn jd_at(&self, s_f: f64) -> f64 {
        let index = (s_f.floor() as usize).min(self.jds.len() - 2);
        let t = s_f - index as f64;
        self.jds[index] + (self.jds[index + 1] - self.jds[index]) * t
    }

    /// Build from raw track points: dedup, subdivide to `CORRIDOR_ALONG_STEP_KM`,
    /// then extend both ends along the final bearings by `extend_deg`.
    fn build(points: &[(f64, EclipseGeoPoint)], extend_deg: f64) -> Option<Self> {
        let mut base: Vec<(f64, [f64; 3])> = Vec::with_capacity(points.len());
        for (jd, point) in points {
            let unit = latlon_to_unit(point.latitude_deg, point.longitude_deg);
            if let Some((_, previous)) = base.last()
                && norm(sub(unit, *previous)) < 1.0e-9
            {
                continue;
            }
            base.push((*jd, unit));
        }
        if base.len() < 2 {
            return None;
        }

        let step_rad = CORRIDOR_ALONG_STEP_KM / EARTH_RADIUS_KM;
        let mut positions: Vec<[f64; 3]> = Vec::new();
        let mut jds: Vec<f64> = Vec::new();
        for pair in base.windows(2) {
            let (jd_a, a) = pair[0];
            let (jd_b, b) = pair[1];
            let arc = dot(a, b).clamp(-1.0, 1.0).acos();
            let subdivisions = ((arc / step_rad).ceil() as usize).max(1);
            for k in 0..subdivisions {
                let t = k as f64 / subdivisions as f64;
                positions.push(unit_normalize([
                    a[0] + (b[0] - a[0]) * t,
                    a[1] + (b[1] - a[1]) * t,
                    a[2] + (b[2] - a[2]) * t,
                ]));
                jds.push(jd_a + (jd_b - jd_a) * t);
            }
        }
        positions.push(base[base.len() - 1].1);
        jds.push(base[base.len() - 1].0);

        // Extend both ends along the local bearing to cover the end caps.
        let extend_steps = ((extend_deg.to_radians() / step_rad).ceil() as usize).max(1);
        let extend = |from: [f64; 3], toward: [f64; 3]| -> Vec<[f64; 3]> {
            let tangent = unit_normalize(sub(from, scale(toward, dot(from, toward))));
            (1..=extend_steps)
                .map(|k| {
                    let angle = k as f64 * step_rad;
                    unit_normalize([
                        from[0] * angle.cos() + tangent[0] * angle.sin(),
                        from[1] * angle.cos() + tangent[1] * angle.sin(),
                        from[2] * angle.cos() + tangent[2] * angle.sin(),
                    ])
                })
                .collect()
        };
        let head = extend(positions[0], positions[1]);
        let tail = extend(
            positions[positions.len() - 1],
            positions[positions.len() - 2],
        );
        let head_jd = jds[0];
        let tail_jd = jds[jds.len() - 1];
        let mut all_positions: Vec<[f64; 3]> = head.into_iter().rev().collect();
        let mut all_jds = vec![head_jd; all_positions.len()];
        all_positions.extend(positions);
        all_jds.extend(jds);
        all_positions.extend(tail);
        all_jds.extend(std::iter::repeat_n(tail_jd, extend_steps));

        // Left-normal per node from central-difference tangents.
        let count = all_positions.len();
        let mut normals = Vec::with_capacity(count);
        for index in 0..count {
            let before = all_positions[index.saturating_sub(1)];
            let after = all_positions[(index + 1).min(count - 1)];
            let position = all_positions[index];
            let tangent_raw = sub(after, before);
            let tangent = unit_normalize(sub(
                tangent_raw,
                scale(position, dot(tangent_raw, position)),
            ));
            normals.push(unit_normalize(cross3(position, tangent)));
        }
        Some(Self {
            positions: all_positions,
            normals,
            jds: all_jds,
        })
    }
}

/// Windowed maxima of the Sun-up-clipped total/annular shadow margins at
/// one ground point, searching only near the shadow's local passage time.
///
/// The Sun-up clip (same -0.833 degree convention as everywhere else)
/// excludes the phantom region where the shadow cone exits through the
/// night side of the ellipsoid; the physical corridor lies on the day side
/// by construction, so interior values are unaffected.
fn central_maxima_windowed(
    table: &FieldTable,
    observer: &ObserverPoint,
    jd_center: f64,
) -> (f64, f64) {
    let clipped_total =
        |eval: &PointEval| -> f64 { eval.total_margin().min(eval.altitude_margin()) };
    let clipped_annular =
        |eval: &PointEval| -> f64 { eval.annular_margin().min(eval.altitude_margin()) };
    let (first, last) = table.index_range(
        jd_center - CORRIDOR_TIME_HALF_WINDOW_DAYS,
        jd_center + CORRIDOR_TIME_HALF_WINDOW_DAYS,
    );
    let mut best_total = f64::NEG_INFINITY;
    let mut best_total_index = first;
    let mut best_annular = f64::NEG_INFINITY;
    let mut best_annular_index = first;
    for index in first..=last {
        let eval = table.eval_index(index, observer);
        let total = clipped_total(&eval);
        if total > best_total {
            best_total = total;
            best_total_index = index;
        }
        let annular = clipped_annular(&eval);
        if annular > best_annular {
            best_annular = annular;
            best_annular_index = index;
        }
    }
    let bracket = |index: usize| -> (f64, f64) {
        (
            table.jd_at(index.max(first + 1) - 1),
            table.jd_at((index + 1).min(last)),
        )
    };
    let (total_left, total_right) = bracket(best_total_index);
    let (_, f_total) = golden_max(total_left, total_right, |jd| {
        clipped_total(&table.eval_jd(jd, observer))
    });
    let (annular_left, annular_right) = bracket(best_annular_index);
    let (_, f_annular) = golden_max(annular_left, annular_right, |jd| {
        clipped_annular(&table.eval_jd(jd, observer))
    });
    (f_total, f_annular)
}

fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let dphi = phi2 - phi1;
    let dlambda = (lon2 - lon1).to_radians();
    let h = (dphi * 0.5).sin().powi(2) + phi1.cos() * phi2.cos() * (dlambda * 0.5).sin().powi(2);
    2.0 * EARTH_RADIUS_KM * h.sqrt().asin()
}

/// Build the swept central corridor for one event.
///
/// `window` should span the central phase (C2..C3 with margin). Returns
/// segments grouped per disjoint swept ring, ordered along the path by time.
pub(crate) fn central_corridor(
    engine: &Engine,
    eop: Option<&EopKernel>,
    window_start_jd: f64,
    window_end_jd: f64,
    track: &CorridorTrack,
) -> Result<SuryaCentralCorridor, SearchError> {
    if track.points.len() < 2 {
        return Ok(SuryaCentralCorridor {
            segments: Vec::new(),
        });
    }

    let window_days = window_end_jd - window_start_jd;
    let cadence_days = (window_days / 600.0).clamp(5.0 / 86_400.0, 30.0 / 86_400.0);
    let table = FieldTable::build(engine, eop, window_start_jd, window_end_jd, cadence_days)?;

    // Cross-track half range: cover the limits seen in `extra` plus margin.
    // Grazing instantaneous outlines can include the cone's remote branch on
    // the far side of the ellipsoid, so only near-track extras contribute
    // and the range is capped; the boundary-positive retry below widens it
    // if the sampled outlines undershot the true swept extent.
    let mut cross_half_deg = 1.5_f64;
    for point in &track.extra {
        let mut nearest = f64::INFINITY;
        for (_, center) in &track.points {
            let d = haversine_km(
                point.latitude_deg,
                point.longitude_deg,
                center.latitude_deg,
                center.longitude_deg,
            );
            nearest = nearest.min(d);
        }
        if nearest <= 600.0 {
            cross_half_deg = cross_half_deg.max(nearest / 111.0 + 1.0);
        }
    }
    cross_half_deg = cross_half_deg.min(6.0);
    let mut extend_deg = cross_half_deg + 1.5;

    let mut attempt = 0;
    let (frame, n_s, n_d, cross0, cross_step, f_total, f_annular) = loop {
        attempt += 1;
        let Some(frame) = TrackFrame::build(&track.points, extend_deg) else {
            return Ok(SuryaCentralCorridor {
                segments: Vec::new(),
            });
        };
        let n_s = frame.positions.len();
        let mut cross_step = CORRIDOR_CROSS_STEP_DEG;
        while (2.0 * cross_half_deg / cross_step) as usize * n_s > 600_000 {
            cross_step *= 1.5;
        }
        let n_d = ((2.0 * cross_half_deg / cross_step).ceil() as usize).max(4) + 1;
        let cross0 = -cross_half_deg;

        // Node margins, parallel over along-track rows.
        let worker_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .min(n_s);
        let rows_per_worker = n_s.div_ceil(worker_count);
        let row_results: Vec<Vec<(Vec<f64>, Vec<f64>)>> = std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for worker in 0..worker_count {
                let row_start = worker * rows_per_worker;
                let row_end = ((worker + 1) * rows_per_worker).min(n_s);
                let table_ref = &table;
                let frame_ref = &frame;
                handles.push(scope.spawn(move || {
                    let mut rows = Vec::new();
                    for row in row_start..row_end {
                        let jd_center = frame_ref.jds[row];
                        let mut totals = Vec::with_capacity(n_d);
                        let mut annulars = Vec::with_capacity(n_d);
                        for col in 0..n_d {
                            let cross = cross0 + col as f64 * cross_step;
                            let geo = frame_ref.offset_point(row as f64, cross);
                            let observer =
                                ObserverPoint::new(geo.latitude_deg, geo.longitude_deg);
                            let (f_total, f_annular) =
                                central_maxima_windowed(table_ref, &observer, jd_center);
                            totals.push(f_total);
                            annulars.push(f_annular);
                        }
                        rows.push((totals, annulars));
                    }
                    rows
                }));
            }
            handles
                .into_iter()
                .map(|handle| handle.join().expect("corridor worker panicked"))
                .collect()
        });
        let mut f_total: Vec<f64> = Vec::with_capacity(n_s * n_d);
        let mut f_annular: Vec<f64> = Vec::with_capacity(n_s * n_d);
        for worker_rows in row_results {
            for (totals, annulars) in worker_rows {
                f_total.extend(totals);
                f_annular.extend(annulars);
            }
        }

        if std::env::var("DHRUV_DEBUG_CORRIDOR").is_ok() {
            let max_total = f_total.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let max_annular = f_annular.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            eprintln!(
                "corridor dbg: attempt={attempt} n_s={n_s} n_d={n_d} cross_half={cross_half_deg:.2} extend={extend_deg:.2} cross_step={cross_step:.3} max_total={max_total:.6} max_annular={max_annular:.6} window=[{window_start_jd:.5},{window_end_jd:.5}]"
            );
        }

        // The swept region must lie strictly inside the grid.
        let node = |row: usize, col: usize| row * n_d + col;
        let mut boundary_positive = false;
        'check: for row in 0..n_s {
            for col in 0..n_d {
                let on_edge = row == 0 || row + 1 == n_s || col == 0 || col + 1 == n_d;
                if on_edge && (f_total[node(row, col)] > 0.0 || f_annular[node(row, col)] > 0.0) {
                    boundary_positive = true;
                    break 'check;
                }
            }
        }
        if !boundary_positive || attempt >= 3 {
            break (frame, n_s, n_d, cross0, cross_step, f_total, f_annular);
        }
        cross_half_deg *= 1.8;
        extend_deg *= 1.8;
    };

    let point = |row_f: f64, col_f: f64| -> EclipseGeoPoint {
        frame.offset_point(row_f, cross0 + col_f * cross_step)
    };
    let field_total = |row_f: f64, col_f: f64| -> f64 {
        let geo = point(row_f, col_f);
        central_maxima_windowed(
            &table,
            &ObserverPoint::new(geo.latitude_deg, geo.longitude_deg),
            frame.jd_at(row_f),
        )
        .0
    };
    let field_annular = |row_f: f64, col_f: f64| -> f64 {
        let geo = point(row_f, col_f);
        central_maxima_windowed(
            &table,
            &ObserverPoint::new(geo.latitude_deg, geo.longitude_deg),
            frame.jd_at(row_f),
        )
        .1
    };

    let total_rings = extract_rings(
        &ContourGrid {
            n_rows: n_s,
            n_cols: n_d,
            wraps: false,
            values: &f_total,
            point: &point,
            field: &field_total,
        },
        0.0,
    );
    let annular_rings = extract_rings(
        &ContourGrid {
            n_rows: n_s,
            n_cols: n_d,
            wraps: false,
            values: &f_annular,
            point: &point,
            field: &field_annular,
        },
        0.0,
    );

    // One segment per disjoint swept ring, ordered along the path by the
    // time of the nearest track sample.
    let ring_time = |ring: &SuryaIsolineRing| -> f64 {
        let anchor = ring.boundary[0];
        let mut best = (f64::INFINITY, track.points[0].0);
        for (jd, center) in &track.points {
            let d = haversine_km(
                anchor.latitude_deg,
                anchor.longitude_deg,
                center.latitude_deg,
                center.longitude_deg,
            );
            if d < best.0 {
                best = (d, *jd);
            }
        }
        best.1
    };
    let mut segments: Vec<(f64, SuryaCorridorSegment)> = Vec::new();
    for ring in total_rings {
        segments.push((
            ring_time(&ring),
            SuryaCorridorSegment {
                grahan_type: SuryaGrahanType::Total,
                rings: vec![ring],
            },
        ));
    }
    for ring in annular_rings {
        segments.push((
            ring_time(&ring),
            SuryaCorridorSegment {
                grahan_type: SuryaGrahanType::Annular,
                rings: vec![ring],
            },
        ));
    }
    segments.sort_by(|a, b| a.0.total_cmp(&b.0));

    Ok(SuryaCentralCorridor {
        segments: segments.into_iter().map(|(_, segment)| segment).collect(),
    })
}

// ---------------------------------------------------------------------------
// Tests (pure math only; kernel-dependent checks live in tests/)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Grid whose rows/cols map directly to degrees: row -> lat, col -> lon.
    struct PlainGrid {
        lat0: f64,
        lon0: f64,
        step: f64,
    }

    impl PlainGrid {
        fn point(&self, row_f: f64, col_f: f64) -> EclipseGeoPoint {
            EclipseGeoPoint {
                latitude_deg: self.lat0 + row_f * self.step,
                longitude_deg: normalize_lon(self.lon0 + col_f * self.step),
            }
        }
    }

    fn build_values(
        n_rows: usize,
        n_cols: usize,
        grid: &PlainGrid,
        field: &dyn Fn(f64, f64) -> f64,
    ) -> Vec<f64> {
        let mut values = Vec::with_capacity(n_rows * n_cols);
        for row in 0..n_rows {
            for col in 0..n_cols {
                let p = grid.point(row as f64, col as f64);
                values.push(field(p.latitude_deg, p.longitude_deg));
            }
        }
        values
    }

    #[test]
    fn circle_contour_is_closed_and_counterclockwise() {
        let plain = PlainGrid {
            lat0: -30.0,
            lon0: -30.0,
            step: 1.0,
        };
        let field = |lat: f64, lon: f64| -> f64 { 10.0 - (lat * lat + lon * lon).sqrt() };
        let n_rows = 61;
        let n_cols = 61;
        let values = build_values(n_rows, n_cols, &plain, &field);
        let point = |row_f: f64, col_f: f64| plain.point(row_f, col_f);
        let field_at = |row_f: f64, col_f: f64| {
            let p = plain.point(row_f, col_f);
            field(p.latitude_deg, p.longitude_deg)
        };
        let rings = extract_rings(
            &ContourGrid {
                n_rows,
                n_cols,
                wraps: false,
                values: &values,
                point: &point,
                field: &field_at,
            },
            0.0,
        );
        assert_eq!(rings.len(), 1);
        let ring = &rings[0];
        assert!(ring.contains_pole.is_none());
        assert_eq!(ring.boundary.first(), ring.boundary.last());
        assert!(ring.boundary.len() > 20);
        // All points on the radius-10 circle within refinement tolerance.
        for point in &ring.boundary {
            let r = (point.latitude_deg.powi(2) + point.longitude_deg.powi(2)).sqrt();
            assert!((r - 10.0).abs() < 0.05, "r = {r}");
        }
        // Counterclockwise (inside on the left): shoelace area positive.
        let area: f64 = ring
            .boundary
            .windows(2)
            .map(|pair| {
                pair[0].longitude_deg * pair[1].latitude_deg
                    - pair[1].longitude_deg * pair[0].latitude_deg
            })
            .sum();
        assert!(area > 0.0, "area = {area}");
    }

    #[test]
    fn polar_cap_contour_winds_and_reports_pole() {
        // Region: everything north of 60 degrees, on a wrapped global grid
        // with pole rows.
        let mut lats: Vec<f64> = vec![-90.0];
        for i in 0..90 {
            lats.push(-90.0 + (i as f64 + 0.5) * 2.0);
        }
        lats.push(90.0);
        let n_rows = lats.len();
        let n_cols = 180;
        let lon0 = -179.0;
        let step = 2.0;
        let field = |lat: f64, _lon: f64| -> f64 { lat - 60.0 };
        let mut values = Vec::new();
        for lat in &lats {
            for _ in 0..n_cols {
                values.push(field(*lat, 0.0));
            }
        }
        let lats_ref = &lats;
        let point = move |row_f: f64, col_f: f64| -> EclipseGeoPoint {
            let row = (row_f.floor() as usize).min(lats_ref.len() - 2);
            let t = row_f - row as f64;
            EclipseGeoPoint {
                latitude_deg: lats_ref[row] + (lats_ref[row + 1] - lats_ref[row]) * t,
                longitude_deg: normalize_lon(lon0 + col_f * step),
            }
        };
        let field_at = |row_f: f64, col_f: f64| {
            let p = point(row_f, col_f);
            field(p.latitude_deg, p.longitude_deg)
        };
        let rings = extract_rings(
            &ContourGrid {
                n_rows,
                n_cols,
                wraps: true,
                values: &values,
                point: &point,
                field: &field_at,
            },
            0.0,
        );
        assert_eq!(rings.len(), 1);
        let ring = &rings[0];
        assert_eq!(ring.contains_pole, Some(PoleSide::North));
        assert_eq!(ring.boundary.first(), ring.boundary.last());
        for point in &ring.boundary {
            assert!((point.latitude_deg - 60.0).abs() < 0.05);
        }
    }

    #[test]
    fn antimeridian_region_stays_ordered() {
        // Circular region centered on the antimeridian at (0, 180).
        let mut lats: Vec<f64> = vec![-90.0];
        for i in 0..90 {
            lats.push(-90.0 + (i as f64 + 0.5) * 2.0);
        }
        lats.push(90.0);
        let n_rows = lats.len();
        let n_cols = 180;
        let lon0 = -179.0;
        let step = 2.0;
        let field = |lat: f64, lon: f64| -> f64 {
            let dlon = wrap_delta(lon - 180.0);
            15.0 - (lat * lat + dlon * dlon).sqrt()
        };
        let mut values = Vec::new();
        for lat in &lats {
            for col in 0..n_cols {
                values.push(field(*lat, normalize_lon(lon0 + col as f64 * step)));
            }
        }
        let lats_ref = &lats;
        let point = move |row_f: f64, col_f: f64| -> EclipseGeoPoint {
            let row = (row_f.floor() as usize).min(lats_ref.len() - 2);
            let t = row_f - row as f64;
            EclipseGeoPoint {
                latitude_deg: lats_ref[row] + (lats_ref[row + 1] - lats_ref[row]) * t,
                longitude_deg: normalize_lon(lon0 + col_f * step),
            }
        };
        let field_at = |row_f: f64, col_f: f64| {
            let p = point(row_f, col_f);
            field(p.latitude_deg, p.longitude_deg)
        };
        let rings = extract_rings(
            &ContourGrid {
                n_rows,
                n_cols,
                wraps: true,
                values: &values,
                point: &point,
                field: &field_at,
            },
            0.0,
        );
        assert_eq!(rings.len(), 1);
        let ring = &rings[0];
        assert!(ring.contains_pole.is_none());
        // Ordered continuously: consecutive longitude deltas after unwrap
        // stay small even across the seam.
        for pair in ring.boundary.windows(2) {
            let delta = wrap_delta(pair[1].longitude_deg - pair[0].longitude_deg).abs();
            assert!(delta < 30.0, "delta = {delta}");
        }
        // Both sides of the seam are present.
        assert!(ring.boundary.iter().any(|p| p.longitude_deg > 170.0));
        assert!(ring.boundary.iter().any(|p| p.longitude_deg < -170.0));
    }

    #[test]
    fn disjoint_regions_yield_two_rings() {
        let plain = PlainGrid {
            lat0: -40.0,
            lon0: -40.0,
            step: 1.0,
        };
        let field = |lat: f64, lon: f64| -> f64 {
            let a = 8.0 - ((lat - 20.0).powi(2) + lon * lon).sqrt();
            let b = 8.0 - ((lat + 20.0).powi(2) + lon * lon).sqrt();
            a.max(b)
        };
        let n_rows = 81;
        let n_cols = 81;
        let values = build_values(n_rows, n_cols, &plain, &field);
        let point = |row_f: f64, col_f: f64| plain.point(row_f, col_f);
        let field_at = |row_f: f64, col_f: f64| {
            let p = plain.point(row_f, col_f);
            field(p.latitude_deg, p.longitude_deg)
        };
        let rings = extract_rings(
            &ContourGrid {
                n_rows,
                n_cols,
                wraps: false,
                values: &values,
                point: &point,
                field: &field_at,
            },
            0.0,
        );
        assert_eq!(rings.len(), 2);
    }

    #[test]
    fn ring_start_is_deterministic() {
        let plain = PlainGrid {
            lat0: -15.0,
            lon0: -15.0,
            step: 1.0,
        };
        let field = |lat: f64, lon: f64| -> f64 { 10.0 - (lat * lat + lon * lon).sqrt() };
        let n_rows = 31;
        let n_cols = 31;
        let values = build_values(n_rows, n_cols, &plain, &field);
        let point = |row_f: f64, col_f: f64| plain.point(row_f, col_f);
        let field_at = |row_f: f64, col_f: f64| {
            let p = plain.point(row_f, col_f);
            field(p.latitude_deg, p.longitude_deg)
        };
        let grid = ContourGrid {
            n_rows,
            n_cols,
            wraps: false,
            values: &values,
            point: &point,
            field: &field_at,
        };
        let first = extract_rings(&grid, 0.0);
        let second = extract_rings(&grid, 0.0);
        assert_eq!(first, second);
        // Starts at the lexicographically smallest (lat, lon) vertex.
        let ring = &first[0];
        let min_point = ring
            .boundary
            .iter()
            .min_by(|a, b| {
                (a.latitude_deg, a.longitude_deg)
                    .partial_cmp(&(b.latitude_deg, b.longitude_deg))
                    .unwrap()
            })
            .unwrap();
        assert_eq!(ring.boundary[0], *min_point);
    }

    #[test]
    fn thin_band_along_diagonal_stays_connected() {
        // A 0.3-degree-wide band along the diagonal of a coarse 1-degree
        // grid: node sampling alone would fragment it, which is exactly the
        // corridor situation; the track-aligned corridor grid avoids it by
        // aligning rows with the band. Here we verify the engine itself on
        // an aligned fine grid.
        let plain = PlainGrid {
            lat0: -1.0,
            lon0: -10.0,
            step: 0.1,
        };
        let field = |lat: f64, lon: f64| -> f64 {
            // Band |lat| < 0.15 for lon in [-8, 8], capped ends.
            let along = lon.abs() - 8.0;
            let across = lat.abs() - 0.15;
            -across.max(along)
        };
        let n_rows = 21;
        let n_cols = 201;
        let values = build_values(n_rows, n_cols, &plain, &field);
        let point = |row_f: f64, col_f: f64| plain.point(row_f, col_f);
        let field_at = |row_f: f64, col_f: f64| {
            let p = plain.point(row_f, col_f);
            field(p.latitude_deg, p.longitude_deg)
        };
        let rings = extract_rings(
            &ContourGrid {
                n_rows,
                n_cols,
                wraps: false,
                values: &values,
                point: &point,
                field: &field_at,
            },
            0.0,
        );
        assert_eq!(rings.len(), 1);
        assert!(rings[0].boundary.len() > 100);
    }

    #[test]
    fn track_frame_offsets_are_perpendicular() {
        let points = vec![
            (
                0.0,
                EclipseGeoPoint {
                    latitude_deg: 0.0,
                    longitude_deg: 0.0,
                },
            ),
            (
                0.01,
                EclipseGeoPoint {
                    latitude_deg: 1.0,
                    longitude_deg: 10.0,
                },
            ),
            (
                0.02,
                EclipseGeoPoint {
                    latitude_deg: 2.0,
                    longitude_deg: 20.0,
                },
            ),
        ];
        let frame = TrackFrame::build(&points, 2.0).expect("frame");
        // Offsetting by +1 degree moves ~111 km from the on-track point.
        let mid = frame.positions.len() as f64 * 0.5;
        let on_track = frame.offset_point(mid, 0.0);
        let offset = frame.offset_point(mid, 1.0);
        let d = haversine_km(
            on_track.latitude_deg,
            on_track.longitude_deg,
            offset.latitude_deg,
            offset.longitude_deg,
        );
        assert!((d - 111.0).abs() < 2.0, "d = {d}");
        // jd interpolation is monotonic within the original span.
        assert!(frame.jd_at(0.0) <= frame.jd_at(mid));
        assert!(frame.jd_at(mid) <= frame.jd_at(frame.positions.len() as f64 - 1.0));
    }

    #[test]
    fn wrap_delta_normalizes() {
        assert_eq!(wrap_delta(0.0), 0.0);
        assert!((wrap_delta(190.0) + 170.0).abs() < 1e-12);
        assert!((wrap_delta(-190.0) - 170.0).abs() < 1e-12);
    }

    #[test]
    fn normalize_lon_maps_into_range() {
        assert_eq!(normalize_lon(0.0), 0.0);
        assert_eq!(normalize_lon(180.0), 180.0);
        assert_eq!(normalize_lon(-180.0), -180.0);
        assert_eq!(normalize_lon(190.0), -170.0);
        assert_eq!(normalize_lon(360.0), 0.0);
        assert_eq!(normalize_lon(540.0), 180.0);
    }
}
