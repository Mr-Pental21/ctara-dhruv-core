"""Tests for graha positions, core bindus, and full kundali."""

import pytest
from datetime import timedelta
from conftest import skip_no_kernels, skip_no_eop


J2000 = 2451545.0


@skip_no_kernels
class TestGrahaLongitudes:
    def test_graha_longitudes_sidereal(self, engine_handles):
        """Sidereal longitudes for all 9 grahas should be in [0, 360)."""
        from ctara_dhruv.kundali import graha_longitudes
        from ctara_dhruv.engine import engine
        result = graha_longitudes(engine(), jd_tdb=J2000, ayanamsha_system=0)
        assert len(result.longitudes) == 9
        assert result.outer_planets is not None
        assert len(result.outer_planets) == 3
        for lon in result.longitudes:
            assert 0 <= lon < 360

    def test_graha_longitudes_tropical(self, engine_handles):
        """Tropical longitudes should differ from sidereal by ~ayanamsha."""
        from ctara_dhruv import GrahaLongitudeKind, GrahaLongitudesConfig
        from ctara_dhruv.kundali import graha_longitudes
        from ctara_dhruv.engine import engine
        sid = graha_longitudes(engine(), jd_tdb=J2000, ayanamsha_system=0)
        trop = graha_longitudes(
            engine(),
            jd_tdb=J2000,
            config=GrahaLongitudesConfig(kind=GrahaLongitudeKind.TROPICAL),
        )
        # Difference should be approximately the ayanamsha (~23.85 at J2000)
        diff = (trop.longitudes[0] - sid.longitudes[0]) % 360
        assert 23.0 < diff < 25.0


@skip_no_kernels
@skip_no_eop
class TestGrahaPositions:
    def test_graha_positions_basic(self, engine_handles):
        """Compute graha positions with default config."""
        from ctara_dhruv.kundali import graha_positions
        from ctara_dhruv.engine import engine, lsk, eop
        result = graha_positions(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
        )
        assert len(result.grahas) == 9
        assert len(result.outer_planets) == 3
        for g in result.grahas:
            assert 0 <= g.sidereal_longitude < 360
            assert 0 <= g.rashi_index <= 11

    def test_graha_positions_with_lagna(self, engine_handles):
        """Compute with include_lagna flag."""
        from ctara_dhruv.kundali import graha_positions
        from ctara_dhruv.engine import engine, lsk, eop
        result = graha_positions(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            config={"include_lagna": 1},
        )
        # Lagna should have a valid longitude
        assert 0 <= result.lagna.sidereal_longitude < 360

    def test_graha_positions_equatorial(self, engine_handles):
        """include_equatorial exposes RA/dec/ecliptic latitude and GMST/GAST."""
        from ctara_dhruv.kundali import graha_positions
        from ctara_dhruv.engine import engine, lsk, eop
        result = graha_positions(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            config={"include_equatorial": True},
        )
        for g in result.grahas:
            assert g.equatorial_valid
            assert 0 <= g.right_ascension_deg < 360
            assert abs(g.declination_deg) <= 90
        # Rahu (7) and Ketu (8) report ecliptic latitude exactly 0.
        assert result.grahas[7].ecliptic_latitude_deg == 0.0
        assert result.grahas[8].ecliptic_latitude_deg == 0.0
        assert result.earth_orientation_valid
        assert 0 <= result.gmst_deg < 360
        assert 0 <= result.gast_deg < 360

    def test_graha_positions_no_equatorial_by_default(self, engine_handles):
        """Without include_equatorial the fields stay invalid."""
        from ctara_dhruv.kundali import graha_positions
        from ctara_dhruv.engine import engine, lsk, eop
        result = graha_positions(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            config={"include_lagna": 1},
        )
        assert not result.grahas[0].equatorial_valid
        assert not result.earth_orientation_valid

    def test_graha_positions_series(self, engine_handles):
        """Series sampling matches the single-epoch call on the first point."""
        from ctara_dhruv.kundali import graha_positions, graha_positions_series
        from ctara_dhruv.engine import engine, lsk, eop
        points = graha_positions_series(
            engine(), lsk(), eop(),
            from_utc=(2024, 1, 15, 12, 0, 0.0),
            to_utc=(2024, 1, 15, 14, 0, 0.0),
            step_minutes=60,
            location=(28.6139, 77.2090),
            config={"include_equatorial": True},
        )
        assert len(points) == 3
        assert points[0].utc[3] == 12
        assert points[2].utc[3] == 14
        single = graha_positions(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 12, 0, 0.0),
            location=(28.6139, 77.2090),
            config={"include_equatorial": True},
        )
        for i in range(9):
            assert (
                points[0].positions.grahas[i].sidereal_longitude
                == single.grahas[i].sidereal_longitude
            )
        assert points[0].positions.gmst_deg == single.gmst_deg

    def test_graha_positions_series_rejects_zero_step(self, engine_handles):
        """step_minutes == 0 raises."""
        import pytest
        from ctara_dhruv.kundali import graha_positions_series
        from ctara_dhruv.engine import engine, lsk, eop
        with pytest.raises(Exception):
            graha_positions_series(
                engine(), lsk(), eop(),
                from_utc=(2024, 1, 15, 12, 0, 0.0),
                to_utc=(2024, 1, 15, 14, 0, 0.0),
                step_minutes=0,
                location=(28.6139, 77.2090),
            )


@skip_no_kernels
@skip_no_eop
class TestFullKundali:
    def test_full_kundali_default(self, engine_handles):
        """Full kundali with default config should have core sections."""
        from ctara_dhruv.kundali import full_kundali
        from ctara_dhruv.engine import engine, lsk, eop
        result = full_kundali(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
        )
        assert result.ayanamsha_deg > 0
        # Default config should include bhava, graha, etc.
        if result.graha_positions is not None:
            assert len(result.graha_positions.grahas) == 9
        if result.bhava_cusps is not None:
            assert len(result.bhava_cusps.bhavas) == 12
        assert result.sphutas is not None
        assert len(result.sphutas.longitudes) == 16

    def test_full_kundali_dasha_hierarchies(self, engine_handles):
        """Full kundali should expose decoded dasha hierarchies with per-system depth."""
        from ctara_dhruv.kundali import full_kundali, full_kundali_config_default
        from ctara_dhruv.engine import engine, lsk, eop

        cfg = full_kundali_config_default()
        cfg.include_dasha = 1
        cfg.dasha_config.count = 2
        cfg.dasha_config.systems[0] = 0
        cfg.dasha_config.systems[1] = 1
        cfg.dasha_config.max_levels[0] = 0
        cfg.dasha_config.max_levels[1] = 1

        result = full_kundali(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            config=cfg,
        )

        assert result.dasha is not None
        assert len(result.dasha) == 2
        assert result.dasha[0].system == 0
        assert result.dasha[1].system == 1
        assert len(result.dasha[0].levels) == 1
        assert len(result.dasha[1].levels) == 2

    def test_full_kundali_ashtakavarga(self, engine_handles):
        """Ashtakavarga in full kundali should have 7 BAVs."""
        from ctara_dhruv.kundali import full_kundali
        from ctara_dhruv.engine import engine, lsk, eop
        result = full_kundali(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
        )
        if result.ashtakavarga is not None:
            assert len(result.ashtakavarga.bavs) == 7
            for bav in result.ashtakavarga.bavs:
                assert len(bav.points) == 12
                assert len(bav.contributors) == 12
                for p in bav.points:
                    assert 0 <= p <= 8
                for i in range(12):
                    assert len(bav.contributors[i]) == 8
                    assert sum(bav.contributors[i]) == bav.points[i]
            sav = result.ashtakavarga.sav
            assert len(sav.total_points) == 12
            assert sum(sav.total_points) == 337  # SAV constant

    def test_full_kundali_amsha_scope_and_selection(self, engine_handles):
        """Full kundali should expose scoped amsha chart sections when selected."""
        from ctara_dhruv.kundali import full_kundali, full_kundali_config_default
        from ctara_dhruv.engine import engine, lsk, eop

        cfg = full_kundali_config_default()
        cfg.include_bhava_cusps = 1
        cfg.include_bindus = 1
        cfg.include_upagrahas = 1
        cfg.include_sphutas = 1
        cfg.include_special_lagnas = 1
        cfg.include_amshas = 1
        cfg.amsha_selection.count = 1
        cfg.amsha_selection.codes[0] = 9
        cfg.amsha_selection.variations[0] = 0
        cfg.amsha_scope.include_bhava_cusps = 1
        cfg.amsha_scope.include_arudha_padas = 1
        cfg.amsha_scope.include_upagrahas = 1
        cfg.amsha_scope.include_sphutas = 1
        cfg.amsha_scope.include_special_lagnas = 1

        result = full_kundali(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            config=cfg,
        )

        assert result.amshas is not None
        assert len(result.amshas) == 1
        chart = result.amshas[0]
        assert chart.amsha_code == 9
        assert chart.bhava_cusps is not None
        assert len(chart.bhava_cusps) == 12
        assert chart.arudha_padas is not None
        assert len(chart.arudha_padas) == 12
        assert chart.upagrahas is not None
        assert len(chart.upagrahas) == 11
        assert chart.sphutas is not None
        assert len(chart.sphutas) == 16
        assert chart.special_lagnas is not None
        assert len(chart.special_lagnas) == 8

    def test_full_kundali_returns_resolved_amsha_union(self, engine_handles):
        """Embedded amshas should include explicit and internally required charts."""
        from ctara_dhruv.kundali import full_kundali, full_kundali_config_default
        from ctara_dhruv.engine import engine, lsk, eop

        cfg = full_kundali_config_default()
        cfg.include_amshas = 1
        cfg.include_shadbala = 1
        cfg.include_vimsopaka = 1
        cfg.amsha_selection.count = 1
        cfg.amsha_selection.codes[0] = 2
        cfg.amsha_selection.variations[0] = 1

        result = full_kundali(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            config=cfg,
        )

        assert result.amshas is not None
        assert len(result.amshas) == 16
        assert result.amshas[0].amsha_code == 2
        assert result.amshas[0].variation_code == 1
        assert any(chart.amsha_code == 60 for chart in result.amshas)

    def test_charakaraka_for_date(self, engine_handles):
        """Direct charakaraka call should return non-empty assignments."""
        from ctara_dhruv.kundali import charakaraka_for_date
        from ctara_dhruv.engine import engine, lsk, eop
        result = charakaraka_for_date(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            ayanamsha_system=0,
            use_nutation=1,
            scheme=3,  # mixed parashara
        )
        assert result.scheme == 3
        assert len(result.entries) >= 7
        assert len(result.entries) <= 8
        assert result.entries[0].rank == 1

    def test_full_kundali_charakaraka_section(self, engine_handles):
        """Full kundali should include charakaraka when enabled in config."""
        from ctara_dhruv.kundali import full_kundali, full_kundali_config_default
        from ctara_dhruv.engine import engine, lsk, eop

        cfg = full_kundali_config_default()
        cfg.include_charakaraka = 1
        cfg.charakaraka_scheme = 2  # seven-pk-merged-mk

        result = full_kundali(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            config=cfg,
        )
        assert result.charakaraka is not None
        assert result.charakaraka.scheme == 2
        assert len(result.charakaraka.entries) == 7


@skip_no_kernels
@skip_no_eop
class TestGrahaSpeeds:
    """include_speeds / speed_valid (ABI v91)."""

    def test_speeds_present_when_requested(self, engine_handles):
        from ctara_dhruv.kundali import graha_positions
        from ctara_dhruv.engine import engine, lsk, eop
        result = graha_positions(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            config={"include_speeds": True, "include_lagna": 1},
        )
        for g in result.grahas:
            assert g.speed_valid
        # Sun ~1 deg/day, Moon ~12-15 deg/day, both direct.
        assert 0.9 < result.grahas[0].longitude_speed_deg_per_day < 1.1
        assert 11.0 < result.grahas[1].longitude_speed_deg_per_day < 16.0
        # Rahu/Ketu report latitude rate exactly 0.
        assert result.grahas[7].latitude_speed_deg_per_day == 0.0
        assert result.grahas[8].latitude_speed_deg_per_day == 0.0
        # Lagna sweeps the zodiac daily; latitude rate 0.
        assert result.lagna.speed_valid
        assert result.lagna.longitude_speed_deg_per_day > 100.0
        assert result.lagna.latitude_speed_deg_per_day == 0.0
        for o in result.outer_planets:
            assert o.speed_valid

    def test_speeds_absent_by_default(self, engine_handles):
        from ctara_dhruv.kundali import graha_positions
        from ctara_dhruv.engine import engine, lsk, eop
        result = graha_positions(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            config={"include_lagna": 1},
        )
        for g in result.grahas + [result.lagna] + result.outer_planets:
            assert not g.speed_valid
            assert g.longitude_speed_deg_per_day == 0.0
            assert g.latitude_speed_deg_per_day == 0.0

    def test_outer_planet_retrograde_with_basic_states(self, engine_handles):
        """Outer planets get a real retrograde flag; it agrees with the speed sign."""
        from ctara_dhruv.kundali import graha_positions
        from ctara_dhruv.engine import engine, lsk, eop
        result = graha_positions(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            config={
                "include_speeds": True,
                "basic_states_config": {"include_basic_states": 1},
            },
        )
        for o in result.outer_planets:
            assert o.basic_states_valid
            assert o.basic_states.retrograde == (o.longitude_speed_deg_per_day < 0)
        # Uranus was retrograde on 2024-01-15 (it stationed direct on 2024-01-27).
        assert result.outer_planets[0].basic_states.retrograde
        assert result.outer_planets[0].longitude_speed_deg_per_day < 0


@skip_no_kernels
@skip_no_eop
class TestFullKundaliV91Sections:
    """tara_bala / samvatsara_officers / transit_ashtakavarga sections."""

    BIRTH = (2024, 1, 15, 6, 0, 0.0)
    LOC = (28.6139, 77.2090)

    def _run(self, cfg=None):
        from ctara_dhruv.kundali import full_kundali
        from ctara_dhruv.engine import engine, lsk, eop
        return full_kundali(
            engine(), lsk(), eop(), jd_utc=self.BIRTH, location=self.LOC, config=cfg,
        )

    def test_sections_absent_by_default(self, engine_handles):
        result = self._run()
        assert result.tara_bala is None
        assert result.samvatsara_officers is None
        assert result.transit_ashtakavarga is None

    def test_tara_bala_from_chart_moon(self, engine_handles):
        from ctara_dhruv.kundali import full_kundali_config_default
        from ctara_dhruv.vedic import tara_bala
        cfg = full_kundali_config_default()
        cfg.include_tara_bala = 1
        cfg.graha_positions_config.include_nakshatra = 1
        cfg.graha_positions_config.include_lagna = 1
        result = self._run(cfg)
        tb = result.tara_bala
        assert tb is not None
        assert tb.reference_is_chart_moon
        moon = result.graha_positions.grahas[1]
        assert tb.reference_nakshatra_index == moon.nakshatra_index
        assert tb.grahas[1].tara_bala.tara_index == 0  # Moon is its own Janma tara
        assert tb.grahas[1].tara_bala.count == 1
        assert tb.lagna.nakshatra_index == result.graha_positions.lagna.nakshatra_index
        for entry in tb.grahas + [tb.lagna]:
            assert entry.tara_bala == tara_bala(tb.reference_nakshatra_index, entry.nakshatra_index)

    def test_tara_bala_explicit_reference(self, engine_handles):
        from ctara_dhruv.kundali import full_kundali_config_default
        cfg = full_kundali_config_default()
        cfg.include_tara_bala = 1
        cfg.tara_bala_reference_nakshatra_index = 5
        result = self._run(cfg)
        assert result.tara_bala is not None
        assert result.tara_bala.reference_nakshatra_index == 5
        assert not result.tara_bala.reference_is_chart_moon

    def test_samvatsara_officers_section(self, engine_handles):
        from ctara_dhruv.kundali import full_kundali_config_default
        from ctara_dhruv.vedic import samvatsara_office_name
        cfg = full_kundali_config_default()
        cfg.include_samvatsara_officers = 1
        result = self._run(cfg)
        so = result.samvatsara_officers
        assert so is not None
        assert 0 <= so.varsha.samvatsara_index <= 59
        assert len(so.officers) == 9
        for i, o in enumerate(so.officers):
            assert o.office_index == i
            assert samvatsara_office_name(i) is not None
            assert 0 <= o.vaar_index <= 6
            assert o.lord_index == o.vaar_index  # vaar lords are Surya..Shani in order
            day_start = o.vedic_day_start.to_datetime()
            event = o.event_utc.to_datetime()
            if i == 0:
                # Raja (udaya rule): the sunrise falls inside the tithi.
                assert event <= day_start < event + timedelta(days=1.2)
            else:
                assert day_start <= event < day_start + timedelta(days=1.1)

    def test_transit_ashtakavarga_section(self, engine_handles):
        from ctara_dhruv.kundali import full_kundali_config_default
        from ctara_dhruv.vedic import kakshya_lord
        base_cfg = full_kundali_config_default()
        base_cfg.graha_positions_config.include_lagna = 1
        base = self._run(base_cfg)
        rashis = [base.graha_positions.grahas[i].rashi_index for i in range(7)]
        lagna_rashi = base.graha_positions.lagna.rashi_index

        cfg = full_kundali_config_default()
        cfg.graha_positions_config.include_lagna = 1
        cfg.include_transit_ashtakavarga = 1
        for i, r in enumerate(rashis):
            cfg.transit_ashtakavarga_reference.graha_rashis[i] = r
        cfg.transit_ashtakavarga_reference.lagna_rashi = lagna_rashi
        result = self._run(cfg)
        ta = result.transit_ashtakavarga
        assert ta is not None
        assert ta.reference.graha_rashis == rashis
        assert ta.reference.lagna_rashi == lagna_rashi
        assert len(ta.entries) == 7
        sav = base.ashtakavarga.sav
        for i, e in enumerate(ta.entries):
            assert e.graha_index == i
            # Transit at the natal moment occupies the natal rashi.
            assert e.rashi_index == rashis[i]
            bav = base.ashtakavarga.bavs[i]
            assert e.bav_bindus == bav.points[e.rashi_index]
            assert e.sav_total == sav.total_points[e.rashi_index]
            assert e.sav_after_trikona == sav.after_trikona[e.rashi_index]
            assert e.sav_after_ekadhipatya == sav.after_ekadhipatya[e.rashi_index]
            assert 0 <= e.kakshya_index <= 7
            assert e.kakshya_lord == kakshya_lord(e.kakshya_index)
            assert e.kakshya_bindu == bool(bav.contributors[e.rashi_index][e.kakshya_lord])
