//! Integration tests for engine-aware lunar node computation.
//!
//! Requires kernel files (de442s.bsp, naif0012.tls). Skips gracefully if absent.

use std::path::{Path, PathBuf};

use dhruv_core::{Engine, EngineConfig};
use dhruv_frames::PrecessionModel;
use dhruv_time::{LeapSecondKernel, UtcTime};
use dhruv_vedic_base::{
    LunarNode, NodeMode, jd_tdb_to_centuries, lunar_node_deg_for_epoch,
    lunar_node_deg_for_epoch_with_model,
};

const SPK_CANDIDATES: [&str; 2] = ["../../data/de442s.bsp", "../../kernels/data/de442s.bsp"];
const LSK_CANDIDATES: [&str; 2] = ["../../data/naif0012.tls", "../../kernels/data/naif0012.tls"];

fn first_existing(paths: &[&str]) -> Option<PathBuf> {
    paths
        .iter()
        .map(PathBuf::from)
        .find(|p| Path::new(p).exists())
}

fn load_test_resources() -> Option<(Engine, LeapSecondKernel)> {
    let spk = first_existing(&SPK_CANDIDATES)?;
    let lsk = first_existing(&LSK_CANDIDATES)?;

    let config = EngineConfig {
        spk_paths: vec![spk.clone()],
        lsk_path: lsk.clone(),
        cache_capacity: 1024,
        strict_validation: false,
    };
    let engine = Engine::new(config).ok()?;
    let lsk_kernel = LeapSecondKernel::load(&lsk).ok()?;
    Some((engine, lsk_kernel))
}

fn angular_separation_deg(a: f64, b: f64) -> f64 {
    let d = (a - b + 180.0).rem_euclid(360.0) - 180.0;
    d.abs()
}

#[test]
fn osculating_true_node_differs_from_mean_at_historical_epoch() {
    let Some((engine, lsk)) = load_test_resources() else {
        eprintln!("Skipping lunar_nodes_engine_golden: kernel files not found");
        return;
    };

    // Osho chart epoch used for node-model debugging.
    let utc = UtcTime::new(1931, 12, 11, 11, 38, 18.0);
    let jd_tdb = utc.to_jd_tdb(&lsk);
    let t = jd_tdb_to_centuries(jd_tdb);

    let mean_rahu = dhruv_vedic_base::lunar_node_deg(LunarNode::Rahu, t, NodeMode::Mean);
    let true_rahu = lunar_node_deg_for_epoch(&engine, LunarNode::Rahu, jd_tdb, NodeMode::True)
        .expect("osculating true node should compute");

    let diff = angular_separation_deg(true_rahu, mean_rahu);
    assert!(
        diff > 0.3,
        "expected osculating true node to differ noticeably from mean; got {diff:.6} deg"
    );
}

#[test]
fn osculating_rahu_ketu_remain_opposite() {
    let Some((engine, lsk)) = load_test_resources() else {
        eprintln!("Skipping lunar_nodes_engine_golden: kernel files not found");
        return;
    };

    let utc = UtcTime::new(1931, 12, 11, 11, 38, 18.0);
    let jd_tdb = utc.to_jd_tdb(&lsk);

    let rahu = lunar_node_deg_for_epoch(&engine, LunarNode::Rahu, jd_tdb, NodeMode::True)
        .expect("osculating rahu should compute");
    let ketu = lunar_node_deg_for_epoch(&engine, LunarNode::Ketu, jd_tdb, NodeMode::True)
        .expect("osculating ketu should compute");

    let diff = (ketu - rahu).rem_euclid(360.0);
    assert!(
        (diff - 180.0).abs() < 1e-10,
        "expected Ketu opposite Rahu, got {diff:.12} deg"
    );
}

#[test]
fn osculating_rahu_model_delta_small_at_osho_epoch() {
    let Some((engine, lsk)) = load_test_resources() else {
        eprintln!("Skipping lunar_nodes_engine_golden: kernel files not found");
        return;
    };

    let utc = UtcTime::new(1931, 12, 11, 11, 38, 18.0);
    let jd_tdb = utc.to_jd_tdb(&lsk);

    let rahu_iau = lunar_node_deg_for_epoch_with_model(
        &engine,
        LunarNode::Rahu,
        jd_tdb,
        NodeMode::True,
        PrecessionModel::Iau2006,
    )
    .expect("IAU true Rahu should compute");
    let rahu_vondrak = lunar_node_deg_for_epoch_with_model(
        &engine,
        LunarNode::Rahu,
        jd_tdb,
        NodeMode::True,
        PrecessionModel::Vondrak2011,
    )
    .expect("Vondrak true Rahu should compute");

    let delta_arcsec = angular_separation_deg(rahu_vondrak, rahu_iau) * 3600.0;
    assert!(
        delta_arcsec < 10.0,
        "expected Vondrak/IAU node delta < 10\", got {delta_arcsec:.3}\""
    );
}
