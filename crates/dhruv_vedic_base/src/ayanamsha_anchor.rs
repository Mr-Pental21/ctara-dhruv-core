//! Star-anchored ayanamsha helpers.
//!
//! These models compute ayanamsha from an anchor point that should stay at a
//! fixed sidereal longitude, instead of using a fixed J2000 offset.

use crate::ayanamsha::AyanamshaSystem;
use crate::util::normalize_360;
use dhruv_frames::{
    PrecessionModel, cartesian_to_spherical, precess_ecliptic_j2000_to_date_with_model,
};

#[derive(Debug, Clone, Copy)]
struct AnchorSpec {
    /// Anchor longitude in J2000 ecliptic degrees.
    lon_j2000_deg: f64,
    /// Anchor latitude in J2000 ecliptic degrees.
    lat_j2000_deg: f64,
    /// Target sidereal longitude that the anchor should keep.
    target_sidereal_lon_deg: f64,
}

fn anchor_spec(system: AyanamshaSystem) -> Option<AnchorSpec> {
    match system {
        // Spica anchor. Calibrated to existing J2000 Lahiri baseline.
        AyanamshaSystem::TrueLahiri => Some(AnchorSpec {
            lon_j2000_deg: 203.853_000,
            lat_j2000_deg: -2.054_489,
            target_sidereal_lon_deg: 180.0,
        }),
        // Pushya anchor. The legacy model defines this as 106° sidereal.
        AyanamshaSystem::PushyaPaksha => Some(AnchorSpec {
            lon_j2000_deg: 127.0,
            lat_j2000_deg: 0.0,
            target_sidereal_lon_deg: 106.0,
        }),
        // Aldebaran anchor at 15°47' Taurus.
        AyanamshaSystem::RohiniPaksha => Some(AnchorSpec {
            lon_j2000_deg: 69.870_333,
            lat_j2000_deg: -5.467_327,
            target_sidereal_lon_deg: 45.783_333,
        }),
        // Aldebaran anchor at 15° Taurus.
        AyanamshaSystem::Aldebaran15Tau => Some(AnchorSpec {
            lon_j2000_deg: 69.870_000,
            lat_j2000_deg: -5.467_327,
            target_sidereal_lon_deg: 45.0,
        }),
        _ => None,
    }
}

fn anchor_tropical_longitude_deg(
    spec: AnchorSpec,
    t_centuries: f64,
    model: PrecessionModel,
) -> f64 {
    let lon = spec.lon_j2000_deg.to_radians();
    let lat = spec.lat_j2000_deg.to_radians();
    let v = [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()];
    let v_date = precess_ecliptic_j2000_to_date_with_model(&v, t_centuries, model);
    cartesian_to_spherical(&v_date).lon_deg
}

/// Star-relative ayanamsha for systems that are defined by anchor locking.
pub(crate) fn anchor_relative_ayanamsha_deg(
    system: AyanamshaSystem,
    t_centuries: f64,
    model: PrecessionModel,
) -> Option<f64> {
    let spec = anchor_spec(system)?;
    let anchor_lon = anchor_tropical_longitude_deg(spec, t_centuries, model);
    Some(normalize_360(anchor_lon - spec.target_sidereal_lon_deg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converted_systems_are_anchor_relative() {
        for &sys in &[
            AyanamshaSystem::TrueLahiri,
            AyanamshaSystem::PushyaPaksha,
            AyanamshaSystem::RohiniPaksha,
            AyanamshaSystem::Aldebaran15Tau,
        ] {
            assert!(
                anchor_spec(sys).is_some(),
                "{sys:?} should have anchor spec"
            );
        }
    }

    #[test]
    fn anchor_lock_invariant_true_lahiri() {
        let spec = anchor_spec(AyanamshaSystem::TrueLahiri).unwrap();
        for t in [-2.0, -1.0, 0.0, 0.5, 1.0, 2.0] {
            let aya = anchor_relative_ayanamsha_deg(
                AyanamshaSystem::TrueLahiri,
                t,
                PrecessionModel::Iau2006,
            )
            .unwrap();
            let anchor_lon = anchor_tropical_longitude_deg(spec, t, PrecessionModel::Iau2006);
            let sid = normalize_360(anchor_lon - aya);
            assert!(
                (sid - spec.target_sidereal_lon_deg).abs() < 1e-9,
                "t={t}: sid={sid}"
            );
        }
    }

    #[test]
    fn model_parameter_is_wired() {
        let t = 25.0;
        let a =
            anchor_relative_ayanamsha_deg(AyanamshaSystem::TrueLahiri, t, PrecessionModel::Iau2006)
                .unwrap();
        let b = anchor_relative_ayanamsha_deg(
            AyanamshaSystem::TrueLahiri,
            t,
            PrecessionModel::Vondrak2011,
        )
        .unwrap();
        assert!((a - b).abs() > 1e-6);
    }
}
