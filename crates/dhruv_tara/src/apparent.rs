//! Apparent-place corrections: annual aberration and gravitational light deflection.
//!
//! Sources:
//! - Annual aberration: first-order relativistic formula (Seidelmann 1992).
//! - Light deflection: PPN formula, SOFA iauLd convention.

/// Speed of light in AU/day. c = 299792.458 km/s, 1 AU = 149597870.7 km.
/// c_au_day = 299792.458 * 86400 / 149597870.7
const C_AU_DAY: f64 = 173.144_632_720_536_34;

/// 2 × GM☉ / c² in AU.
/// GM☉ = 1.32712440018e20 m³/s² = 1.32712440018e11 km³/s²
/// c = 299792.458 km/s
/// 2GM/c² = 2 × 1.32712440018e11 / (299792.458² ) = 2.953e-3 AU? No, let's compute properly.
/// 2GM/(c² in AU): 2 × 1.32712440018e20 / (299792458² × 149597870700) = 9.8706e-6 AU
///
/// Actually, let's use the Schwarzschild radius of the Sun in AU:
/// 2GM☉/c² = 2 × 1.32712440018e20 / (2.99792458e8)² = 2.953250077e3 m = 2.953250077e-3 km
/// In AU: 2.953250077e-3 / 149597870.7 = 1.97412574e-11 AU
///
/// But for the deflection formula we need 2GM/(c²·D) where D is in AU:
/// Factor = 2GM/(c²) in AU = 1.97412574e-11 AU... that's tiny.
///
/// Wait, let me reconsider. The deflection angle formula is:
/// |Δs| = (2GM/(c²D)) × geometric_factor
/// where D is Sun-observer distance.
///
/// 2GM/c² = 2 × 1.32712440018e20 m³/s² / (299792458 m/s)² = 2.953250077e3 m
/// Convert to AU: 2.953250077e3 / 1.495978707e11 = 1.97412574e-8 AU
///
/// Hmm, let me just use a well-known value. The maximum deflection at the solar limb
/// (D = 1 AU, sin(χ)/(1-cos(χ)) → ∞ at χ→0; at R☉ separation, the deflection is ~1.75").
///
/// The factor 2GM/(c²) in AU = the Schwarzschild radius / AU.
/// R_S = 2GM/c² = 2.953250077 km = 1.97412574e-8 AU
const TWO_GM_OVER_C2_AU: f64 = 1.974_125_74e-8;

/// Apply annual aberration correction to a direction vector.
///
/// Uses the first-order relativistic formula:
/// s' = s + (1/c)(v − (s·v)s)
/// then normalize to unit vector.
///
/// # Arguments
/// * `direction` — unit direction vector to star (ICRS)
/// * `earth_vel_au_day` — ICRS barycentric velocity of Earth (AU/day)
///
/// # Returns
/// Corrected unit direction vector.
pub fn apply_aberration(direction: &[f64; 3], earth_vel_au_day: &[f64; 3]) -> [f64; 3] {
    let s = direction;
    let v = earth_vel_au_day;

    // s · v
    let s_dot_v = s[0] * v[0] + s[1] * v[1] + s[2] * v[2];

    // s' = s + (1/c)(v - (s·v)s)
    let inv_c = 1.0 / C_AU_DAY;
    let sp = [
        s[0] + inv_c * (v[0] - s_dot_v * s[0]),
        s[1] + inv_c * (v[1] - s_dot_v * s[1]),
        s[2] + inv_c * (v[2] - s_dot_v * s[2]),
    ];

    // Normalize
    let r = (sp[0] * sp[0] + sp[1] * sp[1] + sp[2] * sp[2]).sqrt();
    if r == 0.0 {
        return *direction;
    }
    [sp[0] / r, sp[1] / r, sp[2] / r]
}

/// Apply gravitational light deflection by the Sun (SOFA iauLd convention).
///
/// Uses the PPN formula:
/// Δs = (2GM/(c²D)) × [(s·e)s − e] / (1 + s·e)
///
/// where:
/// - s = unit direction to star
/// - e = unit vector from Sun to observer
/// - D = Sun-observer distance (AU)
///
/// # Arguments
/// * `direction` — unit direction vector to star (ICRS)
/// * `e_sun_to_obs` — unit vector from Sun to observer
/// * `d_sun_au` — Sun-observer distance in AU
///
/// # Returns
/// Corrected unit direction vector.
pub fn apply_light_deflection(
    direction: &[f64; 3],
    e_sun_to_obs: &[f64; 3],
    d_sun_au: f64,
) -> [f64; 3] {
    let s = direction;
    let e = e_sun_to_obs;

    // s · e
    let s_dot_e = s[0] * e[0] + s[1] * e[1] + s[2] * e[2];

    // Denominator: 1 + s·e
    // When s·e → -1 (star directly behind Sun), deflection → ∞
    // but such stars are unobservable. Clamp to avoid division by zero.
    let denom = 1.0 + s_dot_e;
    if denom < 1e-10 {
        return *direction; // Star too close to Sun direction, skip deflection
    }

    // Deflection factor
    let factor = TWO_GM_OVER_C2_AU / d_sun_au;

    // Δs = factor × [(s·e)s − e] / (1 + s·e)
    let ds = [
        factor * (s_dot_e * s[0] - e[0]) / denom,
        factor * (s_dot_e * s[1] - e[1]) / denom,
        factor * (s_dot_e * s[2] - e[2]) / denom,
    ];

    // s' = s + Δs, normalize
    let sp = [s[0] + ds[0], s[1] + ds[1], s[2] + ds[2]];
    let r = (sp[0] * sp[0] + sp[1] * sp[1] + sp[2] * sp[2]).sqrt();
    if r == 0.0 {
        return *direction;
    }
    [sp[0] / r, sp[1] / r, sp[2] / r]
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARCSEC_TO_RAD: f64 = std::f64::consts::PI / 648_000.0;
    const MAS_TO_RAD: f64 = ARCSEC_TO_RAD / 1000.0;

    /// Angular separation between two unit vectors (radians).
    fn angular_sep(a: &[f64; 3], b: &[f64; 3]) -> f64 {
        let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
        dot.acos()
    }

    #[test]
    fn aberration_magnitude() {
        // Star at ecliptic pole (0, 0, 1), Earth moving in +Y at ~29.8 km/s
        // v ≈ 0.01721 AU/day (Earth's orbital speed)
        let star = [0.0, 0.0, 1.0];
        let v_earth = [0.0, 0.017_21, 0.0]; // ~29.8 km/s

        let corrected = apply_aberration(&star, &v_earth);
        let shift = angular_sep(&star, &corrected);

        // Expected: ~20.5" (aberration constant κ = v/c ≈ 20.4955")
        let expected = 20.5 * ARCSEC_TO_RAD;
        assert!(
            (shift - expected).abs() < 0.1 * ARCSEC_TO_RAD,
            "aberration shift: {:.2}\" (expected ~20.5\")",
            shift / ARCSEC_TO_RAD
        );
    }

    #[test]
    fn aberration_zero_velocity() {
        let star = [1.0, 0.0, 0.0];
        let v_earth = [0.0, 0.0, 0.0];
        let corrected = apply_aberration(&star, &v_earth);
        let shift = angular_sep(&star, &corrected);
        assert!(shift < 1e-15, "shift should be zero for zero velocity");
    }

    #[test]
    fn deflection_at_90_degrees() {
        // Star perpendicular to Sun direction
        // e = unit from Sun to observer; observer is at 1 AU from Sun along +X
        // So Sun is at origin, observer at (1,0,0), e = (1,0,0)
        let star = [0.0, 1.0, 0.0]; // 90° from Sun
        let e = [1.0, 0.0, 0.0]; // Sun→observer
        let d = 1.0; // 1 AU

        let corrected = apply_light_deflection(&star, &e, d);
        let shift = angular_sep(&star, &corrected);

        // Expected at 90°: 2GM/(c²D) × sin(90°)/(1 - cos(90°))
        // = 1.974e-8 × 1/1 = 1.974e-8 rad ≈ 4.07 mas
        let expected_mas = TWO_GM_OVER_C2_AU / d / MAS_TO_RAD;
        let actual_mas = shift / MAS_TO_RAD;
        assert!(
            (actual_mas - expected_mas).abs() < 0.5,
            "deflection at 90°: {actual_mas:.2} mas (expected {expected_mas:.2} mas)"
        );
    }

    #[test]
    fn deflection_at_45_degrees() {
        // Star at 45° from the anti-Sun direction
        // s · e = -cos(45°) = -0.7071... wait, let me think.
        // χ is the angle between star and Sun as seen from observer.
        // In our convention, e points from Sun to observer.
        // s · e = cos(angle between star direction and Sun-to-observer vector)
        // If χ = angle(star, Sun_direction), and Sun_direction = -e,
        // then s · (-e) = cos(χ) → s · e = -cos(χ)
        //
        // For χ = 45°: s · e = -cos(45°) = -0.7071
        // Expected: 2GM/(c²D) × sin(45°)/(1 - cos(45°))
        //         = 1.974e-8 × 0.7071/(1-0.7071) = 1.974e-8 × 2.414
        //         ≈ 4.76e-8 rad ≈ 9.8 mas

        // Construct star at 45° from Sun (Sun is along -X from observer)
        let cos45 = std::f64::consts::FRAC_1_SQRT_2;
        let sin45 = std::f64::consts::FRAC_1_SQRT_2;
        // Sun is at -X, so star at 45° from Sun is between -X and +Y:
        // star = (-cos45, sin45, 0) → s·e = -cos45 (since e = (1,0,0))
        let star = [-cos45, sin45, 0.0];
        let e = [1.0, 0.0, 0.0];
        let d = 1.0;

        let corrected = apply_light_deflection(&star, &e, d);
        let shift = angular_sep(&star, &corrected);

        // Expected: factor × sin(45°)/(1 - cos(45°))
        let factor_rad = TWO_GM_OVER_C2_AU / d;
        let expected_rad = factor_rad * sin45 / (1.0 - cos45);
        let expected_mas = expected_rad / MAS_TO_RAD;
        let actual_mas = shift / MAS_TO_RAD;

        assert!(
            (actual_mas - expected_mas).abs() < 1.0,
            "deflection at 45°: {actual_mas:.2} mas (expected {expected_mas:.2} mas)"
        );
    }

    #[test]
    fn deflection_at_anti_sun() {
        // Star directly opposite Sun (χ = 180°): s · e = cos(0°) = 1
        // → sin(180°)/(1-cos(180°)) = 0/2 = 0 → no deflection
        let star = [1.0, 0.0, 0.0]; // same direction as e
        let e = [1.0, 0.0, 0.0];
        let d = 1.0;

        let corrected = apply_light_deflection(&star, &e, d);
        let shift = angular_sep(&star, &corrected);
        assert!(shift < 1e-12, "deflection at anti-Sun should be zero");
    }

    #[test]
    fn deflection_zero_at_infinity() {
        // At very large distance, deflection → 0
        let star = [0.0, 1.0, 0.0];
        let e = [1.0, 0.0, 0.0];
        let d = 1e10; // very far

        let corrected = apply_light_deflection(&star, &e, d);
        let shift = angular_sep(&star, &corrected);
        assert!(shift < 1e-15);
    }
}
