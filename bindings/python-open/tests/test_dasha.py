"""Tests for dasha (planetary period) computation."""

import pytest
from conftest import skip_no_kernels, skip_no_eop


# Birth data: Delhi, 2024-01-15 06:00 UTC
BIRTH_UTC = (2024, 1, 15, 6, 0, 0.0)
BIRTH_LOC = (28.6139, 77.2090)


class TestDashaVariationConfigStruct:
    def test_cycles_and_min_span_years_fields(self):
        """Variation/selection configs expose level-0 cycle repetition fields."""
        try:
            from ctara_dhruv.dasha import (
                _make_variation_config,
                dasha_selection_config_default,
                dasha_variation_config_default,
            )
        except Exception:
            pytest.skip("native library not available")

        default = dasha_variation_config_default()
        assert default.cycles == 0
        assert default.min_span_years == 0.0

        selection = dasha_selection_config_default()
        assert selection.cycles == 0
        assert selection.min_span_years == 0.0

        cfg = _make_variation_config({"cycles": 3, "min_span_years": 250.0})
        assert cfg.cycles == 3
        assert cfg.min_span_years == 250.0

        cfg = _make_variation_config({"use_abhijit": True})
        assert cfg.cycles == default.cycles
        assert cfg.min_span_years == default.min_span_years


@skip_no_kernels
@skip_no_eop
class TestDashaHierarchy:
    def test_vimshottari_hierarchy(self, engine_handles):
        """Vimshottari dasha hierarchy should have at least 2 levels."""
        from ctara_dhruv.dasha import dasha_hierarchy
        from ctara_dhruv.engine import engine, lsk, eop
        result = dasha_hierarchy(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            system=0,  # Vimshottari
            max_level=2,
        )
        assert len(result.levels) >= 2

    def test_vimshottari_level0_has_9_periods(self, engine_handles):
        """Vimshottari Maha Dasha should have 9 periods."""
        from ctara_dhruv.dasha import dasha_hierarchy
        from ctara_dhruv.engine import engine, lsk, eop
        result = dasha_hierarchy(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            system=0,
            max_level=0,
        )
        level0 = result.levels[0]
        assert level0.level == 0
        assert len(level0.periods) == 9
        # Periods should be contiguous
        for i in range(len(level0.periods) - 1):
            p = level0.periods[i]
            n = level0.periods[i + 1]
            assert abs(p.end_jd - n.start_jd) < 0.01

    def test_hierarchy_period_fields(self, engine_handles):
        """Each period should have valid entity and JD fields."""
        from ctara_dhruv.dasha import dasha_hierarchy
        from ctara_dhruv.engine import engine, lsk, eop
        result = dasha_hierarchy(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            system=0,
            max_level=1,
        )
        for level in result.levels:
            for p in level.periods:
                assert p.entity_type == 0  # Graha for Vimshottari
                assert 0 <= p.entity_index <= 8
                assert p.entity_name is not None
                assert len(p.entity_name) > 0
                assert p.start_jd < p.end_jd
                assert p.level == level.level


@skip_no_kernels
@skip_no_eop
class TestDashaSnapshot:
    def test_vimshottari_snapshot(self, engine_handles):
        """Snapshot at query time should return active periods."""
        from ctara_dhruv.dasha import dasha_snapshot
        from ctara_dhruv.engine import engine, lsk, eop
        snap = dasha_snapshot(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            jd_utc_query=(2025, 1, 1, 0, 0, 0.0),
            location=BIRTH_LOC,
            system=0,
            max_level=2,
        )
        assert snap.system == 0
        assert len(snap.periods) >= 1
        # Each period should contain the query JD
        for p in snap.periods:
            assert p.start_jd <= snap.query_jd <= p.end_jd


@skip_no_kernels
@skip_no_eop
class TestLowTierDasha:
    def test_low_tier_vimshottari_functions(self, engine_handles):
        from ctara_dhruv.dasha import (
            dasha_variation_config_default,
            dasha_level0,
            dasha_level0_entity,
            dasha_children,
            dasha_child_period,
            dasha_complete_level,
        )
        from ctara_dhruv.engine import engine, lsk, eop

        level0 = dasha_level0(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            system=0,
        )
        assert len(level0) == 9

        first = level0[0]
        same = dasha_level0_entity(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            entity={"type": first.entity_type, "index": first.entity_index},
            system=0,
        )
        assert same is not None
        assert same.entity_index == first.entity_index

        variation = dasha_variation_config_default()
        children = dasha_children(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            parent=first,
            system=0,
            variation_config={
                "level_methods": list(variation.level_methods),
                "yogini_scheme": variation.yogini_scheme,
                "use_abhijit": bool(variation.use_abhijit),
            },
        )
        assert len(children) == 9

        child = dasha_child_period(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            parent=first,
            entity={"type": children[0].entity_type, "index": children[0].entity_index},
            system=0,
        )
        assert child is not None
        assert child.entity_index == children[0].entity_index

        complete = dasha_complete_level(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            parent_periods=level0,
            child_level=1,
            system=0,
        )
        assert len(complete) == len(level0) * len(children)

    def test_level0_cycle_repetition(self, engine_handles):
        """cycles=2 should repeat the Vimshottari level-0 sequence twice."""
        from ctara_dhruv.dasha import dasha_level0
        from ctara_dhruv.engine import engine, lsk, eop

        repeated = dasha_level0(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            system=0,
            variation_config={"cycles": 2},
        )
        assert len(repeated) == 18
        # Second cycle repeats the same entity sequence, contiguously.
        for i in range(9):
            assert repeated[i + 9].entity_index == repeated[i].entity_index
        for i in range(len(repeated) - 1):
            assert abs(repeated[i].end_jd - repeated[i + 1].start_jd) < 0.01
        # Cycle number derives from the global order field.
        assert (repeated[0].order - 1) // 9 + 1 == 1
        assert (repeated[17].order - 1) // 9 + 1 == 2

    def test_level0_min_span_years(self, engine_handles):
        """min_span_years should extend level-0 coverage past the target."""
        from ctara_dhruv.dasha import dasha_level0
        from ctara_dhruv.engine import engine, lsk, eop

        periods = dasha_level0(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            system=0,
            variation_config={"min_span_years": 150.0},
        )
        # Vimshottari cycle is 120y, so 150y coverage needs two whole cycles.
        assert len(periods) == 18
        assert (periods[-1].end_jd - periods[0].start_jd) / 365.25 >= 150.0


@skip_no_kernels
@skip_no_eop
class TestRashiDasha:
    def test_chara_dasha(self, engine_handles):
        """Chara dasha (system 11) should have 12 Maha periods (rashi-based)."""
        from ctara_dhruv.dasha import dasha_hierarchy
        from ctara_dhruv.engine import engine, lsk, eop
        result = dasha_hierarchy(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            system=11,  # Chara
            max_level=0,
        )
        level0 = result.levels[0]
        assert len(level0.periods) == 12
        for p in level0.periods:
            assert p.entity_type == 1  # Rashi
            assert 0 <= p.entity_index <= 11
            assert p.entity_name is not None
            assert len(p.entity_name) > 0

    def test_yogini_dasha_names(self, engine_handles):
        """Yogini dasha should surface exact canonical Yogini names."""
        from ctara_dhruv.dasha import dasha_hierarchy
        from ctara_dhruv.engine import engine, lsk, eop
        result = dasha_hierarchy(
            engine(), lsk(), eop(),
            jd_utc_birth=BIRTH_UTC,
            location=BIRTH_LOC,
            system=10,  # Yogini
            max_level=0,
        )
        names = [period.entity_name for period in result.levels[0].periods]
        assert all(name for name in names)
        assert any(name == "Mangala" for name in names)
