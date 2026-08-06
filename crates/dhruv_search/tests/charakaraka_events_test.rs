//! Integration tests for `charakaraka_events`.
//!
//! Requires kernel files. Skips gracefully if absent.

use std::path::Path;

use dhruv_core::{Body, Engine, EngineConfig};
use dhruv_search::sankranti_types::SankrantiConfig;
use dhruv_search::{
    CharakarakaChangeEvent, CharakarakaEventTrigger, SearchError, TransitBody, charakaraka_events,
    charakaraka_for_date, next_charakaraka_event, next_ingress, prev_charakaraka_event,
};
use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::{CharakarakaResult, CharakarakaScheme, Graha, NodeMode};

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";
const EOP_PATH: &str = "../../kernels/data/finals2000A.all";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping charakaraka_events_test: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 1024, false);
    Engine::new(config).ok()
}

fn load_eop() -> Option<EopKernel> {
    if !Path::new(EOP_PATH).exists() {
        return None;
    }
    EopKernel::load(Path::new(EOP_PATH)).ok()
}

fn aya() -> SankrantiConfig {
    SankrantiConfig::default_lahiri()
}

fn ranking_signature(result: &CharakarakaResult) -> Vec<(u8, u8)> {
    result
        .entries
        .iter()
        .map(|e| (e.role.code(), e.graha.index()))
        .collect()
}

fn assert_snapshot_matches_per_moment(
    engine: &Engine,
    eop: &EopKernel,
    config: &SankrantiConfig,
    scheme: CharakarakaScheme,
    event: &CharakarakaChangeEvent,
) {
    let probe = 5e-6;
    let before_utc = UtcTime::from_jd_tdb(event.jd_tdb - probe, engine.lsk());
    let after_utc = UtcTime::from_jd_tdb(event.jd_tdb + probe, engine.lsk());
    let before =
        charakaraka_for_date(engine, eop, &before_utc, config, scheme).expect("per-moment before");
    let after =
        charakaraka_for_date(engine, eop, &after_utc, config, scheme).expect("per-moment after");
    assert_eq!(
        ranking_signature(&before),
        ranking_signature(&event.before),
        "before snapshot disagrees with per-moment op at {:?}",
        before_utc
    );
    assert_eq!(
        ranking_signature(&after),
        ranking_signature(&event.after),
        "after snapshot disagrees with per-moment op at {:?}",
        after_utc
    );
    assert_eq!(before.used_eight_karakas, event.before.used_eight_karakas);
    assert_eq!(after.used_eight_karakas, event.after.used_eight_karakas);
}

/// Brute-force cross-validation: every ranking change seen by dense
/// sampling must be bracketed by at least one event, the event chain must
/// be gapless, and event snapshots must agree with the per-moment op.
#[test]
fn eight_scheme_brute_force_cross_validation() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = aya();
    let scheme = CharakarakaScheme::Eight;
    let from = UtcTime::new(2024, 1, 1, 0, 0, 0.0);
    let to = UtcTime::new(2024, 2, 10, 0, 0, 0.0);

    let result =
        charakaraka_events(&engine, &eop, &from, &to, &config, scheme, 0).expect("range sweep");
    assert!(!result.truncated);
    assert!(
        result.events.len() > 100,
        "40 days of EIGHT should be dense (~4/day), got {}",
        result.events.len()
    );

    // Events strictly ascending and inside the window.
    let from_jd = from.to_jd_tdb(engine.lsk());
    let to_jd = to.to_jd_tdb(engine.lsk());
    for pair in result.events.windows(2) {
        assert!(pair[0].jd_tdb < pair[1].jd_tdb, "events must be ascending");
    }
    for event in &result.events {
        assert!(event.jd_tdb > from_jd && event.jd_tdb <= to_jd + 1e-6);
        assert!(!event.changed_roles.is_empty());
        assert_ne!(
            ranking_signature(&event.before),
            ranking_signature(&event.after),
            "emitted event must be an actual change"
        );
    }

    // Chain invariant: previous.after == next.before.
    for pair in result.events.windows(2) {
        assert_eq!(
            ranking_signature(&pair[0].after),
            ranking_signature(&pair[1].before),
            "chain broken between {:?} and {:?}",
            pair[0].utc,
            pair[1].utc
        );
    }

    // Dense sampling: any change between consecutive samples must have at
    // least one event inside that interval.
    let step = 15.0 / 1440.0;
    let mut t_prev = from_jd;
    let mut sig_prev = {
        let utc = UtcTime::from_jd_tdb(t_prev, engine.lsk());
        ranking_signature(
            &charakaraka_for_date(&engine, &eop, &utc, &config, scheme).expect("sample"),
        )
    };
    let mut checked_intervals = 0usize;
    while t_prev < to_jd {
        let t_curr = (t_prev + step).min(to_jd);
        let utc = UtcTime::from_jd_tdb(t_curr, engine.lsk());
        let sig_curr = ranking_signature(
            &charakaraka_for_date(&engine, &eop, &utc, &config, scheme).expect("sample"),
        );
        if sig_prev != sig_curr {
            let found = result
                .events
                .iter()
                .any(|e| e.jd_tdb > t_prev && e.jd_tdb <= t_curr + 1e-9);
            assert!(
                found,
                "sampled ranking change in ({t_prev}, {t_curr}] has no event"
            );
            checked_intervals += 1;
        }
        t_prev = t_curr;
        sig_prev = sig_curr;
    }
    assert!(checked_intervals > 50, "sampling should see many changes");

    // Per-moment parity for a spread of events.
    for event in result.events.iter().step_by(23) {
        assert_snapshot_matches_per_moment(&engine, &eop, &config, scheme, event);
    }
}

/// A Rahu-involved degree crossing satisfies the sum condition
/// `d_Rahu + d_other = 30` — the boundary family `conjunction` cannot
/// express.
#[test]
fn rahu_sum_crossing_has_sum_thirty() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = aya();
    let from = UtcTime::new(2024, 3, 1, 0, 0, 0.0);
    let to = UtcTime::new(2024, 3, 15, 0, 0, 0.0);

    let result = charakaraka_events(
        &engine,
        &eop,
        &from,
        &to,
        &config,
        CharakarakaScheme::Eight,
        0,
    )
    .expect("range sweep");

    let mut found = false;
    for event in &result.events {
        if event.trigger != CharakarakaEventTrigger::DegreeCrossing {
            continue;
        }
        let rahu_moved = event.changed_roles.iter().any(|&role| {
            let b = event.before.entries.iter().find(|e| e.role == role);
            let a = event.after.entries.iter().find(|e| e.role == role);
            b.map(|e| e.graha) == Some(Graha::Rahu) || a.map(|e| e.graha) == Some(Graha::Rahu)
        });
        if !rahu_moved {
            continue;
        }
        let rahu = event
            .after
            .entries
            .iter()
            .find(|e| e.graha == Graha::Rahu)
            .expect("rahu ranked in EIGHT");
        // Some classical body must satisfy the sum condition at the root.
        let sum_hit = event
            .after
            .entries
            .iter()
            .filter(|e| e.graha != Graha::Rahu)
            .any(|other| {
                let s = (rahu.degrees_in_rashi + other.degrees_in_rashi).rem_euclid(30.0);
                let near_zero = s < 0.01 || s > 29.99;
                let both_zero = rahu.degrees_in_rashi < 0.01 && other.degrees_in_rashi < 0.01;
                near_zero && !both_zero
            });
        if sum_hit {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "two weeks of EIGHT should contain a Rahu sum crossing"
    );
}

/// An ingress-triggered event coincides with the sankranti op's ingress.
#[test]
fn chandra_ingress_event_matches_sankranti_op() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = aya();
    let from = UtcTime::new(2024, 5, 1, 0, 0, 0.0);
    let to = UtcTime::new(2024, 5, 6, 0, 0, 0.0);

    let result = charakaraka_events(
        &engine,
        &eop,
        &from,
        &to,
        &config,
        CharakarakaScheme::Eight,
        0,
    )
    .expect("range sweep");

    // Find an ingress event where Chandra's rashi changed.
    let chandra_rashi = |result: &CharakarakaResult| -> u8 {
        let e = result
            .entries
            .iter()
            .find(|e| e.graha == Graha::Chandra)
            .expect("chandra ranked");
        (e.longitude_deg / 30.0).floor() as u8
    };
    let event = result
        .events
        .iter()
        .find(|e| {
            e.trigger == CharakarakaEventTrigger::RashiIngress
                && chandra_rashi(&e.before) != chandra_rashi(&e.after)
        })
        .expect("5 days must contain a Chandra ingress event");

    let mut ingress_config = config;
    ingress_config.step_size_days = 0.25;
    let ingress = next_ingress(
        &engine,
        TransitBody::Body(Body::Moon),
        &from,
        &ingress_config,
    )
    .expect("ingress search")
    .expect("ingress found");
    let ingress_jd = ingress.utc.to_jd_tdb(engine.lsk());

    // Same boundary: the first Chandra ingress in the window. The two ops
    // use slightly different UTC->TDB paths (EOP vs none), so allow ~9 s.
    assert!(
        (event.jd_tdb - ingress_jd).abs() < 1e-4,
        "charakaraka ingress at {} vs sankranti ingress at {}",
        event.jd_tdb,
        ingress_jd
    );
}

/// MIXED_PARASHARA emits scheme_mode_change events at integer-degree bin
/// boundaries, with used_eight_karakas flipping and per-moment agreement.
#[test]
fn mixed_parashara_mode_toggles() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = aya();
    let scheme = CharakarakaScheme::MixedParashara;
    let from = UtcTime::new(2024, 1, 1, 0, 0, 0.0);
    let to = UtcTime::new(2024, 1, 8, 0, 0, 0.0);

    let result =
        charakaraka_events(&engine, &eop, &from, &to, &config, scheme, 0).expect("range sweep");

    let toggles: Vec<&CharakarakaChangeEvent> = result
        .events
        .iter()
        .filter(|e| e.trigger == CharakarakaEventTrigger::SchemeModeChange)
        .collect();
    assert!(
        !toggles.is_empty(),
        "a week of MIXED should contain mode toggles (measured ~3.4/day in 2024)"
    );
    for event in &toggles {
        assert_ne!(
            event.before.used_eight_karakas, event.after.used_eight_karakas,
            "scheme_mode_change must flip used_eight_karakas"
        );
        // Entry counts flip between 7 and 8 with the mode.
        assert_eq!(
            event.before.entries.len(),
            if event.before.used_eight_karakas {
                8
            } else {
                7
            }
        );
        assert_eq!(
            event.after.entries.len(),
            if event.after.used_eight_karakas { 8 } else { 7 }
        );
    }
    // Non-toggle events must keep the mode.
    for event in &result.events {
        if event.trigger != CharakarakaEventTrigger::SchemeModeChange {
            assert_eq!(
                event.before.used_eight_karakas,
                event.after.used_eight_karakas
            );
        }
    }
    // Per-moment parity on toggles.
    for event in toggles.iter().step_by(3) {
        assert_snapshot_matches_per_moment(&engine, &eop, &config, scheme, event);
    }
    // Chain invariant holds across mode flips too.
    for pair in result.events.windows(2) {
        assert_eq!(
            ranking_signature(&pair[0].after),
            ranking_signature(&pair[1].before)
        );
    }
}

/// Truncation + resume reconstructs the uncapped stream.
#[test]
fn continuation_resumes_without_loss() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = aya();
    let scheme = CharakarakaScheme::Eight;
    let from = UtcTime::new(2024, 7, 1, 0, 0, 0.0);
    let to = UtcTime::new(2024, 7, 11, 0, 0, 0.0);

    let full =
        charakaraka_events(&engine, &eop, &from, &to, &config, scheme, 0).expect("uncapped sweep");
    assert!(!full.truncated);
    assert!(full.events.len() > 20);

    let mut collected: Vec<CharakarakaChangeEvent> = Vec::new();
    let mut cursor = from;
    for _ in 0..100 {
        let part = charakaraka_events(&engine, &eop, &cursor, &to, &config, scheme, 7)
            .expect("capped sweep");
        for event in part.events {
            let dup = collected
                .last()
                .map(|last: &CharakarakaChangeEvent| (event.jd_tdb - last.jd_tdb).abs() < 1e-6)
                .unwrap_or(false);
            if !dup {
                collected.push(event);
            }
        }
        if !part.truncated {
            break;
        }
        cursor = part.next_from_utc.expect("resume point when truncated");
    }

    assert_eq!(
        collected.len(),
        full.events.len(),
        "resumed chunks must reconstruct the uncapped stream"
    );
    for (a, b) in collected.iter().zip(full.events.iter()) {
        assert!(
            (a.jd_tdb - b.jd_tdb).abs() < 1e-6,
            "event times must match: {} vs {}",
            a.jd_tdb,
            b.jd_tdb
        );
        assert_eq!(ranking_signature(&a.after), ranking_signature(&b.after));
    }
}

/// next/prev agree with the range sweep's first/last events.
#[test]
fn next_prev_match_range_edges() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let config = aya();
    let scheme = CharakarakaScheme::SevenPkMergedMk;
    let from = UtcTime::new(2024, 9, 1, 0, 0, 0.0);
    let to = UtcTime::new(2024, 9, 4, 0, 0, 0.0);

    let range =
        charakaraka_events(&engine, &eop, &from, &to, &config, scheme, 0).expect("range sweep");
    assert!(!range.events.is_empty());

    let next = next_charakaraka_event(&engine, &eop, &from, &config, scheme)
        .expect("next search")
        .expect("next event exists");
    assert!(
        (next.jd_tdb - range.events[0].jd_tdb).abs() < 1e-6,
        "next {} vs first range event {}",
        next.jd_tdb,
        range.events[0].jd_tdb
    );

    let prev = prev_charakaraka_event(&engine, &eop, &to, &config, scheme)
        .expect("prev search")
        .expect("prev event exists");
    let last = range.events.last().unwrap();
    assert!(
        (prev.jd_tdb - last.jd_tdb).abs() < 1e-6,
        "prev {} vs last range event {}",
        prev.jd_tdb,
        last.jd_tdb
    );
}

/// The op honors node_mode: parity with the per-moment op under the mean
/// node, and mean vs true Rahu longitudes actually differ.
#[test]
fn node_mode_honored_and_diverges() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let scheme = CharakarakaScheme::Eight;
    let mut mean_config = aya();
    mean_config.node_mode = NodeMode::Mean;
    let from = UtcTime::new(2024, 4, 1, 0, 0, 0.0);
    let to = UtcTime::new(2024, 4, 8, 0, 0, 0.0);

    let mean_run = charakaraka_events(&engine, &eop, &from, &to, &mean_config, scheme, 0)
        .expect("mean-node sweep");
    assert!(!mean_run.events.is_empty());
    for event in mean_run.events.iter().step_by(11) {
        assert_snapshot_matches_per_moment(&engine, &eop, &mean_config, scheme, event);
    }

    let true_run =
        charakaraka_events(&engine, &eop, &from, &to, &aya(), scheme, 0).expect("true-node sweep");
    let rahu_lon = |events: &[CharakarakaChangeEvent]| -> f64 {
        events[0]
            .after
            .entries
            .iter()
            .find(|e| e.graha == Graha::Rahu)
            .expect("rahu ranked")
            .longitude_deg
    };
    assert!(
        (rahu_lon(&mean_run.events) - rahu_lon(&true_run.events)).abs() > 1e-3,
        "mean and true node Rahu longitudes should differ"
    );
}

/// Input validation.
#[test]
fn validation_rejects_bad_input() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let from = UtcTime::new(2024, 1, 2, 0, 0, 0.0);
    let to = UtcTime::new(2024, 1, 1, 0, 0, 0.0);
    let err = charakaraka_events(
        &engine,
        &eop,
        &from,
        &to,
        &aya(),
        CharakarakaScheme::Eight,
        0,
    )
    .unwrap_err();
    assert!(matches!(err, SearchError::InvalidConfig(_)));

    let mut bad = aya();
    bad.convergence_days = 0.0;
    let err = charakaraka_events(&engine, &eop, &to, &from, &bad, CharakarakaScheme::Eight, 0)
        .unwrap_err();
    assert!(matches!(err, SearchError::InvalidConfig(_)));
}
