//! General precession in ecliptic longitude and full 3D ecliptic precession
//! matrix (J2000 ↔ ecliptic-of-date).
//!
//! The general precession p_A measures the accumulated westward motion of
//! the vernal equinox along the ecliptic since J2000.0. This is the
//! foundational quantity for computing ayanamsha at any epoch.
//!
//! The full 3D ecliptic precession rotation uses model-dependent ecliptic
//! parameters π_A (inclination) and Π_A (node longitude).
//!
//! Sources:
//! - Lieske, Lederle, Fricke & Morando 1977, A&A 58, 1-16 (IAU 1976).
//! - Lieske 1979, A&A 73, 282-284 (errata/updates).
//! - Capitaine, Wallace & Chapront 2003, A&A 412, 567-586, Table 1.
//! - IERS Conventions 2010, Chapter 5, Table 5.1.
//! - Vondrák, Capitaine & Wallace 2011, A&A 534, A22.

use std::f64::consts::{PI, TAU};

/// Supported precession models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrecessionModel {
    /// Lieske 1977 / IAU 1976 precession.
    Lieske1977,
    /// IAU 2006 (Capitaine et al. 2003 / IERS 2010).
    Iau2006,
    /// Vondrák, Capitaine & Wallace 2011 long-term model.
    Vondrak2011,
}

/// Default precession model used by wrapper functions.
pub const DEFAULT_PRECESSION_MODEL: PrecessionModel = PrecessionModel::Vondrak2011;

const AS2R: f64 = PI / 648_000.0;

#[derive(Clone, Copy)]
struct VondrakTable1Term {
    period_centuries: f64,
    ap: f64,
    bp: f64,
    aq: f64,
    bq: f64,
}

#[derive(Clone, Copy)]
struct VondrakTable3Term {
    period_centuries: f64,
    cp: f64,
    sp: f64,
}

const VON_TABLE1_TERMS: [VondrakTable1Term; 8] = [
    VondrakTable1Term {
        period_centuries: 708.15,
        ap: -5_486.751_211,
        bp: -684.661_560,
        aq: 667.666_730,
        bq: -5_523.863_691,
    },
    VondrakTable1Term {
        period_centuries: 2309.0,
        ap: -17.127_623,
        bp: 2_446.283_880,
        aq: -2_354.886_252,
        bq: -549.747_450,
    },
    VondrakTable1Term {
        period_centuries: 1620.0,
        ap: -617.517_403,
        bp: 399.671_049,
        aq: -428.152_441,
        bq: -310.998_056,
    },
    VondrakTable1Term {
        period_centuries: 492.2,
        ap: 413.442_940,
        bp: -356.652_376,
        aq: 376.202_861,
        bq: 421.535_876,
    },
    VondrakTable1Term {
        period_centuries: 1183.0,
        ap: 78.614_193,
        bp: -186.387_003,
        aq: 184.778_874,
        bq: -36.776_172,
    },
    VondrakTable1Term {
        period_centuries: 622.0,
        ap: -180.732_815,
        bp: -316.800_070,
        aq: 335.321_713,
        bq: -145.278_396,
    },
    VondrakTable1Term {
        period_centuries: 882.0,
        ap: -87.676_083,
        bp: 198.296_701,
        aq: -185.138_669,
        bq: -34.744_450,
    },
    VondrakTable1Term {
        period_centuries: 547.0,
        ap: 46.140_315,
        bp: 101.135_679,
        aq: -120.972_830,
        bq: 22.885_731,
    },
];

const VON_TABLE3_TERMS: [VondrakTable3Term; 10] = [
    VondrakTable3Term {
        period_centuries: 409.90,
        cp: -6_908.287_473,
        sp: -2_845.175_469,
    },
    VondrakTable3Term {
        period_centuries: 396.15,
        cp: -3_198.706_291,
        sp: 449.844_989,
    },
    VondrakTable3Term {
        period_centuries: 537.22,
        cp: 1_453.674_527,
        sp: -1_255.915_323,
    },
    VondrakTable3Term {
        period_centuries: 402.90,
        cp: -857.748_557,
        sp: 886.736_783,
    },
    VondrakTable3Term {
        period_centuries: 417.15,
        cp: 1_173.231_614,
        sp: 418.887_514,
    },
    VondrakTable3Term {
        period_centuries: 288.92,
        cp: -156.981_465,
        sp: 997.912_441,
    },
    VondrakTable3Term {
        period_centuries: 4043.00,
        cp: 371.836_550,
        sp: -240.979_710,
    },
    VondrakTable3Term {
        period_centuries: 306.00,
        cp: -216.619_040,
        sp: 76.541_307,
    },
    VondrakTable3Term {
        period_centuries: 277.00,
        cp: 193.691_479,
        sp: -36.788_069,
    },
    VondrakTable3Term {
        period_centuries: 203.00,
        cp: 11.891_524,
        sp: -170.964_086,
    },
];

#[inline]
fn vondrak2011_periodic_argument_rad(t: f64, period_centuries: f64) -> f64 {
    TAU * t / period_centuries
}

#[inline]
fn vondrak2011_pq_raw_rad(t: f64) -> (f64, f64) {
    let t2 = t * t;
    let t3 = t2 * t;
    let mut p_arcsec = 5_851.607_687 - 0.118_900_0 * t - 0.000_289_13 * t2 + 0.000_000_101 * t3;
    let mut q_arcsec = -1_600.886_300 + 1.168_981_8 * t - 0.000_000_20 * t2 - 0.000_000_437 * t3;
    for term in VON_TABLE1_TERMS {
        let (s, c) = vondrak2011_periodic_argument_rad(t, term.period_centuries).sin_cos();
        // Vondrak Table 1 uses opposite sign for the p-series sine term
        // under this positive argument convention.
        p_arcsec += term.ap * c - term.bp * s;
        q_arcsec += term.aq * c + term.bq * s;
    }
    (p_arcsec * AS2R, q_arcsec * AS2R)
}

#[inline]
fn vondrak2011_pq_rad(t: f64) -> (f64, f64) {
    // Eq. (4)/(5) contain a fitted J2000 offset. Normalize so these track
    // ecliptic precession relative to J2000 in this API.
    let (p, q) = vondrak2011_pq_raw_rad(t);
    let (p0, q0) = vondrak2011_pq_raw_rad(0.0);
    (p - p0, q - q0)
}

#[inline]
fn vondrak2011_pi_cap_pi_rad(t: f64) -> (f64, f64) {
    let (p, q) = vondrak2011_pq_rad(t);
    let sin_pi = (p * p + q * q).sqrt().min(1.0);
    let pi_a = sin_pi.asin();
    let cap_pi_a = p.atan2(q).rem_euclid(TAU);
    (pi_a, cap_pi_a)
}

// ---------- Lieske 1977 / IAU 1976 ----------
// Lieske et al. 1977, A&A 58; Lieske 1979, A&A 73, 282;
// Explanatory Supplement 1992, Ch. 3.

#[inline]
fn lieske1977_general_precession_longitude_arcsec(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    5029.0966 * t + 1.11113 * t2 - 0.000006 * t3
}

#[inline]
fn lieske1977_ecliptic_inclination_arcsec(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    47.0029 * t - 0.06603 * t2 + 0.000598 * t3
}

#[inline]
fn lieske1977_ecliptic_node_longitude_arcsec(t: f64) -> f64 {
    // 174°52'34.982" = 629554.982"
    let t2 = t * t;
    629_554.982 + 3289.4789 * t + 0.60622 * t2
}

#[inline]
fn iau2006_general_precession_longitude_arcsec(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    5028.796195 * t + 1.1054348 * t2 + 0.00007964 * t3 - 0.000023857 * t4 - 0.0000000383 * t5
}

#[inline]
fn iau2006_ecliptic_inclination_arcsec(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    46.998_973 * t - 0.033_492_6 * t2 - 0.000_125_59 * t3 + 0.000_000_113 * t4
        - 0.000_000_002_2 * t5
}

#[inline]
fn iau2006_ecliptic_node_longitude_arcsec(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    629_546.793_6 + 3_289.478_9 * t + 0.606_22 * t2
        - 0.000_83 * t3
        - 0.000_01 * t4
        - 0.000_000_01 * t5
}

#[inline]
fn vondrak2011_general_precession_raw_arcsec(t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mut p_a = 8_134.017_132 + 5_043.052_003_5 * t - 0.007_107_33 * t2 + 0.000_000_271 * t3;
    for term in VON_TABLE3_TERMS {
        let (s, c) = vondrak2011_periodic_argument_rad(t, term.period_centuries).sin_cos();
        p_a += term.cp * c + term.sp * s;
    }
    p_a
}

#[inline]
fn vondrak2011_general_precession_longitude_arcsec(t: f64) -> f64 {
    // Eq. (10) in the paper includes a fitted zero-point offset; normalize
    // so this API remains "accumulated since J2000.0" like the IAU path.
    vondrak2011_general_precession_raw_arcsec(t) - vondrak2011_general_precession_raw_arcsec(0.0)
}

#[inline]
fn vondrak2011_ecliptic_inclination_arcsec(t: f64) -> f64 {
    let (pi_a, _) = vondrak2011_pi_cap_pi_rad(t);
    pi_a.to_degrees() * 3600.0
}

#[inline]
fn vondrak2011_ecliptic_node_longitude_arcsec(t: f64) -> f64 {
    let (_, cap_pi_a) = vondrak2011_pi_cap_pi_rad(t);
    cap_pi_a.to_degrees() * 3600.0
}

/// Default-model general precession in ecliptic longitude, in arcseconds.
///
/// # Arguments
/// * `t` — Julian centuries of TDB since J2000.0: `(JD_TDB - 2451545.0) / 36525.0`
///
/// # Returns
/// Accumulated precession in arcseconds. Positive means the equinox has
/// moved westward (tropical longitudes of stars have increased).
///
/// Typical present-era linear term is about 1.4°/century.
pub fn general_precession_longitude_arcsec(t: f64) -> f64 {
    general_precession_longitude_arcsec_with_model(t, DEFAULT_PRECESSION_MODEL)
}

/// General precession in ecliptic longitude for a specific model, in arcseconds.
pub fn general_precession_longitude_arcsec_with_model(t: f64, model: PrecessionModel) -> f64 {
    match model {
        PrecessionModel::Lieske1977 => lieske1977_general_precession_longitude_arcsec(t),
        PrecessionModel::Iau2006 => iau2006_general_precession_longitude_arcsec(t),
        PrecessionModel::Vondrak2011 => vondrak2011_general_precession_longitude_arcsec(t),
    }
}

/// Default-model general precession in ecliptic longitude, in degrees.
///
/// Same as [`general_precession_longitude_arcsec`] but converted to degrees.
pub fn general_precession_longitude_deg(t: f64) -> f64 {
    general_precession_longitude_deg_with_model(t, DEFAULT_PRECESSION_MODEL)
}

/// General precession in ecliptic longitude for a specific model, in degrees.
pub fn general_precession_longitude_deg_with_model(t: f64, model: PrecessionModel) -> f64 {
    general_precession_longitude_arcsec_with_model(t, model) / 3600.0
}

/// Inclination of the ecliptic of date to the J2000 ecliptic, in arcseconds.
///
/// π_A from IERS Conventions 2010, Table 5.1 (Capitaine et al. 2003).
/// `t` = Julian centuries of TDB since J2000.0.
pub fn ecliptic_inclination_arcsec(t: f64) -> f64 {
    ecliptic_inclination_arcsec_with_model(t, DEFAULT_PRECESSION_MODEL)
}

/// Inclination of the ecliptic of date to the J2000 ecliptic for a specific model, in arcseconds.
pub fn ecliptic_inclination_arcsec_with_model(t: f64, model: PrecessionModel) -> f64 {
    match model {
        PrecessionModel::Lieske1977 => lieske1977_ecliptic_inclination_arcsec(t),
        PrecessionModel::Iau2006 => iau2006_ecliptic_inclination_arcsec(t),
        PrecessionModel::Vondrak2011 => vondrak2011_ecliptic_inclination_arcsec(t),
    }
}

/// Longitude of the ascending node of the ecliptic of date on the J2000
/// ecliptic, in arcseconds.
///
/// Π_A from IERS Conventions 2010, Table 5.1 (Capitaine et al. 2003).
/// `t` = Julian centuries of TDB since J2000.0.
pub fn ecliptic_node_longitude_arcsec(t: f64) -> f64 {
    ecliptic_node_longitude_arcsec_with_model(t, DEFAULT_PRECESSION_MODEL)
}

/// Longitude of the ascending node of the ecliptic of date on the J2000
/// ecliptic for a specific model, in arcseconds.
pub fn ecliptic_node_longitude_arcsec_with_model(t: f64, model: PrecessionModel) -> f64 {
    match model {
        PrecessionModel::Lieske1977 => lieske1977_ecliptic_node_longitude_arcsec(t),
        PrecessionModel::Iau2006 => iau2006_ecliptic_node_longitude_arcsec(t),
        PrecessionModel::Vondrak2011 => vondrak2011_ecliptic_node_longitude_arcsec(t),
    }
}

/// Time derivative of the general precession in ecliptic longitude, in deg/day.
///
/// d(p_A)/dt evaluated at epoch `t` (Julian centuries since J2000.0).
/// Informational: used for documenting the ~50"/yr correction magnitude.
/// Velocity transforms use finite-differencing rather than this scalar.
pub fn general_precession_rate_deg_per_day(t: f64) -> f64 {
    general_precession_rate_deg_per_day_with_model(t, DEFAULT_PRECESSION_MODEL)
}

/// Time derivative of the model-specific general precession in ecliptic longitude, in deg/day.
pub fn general_precession_rate_deg_per_day_with_model(t: f64, model: PrecessionModel) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    // d(p_A)/dt in arcsec/century
    let rate = match model {
        PrecessionModel::Lieske1977 => {
            5029.0966 + 2.0 * 1.11113 * t - 3.0 * 0.000006 * t2
        }
        PrecessionModel::Iau2006 => {
            5_028.796_195 + 2.0 * 1.105_434_8 * t + 3.0 * 0.000_079_64 * t2
                - 4.0 * 0.000_023_857 * t3
                - 5.0 * 0.000_000_038_3 * t4
        }
        PrecessionModel::Vondrak2011 => {
            let mut r = 5_043.052_003_5 - 2.0 * 0.007_107_33 * t + 3.0 * 0.000_000_271 * t2;
            for term in VON_TABLE3_TERMS {
                let w = TAU / term.period_centuries;
                let (s, c) = vondrak2011_periodic_argument_rad(t, term.period_centuries).sin_cos();
                r += -term.cp * w * s + term.sp * w * c;
            }
            r
        }
    };
    rate / 3600.0 / 36525.0
}

/// Precess a 3-vector from J2000 ecliptic to ecliptic-of-date.
///
/// Applies the full ecliptic precession rotation matrix:
/// `P = R3(-(Π_A + p_A)) · R1(π_A) · R3(Π_A)`
///
/// `t` = Julian centuries of TDB since J2000.0.
/// Returns identity at T=0.
pub fn precess_ecliptic_j2000_to_date(v: &[f64; 3], t: f64) -> [f64; 3] {
    precess_ecliptic_j2000_to_date_with_model(v, t, DEFAULT_PRECESSION_MODEL)
}

/// Precess a 3-vector from J2000 ecliptic to ecliptic-of-date with a specific model.
pub fn precess_ecliptic_j2000_to_date_with_model(
    v: &[f64; 3],
    t: f64,
    model: PrecessionModel,
) -> [f64; 3] {
    if t.abs() < 1e-15 {
        return *v;
    }

    let pi_a = (ecliptic_inclination_arcsec_with_model(t, model) / 3600.0).to_radians();
    let cap_pi_a = (ecliptic_node_longitude_arcsec_with_model(t, model) / 3600.0).to_radians();
    let p_a = (general_precession_longitude_arcsec_with_model(t, model) / 3600.0).to_radians();

    // R3(Π_A) applied to v
    let (s1, c1) = cap_pi_a.sin_cos();
    let x1 = c1 * v[0] + s1 * v[1];
    let y1 = -s1 * v[0] + c1 * v[1];
    let z1 = v[2];

    // R1(π_A) applied
    let (s2, c2) = pi_a.sin_cos();
    let x2 = x1;
    let y2 = c2 * y1 + s2 * z1;
    let z2 = -s2 * y1 + c2 * z1;

    // R3(-(Π_A + p_A)) applied
    let (s3, c3) = (-(cap_pi_a + p_a)).sin_cos();
    [c3 * x2 + s3 * y2, -s3 * x2 + c3 * y2, z2]
}

/// Precess a 3-vector from ecliptic-of-date back to J2000 ecliptic.
///
/// Applies P^{-1} = P^T (P is orthogonal):
/// `P^{-1} = R3(-Π_A) · R1(-π_A) · R3(Π_A + p_A)`
///
/// Used for round-trip tests and any consumer that needs J2000 coordinates
/// from of-date input.
pub fn precess_ecliptic_date_to_j2000(v: &[f64; 3], t: f64) -> [f64; 3] {
    precess_ecliptic_date_to_j2000_with_model(v, t, DEFAULT_PRECESSION_MODEL)
}

/// Precess a 3-vector from ecliptic-of-date back to J2000 ecliptic with a specific model.
pub fn precess_ecliptic_date_to_j2000_with_model(
    v: &[f64; 3],
    t: f64,
    model: PrecessionModel,
) -> [f64; 3] {
    if t.abs() < 1e-15 {
        return *v;
    }

    let pi_a = (ecliptic_inclination_arcsec_with_model(t, model) / 3600.0).to_radians();
    let cap_pi_a = (ecliptic_node_longitude_arcsec_with_model(t, model) / 3600.0).to_radians();
    let p_a = (general_precession_longitude_arcsec_with_model(t, model) / 3600.0).to_radians();

    // R3(Π_A + p_A) applied to v
    let (s1, c1) = (cap_pi_a + p_a).sin_cos();
    let x1 = c1 * v[0] + s1 * v[1];
    let y1 = -s1 * v[0] + c1 * v[1];
    let z1 = v[2];

    // R1(-π_A) applied
    let (s2, c2) = (-pi_a).sin_cos();
    let x2 = x1;
    let y2 = c2 * y1 + s2 * z1;
    let z2 = -s2 * y1 + c2 * z1;

    // R3(-Π_A) applied
    let (s3, c3) = (-cap_pi_a).sin_cos();
    [c3 * x2 + s3 * y2, -s3 * x2 + c3 * y2, z2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_at_j2000() {
        assert_eq!(general_precession_longitude_arcsec(0.0), 0.0);
    }

    #[test]
    fn one_century_approx() {
        let p = general_precession_longitude_arcsec(1.0);
        // 5028.796195 + 1.1054348 + 0.00007964 - ... ≈ 5029.90
        assert!((p - 5029.90).abs() < 1.0, "p_A(1.0) = {p}");
    }

    #[test]
    fn negative_century() {
        let p = general_precession_longitude_arcsec(-1.0);
        assert!(p < 0.0, "p_A(-1.0) should be negative, got {p}");
    }

    #[test]
    fn rate_per_year() {
        // 1 year = 0.01 century
        let p = general_precession_longitude_arcsec(0.01);
        // ~50.29" per year
        assert!((p - 50.29).abs() < 0.1, "p_A(0.01) = {p}");
    }

    #[test]
    fn deg_conversion_consistent() {
        let t = 0.5;
        let arcsec = general_precession_longitude_arcsec(t);
        let deg = general_precession_longitude_deg(t);
        assert!((deg - arcsec / 3600.0).abs() < 1e-15);
    }

    // ---------- ecliptic inclination / node ----------

    #[test]
    fn ecliptic_inclination_zero_at_j2000() {
        // π_A(0) should be 0 by definition (no precession at J2000)
        assert_eq!(ecliptic_inclination_arcsec(0.0), 0.0);
    }

    #[test]
    fn ecliptic_inclination_one_century() {
        // Leading term: ~46.999"/century
        let pi = ecliptic_inclination_arcsec(1.0);
        assert!((pi - 47.0).abs() < 1.0, "π_A(1.0) = {pi}");
    }

    #[test]
    fn ecliptic_node_longitude_at_j2000() {
        let node_default = ecliptic_node_longitude_arcsec(0.0);
        assert!(node_default.is_finite(), "Π_A(default, 0) = {node_default}");

        // IAU 2006 retains the traditional Π_A(0) convention.
        let node_iau = ecliptic_node_longitude_arcsec_with_model(0.0, PrecessionModel::Iau2006);
        assert!(
            (node_iau - 629_546.793_6).abs() < 1e-6,
            "Π_A(IAU, 0) = {node_iau}"
        );
    }

    // ---------- precession matrix ----------

    #[test]
    fn precess_j2000_to_date_identity_at_t0() {
        let v = [1.0_f64, 0.5, -0.3];
        let out = precess_ecliptic_j2000_to_date(&v, 0.0);
        for i in 0..3 {
            assert!((out[i] - v[i]).abs() < 1e-15, "component {i}: {}", out[i]);
        }
    }

    #[test]
    fn precess_date_to_j2000_identity_at_t0() {
        let v = [1.0_f64, 0.5, -0.3];
        let out = precess_ecliptic_date_to_j2000(&v, 0.0);
        for i in 0..3 {
            assert!((out[i] - v[i]).abs() < 1e-15, "component {i}: {}", out[i]);
        }
    }

    #[test]
    fn precess_round_trip() {
        // forward then inverse must recover original to machine precision
        let v = [0.8_f64, 0.5, 0.1];
        for &t in &[0.5_f64, 1.0, -1.0, 5.0] {
            let fwd = precess_ecliptic_j2000_to_date(&v, t);
            let back = precess_ecliptic_date_to_j2000(&fwd, t);
            for i in 0..3 {
                assert!(
                    (back[i] - v[i]).abs() < 1e-12,
                    "t={t} component {i}: got {}, expected {}",
                    back[i],
                    v[i]
                );
            }
        }
    }

    #[test]
    fn precess_unit_length_preserved() {
        // Rotation must preserve vector length
        let v = [0.6_f64, 0.8, 0.0];
        let len_in = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        for &t in &[1.0_f64, -1.0, 5.0] {
            let out = precess_ecliptic_j2000_to_date(&v, t);
            let len_out = (out[0] * out[0] + out[1] * out[1] + out[2] * out[2]).sqrt();
            assert!(
                (len_out - len_in).abs() < 1e-13,
                "t={t}: |v|={len_in}, |Pv|={len_out}"
            );
        }
    }

    #[test]
    fn precess_j2000_x_axis_rotates_by_p_a() {
        // At T=1, (1,0,0) should rotate by ~p_A in longitude
        let v = [1.0_f64, 0.0, 0.0];
        let out = precess_ecliptic_j2000_to_date(&v, 1.0);
        let lon_in = 0.0_f64;
        let lon_out = out[1].atan2(out[0]).to_degrees();
        let p_a = general_precession_longitude_deg(1.0);
        let diff = (lon_out - lon_in - p_a).abs() % 360.0;
        // Allow 1° tolerance (higher-order terms from π_A, Π_A)
        assert!(
            diff.min(360.0 - diff) < 1.0,
            "lon shift={:.4}°, p_A={:.4}°",
            lon_out - lon_in,
            p_a
        );
    }

    #[test]
    fn default_wrappers_match_explicit_default_model() {
        let t = 0.75;
        let v = [0.2, -0.9, 0.38];
        assert!(
            (general_precession_longitude_arcsec(t)
                - general_precession_longitude_arcsec_with_model(t, DEFAULT_PRECESSION_MODEL))
            .abs()
                < 1e-15
        );
        assert!(
            (general_precession_longitude_deg(t)
                - general_precession_longitude_deg_with_model(t, DEFAULT_PRECESSION_MODEL))
            .abs()
                < 1e-15
        );
        assert!(
            (ecliptic_inclination_arcsec(t)
                - ecliptic_inclination_arcsec_with_model(t, DEFAULT_PRECESSION_MODEL))
            .abs()
                < 1e-15
        );
        assert!(
            (ecliptic_node_longitude_arcsec(t)
                - ecliptic_node_longitude_arcsec_with_model(t, DEFAULT_PRECESSION_MODEL))
            .abs()
                < 1e-15
        );
        let a = precess_ecliptic_j2000_to_date(&v, t);
        let b = precess_ecliptic_j2000_to_date_with_model(&v, t, DEFAULT_PRECESSION_MODEL);
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < 1e-15);
        }
    }

    #[test]
    fn vondrak_path_is_wired() {
        let t = 25.0;
        let v = [0.4, 0.5, -0.7];
        let p_iau = general_precession_longitude_arcsec_with_model(t, PrecessionModel::Iau2006);
        let p_vondrak =
            general_precession_longitude_arcsec_with_model(t, PrecessionModel::Vondrak2011);
        assert!((p_iau - p_vondrak).abs() > 1e-3);

        let out_iau = precess_ecliptic_j2000_to_date_with_model(&v, t, PrecessionModel::Iau2006);
        let out_vondrak =
            precess_ecliptic_j2000_to_date_with_model(&v, t, PrecessionModel::Vondrak2011);
        assert!((out_iau[0] - out_vondrak[0]).abs() > 1e-10);

        let fwd = precess_ecliptic_j2000_to_date_with_model(&v, t, PrecessionModel::Vondrak2011);
        let back = precess_ecliptic_date_to_j2000_with_model(&fwd, t, PrecessionModel::Vondrak2011);
        for i in 0..3 {
            assert!((back[i] - v[i]).abs() < 1e-12);
        }
    }

    #[test]
    fn vondrak_pq_tracks_iau_near_modern_epochs() {
        // Around modern epochs the long-term Vondrak series should closely
        // track IAU 2006 p/q components (sub-arcsecond scale).
        for &t in &[-1.0_f64, -0.6804, 0.26, 1.0] {
            let (p_v, q_v) = vondrak2011_pq_rad(t);
            let pi_i = (iau2006_ecliptic_inclination_arcsec(t) / 3600.0).to_radians();
            let cap_i = (iau2006_ecliptic_node_longitude_arcsec(t) / 3600.0).to_radians();
            let p_i = pi_i.sin() * cap_i.sin();
            let q_i = pi_i.sin() * cap_i.cos();

            let p_err_arcsec = ((p_v - p_i) / AS2R).abs();
            let q_err_arcsec = ((q_v - q_i) / AS2R).abs();
            assert!(p_err_arcsec < 2.0, "t={t}: |Δp|={p_err_arcsec}\"");
            assert!(q_err_arcsec < 0.1, "t={t}: |Δq|={q_err_arcsec}\"");
        }
    }
}
