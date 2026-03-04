//! Integration tests for dhruv_rs context-first APIs (require kernels).

use std::path::PathBuf;

use dhruv_rs::*;

fn kernel_paths() -> (PathBuf, PathBuf) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../kernels/data");
    (base.join("de442s.bsp"), base.join("naif0012.tls"))
}

fn kernels_available() -> bool {
    let (spk, lsk) = kernel_paths();
    spk.exists() && lsk.exists()
}

fn make_context() -> Option<DhruvContext> {
    if !kernels_available() {
        eprintln!("Skipping: kernel files not found");
        return None;
    }
    let (spk, lsk) = kernel_paths();
    let config = EngineConfig::with_single_spk(spk, lsk, 256, true);
    Some(DhruvContext::new(config).expect("context init"))
}

#[test]
fn context_builds_with_engine() {
    if let Some(ctx) = make_context() {
        let _ = ctx.engine();
    }
}

#[test]
fn conjunction_next_runs() {
    let Some(ctx) = make_context() else {
        return;
    };

    let req = ConjunctionRequest {
        body1: Body::Sun,
        body2: Body::Mercury,
        config: Some(ConjunctionConfig::conjunction(1.0)),
        query: ConjunctionRequestQuery::Next {
            at: TimeInput::Utc(UtcDate::new(2024, 3, 20, 0, 0, 0.0)),
        },
    };

    let out = conjunction(&ctx, &req).expect("conjunction op should run");
    match out {
        ConjunctionResult::Single(_) => {}
        _ => panic!("expected single conjunction result"),
    }
}

#[test]
fn sankranti_range_runs() {
    let Some(ctx) = make_context() else {
        return;
    };

    let req = SankrantiRequest {
        target: SankrantiTarget::Any,
        config: Some(SankrantiConfig::default_lahiri()),
        query: SankrantiRequestQuery::Range {
            start: TimeInput::Utc(UtcDate::new(2024, 1, 1, 0, 0, 0.0)),
            end: TimeInput::Utc(UtcDate::new(2024, 12, 31, 0, 0, 0.0)),
        },
    };

    let out = sankranti(&ctx, &req).expect("sankranti op should run");
    match out {
        SankrantiResult::Many(v) => assert!(!v.is_empty()),
        _ => panic!("expected many sankranti results"),
    }
}

#[test]
fn context_time_policy_roundtrip() {
    let Some(mut ctx) = make_context() else {
        return;
    };

    let p = TimeConversionPolicy::StrictLsk;
    ctx.set_time_conversion_policy(p);
    assert_eq!(ctx.time_conversion_policy(), p);
}
