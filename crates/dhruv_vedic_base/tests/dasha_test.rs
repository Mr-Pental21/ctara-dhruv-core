//! Integration tests for dasha pure-math calculations.
//!
//! These tests verify the pure-math dasha engine without requiring kernel files.

use dhruv_vedic_base::dasha::{
    DashaEntity, DashaLevel, DashaSystem, DashaVariationConfig, MAX_DASHA_LEVEL,
    nakshatra_hierarchy, nakshatra_level0, nakshatra_snapshot, snapshot_from_hierarchy,
    vimshottari_config,
};
use dhruv_vedic_base::Graha;

/// Moon at 0° Aries (Ashwini nakshatra, index 0) → Ketu mahadasha, full 7y.
#[test]
fn vimshottari_moon_at_zero() {
    let cfg = vimshottari_config();
    let birth_jd = 2451545.0; // J2000
    let moon_lon = 0.0; // 0° = Ashwini, pad 1, start of nakshatra

    let level0 = nakshatra_level0(birth_jd, moon_lon, &cfg);
    assert_eq!(level0.len(), 9);

    // First period should be Ketu (Ashwini → Ketu in Vimshottari)
    assert_eq!(level0[0].entity, DashaEntity::Graha(Graha::Ketu));
    assert_eq!(level0[0].level, DashaLevel::Mahadasha);
    assert_eq!(level0[0].order, 1);

    // At 0°, the Moon is at the very start of Ashwini, so balance = full period
    let ketu_period_days = 7.0 * 365.25;
    let actual_duration = level0[0].duration_days();
    assert!(
        (actual_duration - ketu_period_days).abs() < 0.01,
        "Ketu mahadasha should be full 7y ({ketu_period_days} days), got {actual_duration}"
    );

    // Start JD should be birth JD
    assert!((level0[0].start_jd - birth_jd).abs() < 1e-10);

    // Second period should be Shukra (next in Vimshottari sequence)
    assert_eq!(level0[1].entity, DashaEntity::Graha(Graha::Shukra));
    let shukra_days = 20.0 * 365.25;
    let actual_shukra = level0[1].duration_days();
    assert!(
        (actual_shukra - shukra_days).abs() < 0.01,
        "Shukra mahadasha should be full 20y, got {actual_shukra}"
    );
}

/// Moon at 40° (Rohini nakshatra, index 3) → Chandra mahadasha with partial balance.
#[test]
fn vimshottari_moon_at_40() {
    let cfg = vimshottari_config();
    let birth_jd = 2451545.0;
    let moon_lon = 40.0;

    let level0 = nakshatra_level0(birth_jd, moon_lon, &cfg);
    assert_eq!(level0.len(), 9);

    // Nakshatra index = floor(40 / (360/27)) = floor(40/13.3333) = floor(3.0) = 3
    // Nakshatra 3 = Rohini → Chandra in Vimshottari
    assert_eq!(level0[0].entity, DashaEntity::Graha(Graha::Chandra));

    // elapsed_fraction = (40 % 13.3333) / 13.3333 = 0 / 13.3333 = 0.0
    // (Actually 40.0 / 13.3333 = 3.0 exactly, so fraction = 0.0, balance = full 10y)
    // Since Moon is at exactly the start of Rohini, balance should be full period
    let chandra_days = 10.0 * 365.25;
    let actual = level0[0].duration_days();
    assert!(
        (actual - chandra_days).abs() < 0.5,
        "Chandra balance should be close to full {chandra_days}, got {actual}"
    );

    // Next should be Mangal (Chandra→Mangal in Vimshottari sequence)
    assert_eq!(level0[1].entity, DashaEntity::Graha(Graha::Mangal));
}

/// When Moon is at 0° (start of nakshatra), total span should be exactly 120 years.
#[test]
fn vimshottari_total_span_120y_at_zero() {
    let cfg = vimshottari_config();
    let birth_jd = 2451545.0;
    let moon_lon = 0.0; // At start of nakshatra, balance = full period

    let level0 = nakshatra_level0(birth_jd, moon_lon, &cfg);
    let total_span = level0.last().unwrap().end_jd - level0.first().unwrap().start_jd;
    let expected = 120.0 * 365.25;

    assert!(
        (total_span - expected).abs() < 1e-6,
        "Total span should be exactly {expected} when balance is full, got {total_span}"
    );
}

/// When Moon is mid-nakshatra, total span should be < 120 years (partial first period).
#[test]
fn vimshottari_total_span_partial_balance() {
    let cfg = vimshottari_config();
    let birth_jd = 2451545.0;
    let moon_lon = 123.456; // Mid-nakshatra

    let level0 = nakshatra_level0(birth_jd, moon_lon, &cfg);
    let total_span = level0.last().unwrap().end_jd - level0.first().unwrap().start_jd;
    let max_span = 120.0 * 365.25;

    assert!(total_span > 0.0, "Total span should be positive");
    assert!(
        total_span <= max_span,
        "Total span should be <= {max_span}, got {total_span}"
    );
}

/// Full hierarchy at depth 2 should produce correct level counts.
#[test]
fn vimshottari_hierarchy_depth_2() {
    let cfg = vimshottari_config();
    let birth_jd = 2451545.0;
    let moon_lon = 100.0;
    let variation = DashaVariationConfig::default();

    let result = nakshatra_hierarchy(birth_jd, moon_lon, &cfg, 2, &variation);
    assert!(result.is_ok());

    let hierarchy = result.unwrap();
    assert_eq!(hierarchy.system, DashaSystem::Vimshottari);
    assert_eq!(hierarchy.levels.len(), 3);
    assert_eq!(hierarchy.levels[0].len(), 9); // 9 mahadashas
    assert_eq!(hierarchy.levels[1].len(), 81); // 9*9 antardashas
    assert_eq!(hierarchy.levels[2].len(), 729); // 81*9 pratyantardashas
}

/// Hierarchy at depth 0 should produce only mahadashas.
#[test]
fn vimshottari_hierarchy_depth_0() {
    let cfg = vimshottari_config();
    let birth_jd = 2451545.0;
    let moon_lon = 50.0;
    let variation = DashaVariationConfig::default();

    let result = nakshatra_hierarchy(birth_jd, moon_lon, &cfg, 0, &variation);
    assert!(result.is_ok());

    let hierarchy = result.unwrap();
    assert_eq!(hierarchy.levels.len(), 1);
    assert_eq!(hierarchy.levels[0].len(), 9);
}

/// Snapshot should match hierarchy lookup.
#[test]
fn snapshot_matches_hierarchy() {
    let cfg = vimshottari_config();
    let birth_jd = 2451545.0;
    let moon_lon = 75.0;
    let variation = DashaVariationConfig::default();
    let query_jd = birth_jd + 10_000.0; // ~27 years after birth

    let hierarchy = nakshatra_hierarchy(birth_jd, moon_lon, &cfg, 2, &variation).unwrap();
    let from_hierarchy = snapshot_from_hierarchy(&hierarchy, query_jd);
    let direct = nakshatra_snapshot(birth_jd, moon_lon, &cfg, query_jd, 2, &variation);

    assert_eq!(from_hierarchy.periods.len(), direct.periods.len());

    for (i, (h_period, d_period)) in from_hierarchy
        .periods
        .iter()
        .zip(direct.periods.iter())
        .enumerate()
    {
        assert_eq!(h_period.entity, d_period.entity, "Level {i} entity mismatch");
        assert!(
            (h_period.start_jd - d_period.start_jd).abs() < 1e-10,
            "Level {i} start_jd mismatch"
        );
        assert!(
            (h_period.end_jd - d_period.end_jd).abs() < 1e-10,
            "Level {i} end_jd mismatch"
        );
    }
}

/// Last child end should snap to parent end (no drift).
#[test]
fn last_child_snaps_to_parent() {
    let cfg = vimshottari_config();
    let birth_jd = 2451545.0;
    let moon_lon = 200.0;
    let variation = DashaVariationConfig::default();

    let hierarchy = nakshatra_hierarchy(birth_jd, moon_lon, &cfg, 1, &variation).unwrap();

    // For each mahadasha, check that the last antardasha ends exactly at parent end
    for (parent_idx, parent) in hierarchy.levels[0].iter().enumerate() {
        let children: Vec<_> = hierarchy.levels[1]
            .iter()
            .filter(|c| c.parent_idx == parent_idx as u32)
            .collect();
        assert_eq!(children.len(), 9, "Each mahadasha should have 9 antardashas");

        let last_child = children.last().unwrap();
        assert!(
            (last_child.end_jd - parent.end_jd).abs() < 1e-10,
            "Last antardasha end ({}) should snap to mahadasha end ({})",
            last_child.end_jd,
            parent.end_jd
        );
    }
}

/// Hierarchy at depth 4 should succeed (max level).
#[test]
fn vimshottari_hierarchy_depth_4() {
    let cfg = vimshottari_config();
    let birth_jd = 2451545.0;
    let moon_lon = 150.0;
    let variation = DashaVariationConfig::default();

    let result = nakshatra_hierarchy(birth_jd, moon_lon, &cfg, MAX_DASHA_LEVEL, &variation);
    assert!(result.is_ok());

    let hierarchy = result.unwrap();
    assert_eq!(hierarchy.levels.len(), 5);
    assert_eq!(hierarchy.levels[0].len(), 9);
    assert_eq!(hierarchy.levels[1].len(), 81);
    assert_eq!(hierarchy.levels[2].len(), 729);
    assert_eq!(hierarchy.levels[3].len(), 6561);
    assert_eq!(hierarchy.levels[4].len(), 59049);
}

/// Contiguity: all levels should have no gaps between adjacent periods.
#[test]
fn all_levels_contiguous() {
    let cfg = vimshottari_config();
    let birth_jd = 2451545.0;
    let moon_lon = 300.0;
    let variation = DashaVariationConfig::default();

    let hierarchy = nakshatra_hierarchy(birth_jd, moon_lon, &cfg, 2, &variation).unwrap();

    for (lvl, level) in hierarchy.levels.iter().enumerate() {
        // Check siblings within same parent are contiguous
        let mut i = 0;
        while i < level.len() {
            // Find range of siblings with same parent_idx
            let parent = level[i].parent_idx;
            let mut j = i + 1;
            while j < level.len() && level[j].parent_idx == parent {
                let prev_end = level[j - 1].end_jd;
                let curr_start = level[j].start_jd;
                assert!(
                    (prev_end - curr_start).abs() < 1e-10,
                    "Level {lvl}, siblings {}/{}: gap between {prev_end} and {curr_start}",
                    j - 1,
                    j
                );
                j += 1;
            }
            i = j;
        }
    }
}
