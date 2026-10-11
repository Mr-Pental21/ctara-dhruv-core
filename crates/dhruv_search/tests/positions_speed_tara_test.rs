//! Integration tests for per-entry speeds, outer-planet retrograde,
//! vargottama, tara bala, transit ashtakavarga, samvatsara officers and
//! progress fields (ABI v91 additions). Requires kernel files; skips
//! gracefully when absent.

use std::path::Path;

use dhruv_core::{Engine, EngineConfig};
use dhruv_search::sankranti_types::SankrantiConfig;
use dhruv_search::{
    AmshaSelectionConfig, AshtakavargaReference, FullKundaliConfig, GrahaPositionsConfig,
    PANCHANG_INCLUDE_ALL_CORE, SamvatsaraOffice, TARA_BALA_REFERENCE_CHART_MOON,
    full_kundali_for_date, graha_positions, samvatsara_officers, tara_bala_for_date,
    tara_bala_from_longitudes, transit_ashtakavarga_for_date,
};
use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::riseset_types::{GeoLocation, RiseSetConfig};
use dhruv_vedic_base::{Amsha, BhavaConfig, Graha, Tara, Vaar, vaar_lord};

const SPK_PATH: &str = "../../kernels/data/de442s.bsp";
const LSK_PATH: &str = "../../kernels/data/naif0012.tls";
const EOP_PATH: &str = "../../kernels/data/finals2000A.all";

fn load_engine() -> Option<Engine> {
    if !Path::new(SPK_PATH).exists() || !Path::new(LSK_PATH).exists() {
        eprintln!("Skipping positions_speed_tara_test: kernel files not found");
        return None;
    }
    let config = EngineConfig::with_single_spk(SPK_PATH.into(), LSK_PATH.into(), 1024, false);
    Engine::new(config).ok()
}

fn load_eop() -> Option<EopKernel> {
    if !Path::new(EOP_PATH).exists() {
        eprintln!("Skipping positions_speed_tara_test: EOP file not found");
        return None;
    }
    EopKernel::load(Path::new(EOP_PATH)).ok()
}

fn new_delhi() -> GeoLocation {
    GeoLocation::new(28.6139, 77.2090, 0.0)
}

fn aya() -> SankrantiConfig {
    SankrantiConfig::default_lahiri()
}

#[test]
fn speeds_absent_by_default_and_present_when_requested() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let utc = UtcTime::new(2024, 1, 15, 12, 0, 0.0);
    let base = GrahaPositionsConfig {
        include_lagna: true,
        include_outer_planets: true,
        ..GrahaPositionsConfig::default()
    };
    let without = graha_positions(
        &engine,
        &eop,
        &utc,
        &new_delhi(),
        &BhavaConfig::default(),
        &aya(),
        &base,
    )
    .unwrap();
    assert!(without.grahas.iter().all(|e| !e.speed_valid));
    assert!(!without.lagna.speed_valid);
    assert!(without.outer_planets.iter().all(|e| !e.speed_valid));

    let with = graha_positions(
        &engine,
        &eop,
        &utc,
        &new_delhi(),
        &BhavaConfig::default(),
        &aya(),
        &GrahaPositionsConfig {
            include_speeds: true,
            ..base
        },
    )
    .unwrap();
    for (i, e) in with.grahas.iter().enumerate() {
        assert!(e.speed_valid, "graha {i} speed_valid");
        assert!(
            e.longitude_speed_deg_per_day.is_finite() && e.latitude_speed_deg_per_day.is_finite()
        );
        assert_eq!(e.sidereal_longitude, without.grahas[i].sidereal_longitude);
    }
    // Sun ~0.95-1.02 deg/day, Moon ~11.8-15.4 deg/day, always direct.
    let sun = with.grahas[Graha::Surya.index() as usize].longitude_speed_deg_per_day;
    let moon = with.grahas[Graha::Chandra.index() as usize].longitude_speed_deg_per_day;
    assert!((0.95..1.03).contains(&sun), "sun speed {sun}");
    assert!((11.5..15.5).contains(&moon), "moon speed {moon}");
    // Mean/true node: retrograde on average (~-0.053 deg/day), Ketu equal.
    let rahu = with.grahas[Graha::Rahu.index() as usize].longitude_speed_deg_per_day;
    let ketu = with.grahas[Graha::Ketu.index() as usize].longitude_speed_deg_per_day;
    assert_eq!(rahu, ketu);
    assert_eq!(
        with.grahas[Graha::Rahu.index() as usize].latitude_speed_deg_per_day,
        0.0
    );
    // Lagna: roughly one zodiac per sidereal day.
    assert!(with.lagna.speed_valid);
    assert!(
        with.lagna.longitude_speed_deg_per_day > 200.0
            && with.lagna.longitude_speed_deg_per_day < 700.0,
        "lagna speed {}",
        with.lagna.longitude_speed_deg_per_day
    );
    assert_eq!(with.lagna.latitude_speed_deg_per_day, 0.0);
    for e in &with.outer_planets {
        assert!(e.speed_valid);
        assert!(e.longitude_speed_deg_per_day.abs() < 0.1);
    }
}

#[test]
fn speed_sign_matches_retrograde_flag_and_outer_planets_get_retrograde() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    // Several epochs so at least one sapta graha is retrograde.
    let epochs = [
        UtcTime::new(2024, 1, 15, 12, 0, 0.0),
        UtcTime::new(2024, 4, 10, 0, 0, 0.0),
        UtcTime::new(2025, 8, 1, 0, 0, 0.0),
    ];
    let mut saw_retrograde = false;
    let mut saw_outer_retrograde = false;
    for utc in epochs {
        let mut config = GrahaPositionsConfig {
            include_outer_planets: true,
            include_speeds: true,
            ..GrahaPositionsConfig::default()
        };
        config.basic_states_config.include_basic_states = true;
        let result = graha_positions(
            &engine,
            &eop,
            &utc,
            &new_delhi(),
            &BhavaConfig::default(),
            &aya(),
            &config,
        )
        .unwrap();
        for i in 0..7 {
            let e = &result.grahas[i];
            assert_eq!(
                e.basic_states.retrograde,
                e.longitude_speed_deg_per_day < 0.0,
                "retrograde flag vs speed sign for graha {i} at {utc:?}"
            );
            saw_retrograde |= e.basic_states.retrograde;
        }
        for e in &result.outer_planets {
            assert!(e.basic_states_valid);
            assert_eq!(
                e.basic_states.retrograde,
                e.longitude_speed_deg_per_day < 0.0
            );
            saw_outer_retrograde |= e.basic_states.retrograde;
        }
    }
    assert!(
        saw_retrograde,
        "expected at least one retrograde sapta graha"
    );
    assert!(
        saw_outer_retrograde,
        "expected at least one retrograde outer planet"
    );
}

#[test]
fn outer_planet_retrograde_without_speed_output() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let utc = UtcTime::new(2024, 1, 15, 12, 0, 0.0);
    let mut config = GrahaPositionsConfig {
        include_outer_planets: true,
        ..GrahaPositionsConfig::default()
    };
    config.basic_states_config.include_basic_states = true;
    let result = graha_positions(
        &engine,
        &eop,
        &utc,
        &new_delhi(),
        &BhavaConfig::default(),
        &aya(),
        &config,
    )
    .unwrap();
    // Mid-January 2024: Uranus retrograde (until late January), Neptune and
    // Pluto direct.
    assert!(result.outer_planets[0].basic_states.retrograde, "Uranus");
    assert!(!result.outer_planets[1].basic_states.retrograde, "Neptune");
    assert!(!result.outer_planets[2].basic_states.retrograde, "Pluto");
    assert!(result.outer_planets.iter().all(|e| !e.speed_valid));
}

#[test]
fn vargottama_flag_matches_d1_rashi() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let utc = UtcTime::new(2024, 1, 15, 12, 0, 0.0);
    let mut amsha_selection = AmshaSelectionConfig::default();
    amsha_selection.count = 2;
    amsha_selection.codes[0] = Amsha::D1.code();
    amsha_selection.codes[1] = Amsha::D9.code();
    let config = FullKundaliConfig {
        include_amshas: true,
        amsha_selection,
        ..FullKundaliConfig::default()
    };
    let result = full_kundali_for_date(
        &engine,
        &eop,
        &utc,
        &new_delhi(),
        &BhavaConfig::default(),
        &RiseSetConfig::default(),
        &aya(),
        &config,
    )
    .unwrap();
    let positions = result.graha_positions.as_ref().unwrap();
    let amshas = result.amshas.as_ref().unwrap();
    let d1 = amshas.charts.iter().find(|c| c.amsha == Amsha::D1).unwrap();
    let d9 = amshas.charts.iter().find(|c| c.amsha == Amsha::D9).unwrap();
    assert!(d1.grahas.iter().all(|e| !e.vargottama));
    assert!(!d1.lagna.vargottama);
    for (i, e) in d9.grahas.iter().enumerate() {
        assert_eq!(
            e.vargottama,
            e.rashi_index == positions.grahas[i].rashi_index
        );
    }
    assert_eq!(
        d9.lagna.vargottama,
        d9.lagna.rashi_index == positions.lagna.rashi_index
    );
}

#[test]
fn tara_bala_from_longitudes_counts_from_reference() {
    let lons = [10.0, 100.0, 200.0, 300.0, 50.0, 150.0, 250.0, 350.0, 170.0];
    let result = tara_bala_from_longitudes(&lons, 123.0, TARA_BALA_REFERENCE_CHART_MOON).unwrap();
    // Moon at 100° = nakshatra 7 (Pushya).
    assert_eq!(result.reference_nakshatra_index, 7);
    assert!(result.reference_is_chart_moon);
    assert_eq!(result.grahas[1].tara_bala.tara, Tara::Janma);
    assert_eq!(result.grahas[1].tara_bala.count, 1);
    // Sun at 10° = nakshatra 0: count from 7 to 0 = 21 -> index 20 -> 20 % 9 = 2 = Vipat, cycle 3.
    assert_eq!(result.grahas[0].nakshatra_index, 0);
    assert_eq!(result.grahas[0].tara_bala.count, 21);
    assert_eq!(result.grahas[0].tara_bala.tara, Tara::Vipat);
    assert_eq!(result.grahas[0].tara_bala.cycle, 3);
    // Explicit reference.
    let explicit = tara_bala_from_longitudes(&lons, 123.0, 0).unwrap();
    assert_eq!(explicit.reference_nakshatra_index, 0);
    assert!(!explicit.reference_is_chart_moon);
    assert_eq!(explicit.grahas[0].tara_bala.tara, Tara::Janma);
    assert!(tara_bala_from_longitudes(&lons, 123.0, 27).is_err());
}

#[test]
fn tara_bala_for_date_matches_kundali_section() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let utc = UtcTime::new(2024, 1, 15, 12, 0, 0.0);
    let direct = tara_bala_for_date(&engine, &eop, &utc, &new_delhi(), &aya(), 4).unwrap();
    assert_eq!(direct.reference_nakshatra_index, 4);
    let config = FullKundaliConfig {
        include_tara_bala: true,
        tara_bala_reference_nakshatra_index: 4,
        graha_positions_config: GrahaPositionsConfig {
            include_nakshatra: true,
            include_lagna: true,
            ..GrahaPositionsConfig::default()
        },
        ..FullKundaliConfig::default()
    };
    let result = full_kundali_for_date(
        &engine,
        &eop,
        &utc,
        &new_delhi(),
        &BhavaConfig::default(),
        &RiseSetConfig::default(),
        &aya(),
        &config,
    )
    .unwrap();
    assert_eq!(result.tara_bala, Some(direct));
    let positions = result.graha_positions.as_ref().unwrap();
    for (i, e) in direct.grahas.iter().enumerate() {
        assert_eq!(e.nakshatra_index, positions.grahas[i].nakshatra_index);
    }
    // Default config leaves the section out.
    let plain = full_kundali_for_date(
        &engine,
        &eop,
        &utc,
        &new_delhi(),
        &BhavaConfig::default(),
        &RiseSetConfig::default(),
        &aya(),
        &FullKundaliConfig::default(),
    )
    .unwrap();
    assert!(plain.tara_bala.is_none());
    assert!(plain.samvatsara_officers.is_none());
    assert!(plain.transit_ashtakavarga.is_none());
}

#[test]
fn transit_ashtakavarga_section_requires_reference() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let utc = UtcTime::new(2024, 1, 15, 12, 0, 0.0);
    let missing = FullKundaliConfig {
        include_transit_ashtakavarga: true,
        ..FullKundaliConfig::default()
    };
    assert!(
        full_kundali_for_date(
            &engine,
            &eop,
            &utc,
            &new_delhi(),
            &BhavaConfig::default(),
            &RiseSetConfig::default(),
            &aya(),
            &missing,
        )
        .is_err()
    );
    let reference = AshtakavargaReference {
        graha_rashis: [3, 7, 0, 11, 5, 9, 2],
        lagna_rashi: 1,
    };
    let config = FullKundaliConfig {
        include_transit_ashtakavarga: true,
        transit_ashtakavarga_reference: Some(reference),
        ..FullKundaliConfig::default()
    };
    let result = full_kundali_for_date(
        &engine,
        &eop,
        &utc,
        &new_delhi(),
        &BhavaConfig::default(),
        &RiseSetConfig::default(),
        &aya(),
        &config,
    )
    .unwrap();
    let section = result.transit_ashtakavarga.unwrap();
    let direct = transit_ashtakavarga_for_date(&engine, &eop, &utc, &aya(), &reference).unwrap();
    assert_eq!(section, direct);
    let positions = result.graha_positions.as_ref().unwrap();
    for (i, e) in section.entries.iter().enumerate() {
        assert_eq!(e.graha_index as usize, i);
        assert_eq!(e.rashi_index, positions.grahas[i].rashi_index);
        assert!(e.bav_bindus <= 8 && e.kakshya_index < 8 && e.kakshya_lord < 8);
    }
    let bad = AshtakavargaReference {
        graha_rashis: [12, 0, 0, 0, 0, 0, 0],
        lagna_rashi: 0,
    };
    assert!(transit_ashtakavarga_for_date(&engine, &eop, &utc, &aya(), &bad).is_err());
}

#[test]
fn samvatsara_officers_2026_lunar_year() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let utc = UtcTime::new(2026, 10, 12, 0, 0, 0.0);
    let result = samvatsara_officers(
        &engine,
        &eop,
        &utc,
        &new_delhi(),
        &RiseSetConfig::default(),
        &aya(),
    )
    .unwrap();
    assert_eq!(
        (
            result.varsha.start.year,
            result.varsha.start.month,
            result.varsha.start.day
        ),
        (2026, 3, 19)
    );
    for (i, officer) in result.officers.iter().enumerate() {
        assert_eq!(officer.office.index() as usize, i);
        assert_eq!(officer.lord, vaar_lord(officer.vaar));
        // The defining Vedic day starts at or before the event (udaya rule
        // may move the Raja day to the next sunrise).
        let event_jd = officer.event_utc.to_jd_tdb(engine.lsk());
        let day_jd = officer.vedic_day_start.to_jd_tdb(engine.lsk());
        if officer.office == SamvatsaraOffice::Raja {
            assert!((day_jd - event_jd).abs() < 1.5);
        } else {
            assert!(day_jd <= event_jd && event_jd - day_jd < 1.0);
            assert!(event_jd >= result.varsha.start.to_jd_tdb(engine.lsk()));
        }
    }
    // 2026-03-19 01:23 UTC pratipada start; first sunrise inside the tithi
    // at New Delhi is 2026-03-19 (Thursday) -> Raja = Guru.
    let raja = result.officer(SamvatsaraOffice::Raja);
    assert_eq!(raja.vaar, Vaar::Guruvaar);
    assert_eq!(raja.lord, Graha::Guru);
    // Mesha sankranti 2026-04-14 (Tuesday, IST) -> Mantri = Mangal.
    let mantri = result.officer(SamvatsaraOffice::Mantri);
    assert_eq!(mantri.event_utc.month, 4);
    assert_eq!(mantri.vaar, Vaar::Mangalvaar);
    assert_eq!(mantri.lord, Graha::Mangal);
    // Sankranti months within the year.
    assert_eq!(
        result.officer(SamvatsaraOffice::Sasyesha).event_utc.month,
        7
    );
    assert_eq!(
        result.officer(SamvatsaraOffice::Durgesha).event_utc.month,
        8
    );
    assert_eq!(
        result.officer(SamvatsaraOffice::Rasesha).event_utc.month,
        10
    );
    assert_eq!(
        result.officer(SamvatsaraOffice::Dhanyesha).event_utc.month,
        12
    );
    assert_eq!(
        result.officer(SamvatsaraOffice::Nirasesha).event_utc.month,
        1
    );
    assert_eq!(
        result.officer(SamvatsaraOffice::Phalesha).event_utc.month,
        3
    );
    assert_eq!(
        result.officer(SamvatsaraOffice::Meghesha).event_utc.month,
        6
    );

    // Kundali section matches the standalone call.
    let config = FullKundaliConfig {
        include_samvatsara_officers: true,
        ..FullKundaliConfig::default()
    };
    let kundali = full_kundali_for_date(
        &engine,
        &eop,
        &utc,
        &new_delhi(),
        &BhavaConfig::default(),
        &RiseSetConfig::default(),
        &aya(),
        &config,
    )
    .unwrap();
    assert_eq!(kundali.samvatsara_officers, Some(result));
}

#[test]
fn panchang_and_dasha_progress_in_unit_interval() {
    let Some(engine) = load_engine() else { return };
    let Some(eop) = load_eop() else { return };
    let utc = UtcTime::new(2024, 1, 15, 12, 0, 0.0);
    let config = FullKundaliConfig {
        panchang_include_mask: PANCHANG_INCLUDE_ALL_CORE,
        ..FullKundaliConfig::default()
    };
    let result = full_kundali_for_date(
        &engine,
        &eop,
        &utc,
        &new_delhi(),
        &BhavaConfig::default(),
        &RiseSetConfig::default(),
        &aya(),
        &config,
    )
    .unwrap();
    let p = result.panchang.unwrap();
    let in_unit = |v: f64| (0.0..=1.0).contains(&v);
    assert!(in_unit(p.tithi.unwrap().progress));
    assert!(in_unit(p.karana.unwrap().progress));
    assert!(in_unit(p.yoga.unwrap().progress));
    assert!(in_unit(p.nakshatra.unwrap().progress));
    let vaar = p.vaar.unwrap();
    assert!(vaar.progress > 0.0 && vaar.progress < 1.0);
    assert!(in_unit(p.hora.unwrap().progress));
    assert!(in_unit(p.ghatika.unwrap().progress));
}
