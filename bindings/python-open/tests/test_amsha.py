"""Tests for amsha (divisional chart) computation."""

import pytest
from conftest import skip_no_kernels, skip_no_eop


class TestAmshaLongitudePureMath:
    def test_d1_identity(self):
        """D1 (rashi chart) should return the same longitude."""
        from ctara_dhruv.amsha import amsha_longitude
        lon = amsha_longitude(45.0, 1)
        assert abs(lon - 45.0) < 0.01

    def test_d9_navamsha(self):
        """D9 Navamsha: 45 deg should map to a valid amsha longitude."""
        from ctara_dhruv.amsha import amsha_longitude
        result = amsha_longitude(45.0, 9)
        assert 0 <= result < 360

    def test_d9_navamsha_boundary(self):
        """D9: 0 deg should map to 0 deg (Mesha, first navamsha)."""
        from ctara_dhruv.amsha import amsha_longitude
        result = amsha_longitude(0.0, 9)
        assert abs(result) < 0.01 or abs(result - 360.0) < 0.01

    def test_d12_dwadashamsha(self):
        """D12: should produce valid longitude."""
        from ctara_dhruv.amsha import amsha_longitude
        result = amsha_longitude(100.0, 12)
        assert 0 <= result < 360

    def test_d60_shastiamsha(self):
        """D60: should produce valid longitude."""
        from ctara_dhruv.amsha import amsha_longitude
        result = amsha_longitude(200.0, 60)
        assert 0 <= result < 360

    def test_new_d2_hora_variations(self):
        """D2 Hora variations should be accepted by numeric code."""
        from ctara_dhruv.amsha import amsha_longitude
        assert abs(amsha_longitude(1.25, 2, 2) - 135.0) < 0.01
        assert abs(amsha_longitude(20.0, 2, 3) - 220.0) < 0.01


class TestAmshaRashiInfo:
    def test_amsha_rashi_info_d9(self):
        """Rashi info for D9 should be valid."""
        from ctara_dhruv.amsha import amsha_rashi_info
        ri = amsha_rashi_info(45.0, 9)
        assert 0 <= ri.rashi_index <= 11
        assert 0 <= ri.degrees_in_rashi < 30


class TestAmshaLongitudesBatch:
    def test_batch_multiple_codes(self):
        """Batch computation for multiple D-codes."""
        from ctara_dhruv.amsha import amsha_longitudes
        results = amsha_longitudes(45.0, [1, 9, 12])
        assert len(results) == 3
        for lon in results:
            assert 0 <= lon < 360

    def test_amsha_variation_catalogs(self):
        """Variation discovery should be scoped by amsha."""
        from ctara_dhruv.amsha import amsha_variations, amsha_variations_many

        d2 = amsha_variations(2)
        assert d2.amsha_code == 2
        assert d2.default_variation_code == 0
        assert [entry.name for entry in d2.variations] == [
            "default",
            "cancer-leo-only",
            "lunar-hora",
            "kashinath-hora",
        ]

        many = amsha_variations_many([2, 9])
        assert len(many) == 2
        assert many[1].amsha_code == 9
        assert len(many[1].variations) == 1
        assert many[1].variations[0].variation_code == 0


@skip_no_kernels
@skip_no_eop
class TestAmshaChartForDate:
    def test_d9_chart_for_date(self, engine_handles):
        """Compute D9 chart for a birth date."""
        from ctara_dhruv.amsha import amsha_chart_for_date
        from ctara_dhruv.engine import engine, lsk, eop
        chart = amsha_chart_for_date(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            amsha_code=9,
        )
        assert chart.amsha_code == 9
        assert len(chart.grahas) == 9
        assert chart.outer_planets is not None
        assert len(chart.outer_planets) == 3
        for g in chart.grahas:
            assert 0 <= g.sidereal_longitude < 360
            assert 0 <= g.rashi_index <= 11
        assert 0 <= chart.lagna.sidereal_longitude < 360

    def test_chart_for_date_with_optional_scope_sections(self, engine_handles):
        """Amsha chart should expose optional scoped sections when requested."""
        from ctara_dhruv.amsha import amsha_chart_for_date
        from ctara_dhruv.engine import engine, lsk, eop

        chart = amsha_chart_for_date(
            engine(), lsk(), eop(),
            jd_utc=(2024, 1, 15, 6, 0, 0.0),
            location=(28.6139, 77.2090),
            amsha_code=9,
            scope={
                "include_bhava_cusps": 1,
                "include_arudha_padas": 1,
                "include_upagrahas": 1,
                "include_sphutas": 1,
                "include_special_lagnas": 1,
            },
        )

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
        assert chart.outer_planets is not None
        assert len(chart.outer_planets) == 3


@skip_no_kernels
@skip_no_eop
class TestAmshaSeries:
    _DELHI = (28.6139, 77.2090)

    def test_three_point_smoke_matches_single_epoch(self, engine_handles):
        """3-point series should match single-epoch amsha charts exactly.

        The default sankranti config uses ayanamsha 0 (Lahiri) without
        nutation, so compare against amsha_chart_for_date(use_nutation=0).
        """
        from ctara_dhruv.amsha import amsha_series, amsha_chart_for_date
        from ctara_dhruv.engine import engine, lsk, eop

        points = amsha_series(
            engine(), eop(),
            (2024, 1, 15, 0, 0, 0.0), (2024, 1, 15, 2, 0, 0.0),
            60, self._DELHI, [9],
        )
        assert len(points) == 3
        for p in points:
            assert len(p.charts) == 1
            chart = p.charts[0]
            assert chart.amsha_code == 9
            assert chart.variation_code == 0
            assert chart.grahas is not None
            assert len(chart.grahas) == 9

            single = amsha_chart_for_date(
                engine(), lsk(), eop(), p.utc, self._DELHI, 9, use_nutation=0
            )
            assert chart.lagna.rashi_index == single.lagna.rashi_index
            assert abs(
                chart.lagna.sidereal_longitude - single.lagna.sidereal_longitude
            ) < 1e-9
            for g, sg in zip(chart.grahas, single.grahas):
                assert abs(g.sidereal_longitude - sg.sidereal_longitude) < 1e-9

    def test_series_without_grahas(self, engine_handles):
        """include_grahas=False should omit graha entries but keep lagna."""
        from ctara_dhruv.amsha import amsha_series
        from ctara_dhruv.engine import engine, eop

        points = amsha_series(
            engine(), eop(),
            (2024, 1, 15, 0, 0, 0.0), (2024, 1, 15, 1, 0, 0.0),
            60, self._DELHI, [1, 9], include_grahas=False,
        )
        assert len(points) == 2
        for p in points:
            assert [c.amsha_code for c in p.charts] == [1, 9]
            for c in p.charts:
                assert c.grahas is None
                assert 0 <= c.lagna.sidereal_longitude < 360

    def test_empty_request_raises(self, engine_handles):
        from ctara_dhruv import DhruvError
        from ctara_dhruv.amsha import amsha_series
        from ctara_dhruv.engine import engine, eop

        with pytest.raises(DhruvError):
            amsha_series(
                engine(), eop(),
                (2024, 1, 15, 0, 0, 0.0), (2024, 1, 15, 1, 0, 0.0),
                60, self._DELHI, [],
            )


@skip_no_kernels
@skip_no_eop
class TestAmshaLagnaEvents:
    _DELHI = (28.6139, 77.2090)

    def test_d1_smoke_and_segment_chaining(self, engine_handles):
        """D1 lagna segments over one day chain exactly and advance by rashi."""
        from ctara_dhruv.amsha import amsha_lagna_events
        from ctara_dhruv.engine import engine, eop

        result = amsha_lagna_events(
            engine(), eop(),
            (2024, 1, 15, 0, 0, 0.0), (2024, 1, 16, 0, 0, 0.0),
            self._DELHI, [1],
        )
        assert not result.truncated
        assert result.next_from is None
        assert len(result.entries) == 1
        entry = result.entries[0]
        assert entry.amsha_code == 1
        assert entry.variation_code == 0

        segments = entry.segments
        # The D1 lagna passes through all 12 rashis in a sidereal day.
        assert len(segments) >= 12
        first = segments[0]
        assert (first.start.year, first.start.month, first.start.day) == (2024, 1, 15)
        for a, b in zip(segments, segments[1:]):
            assert a.end == b.start
            assert (a.rashi_index + 1) % 12 == b.rashi_index
        for s in segments:
            assert 0 <= s.rashi_index <= 11
            assert s.end.to_datetime() > s.start.to_datetime()

    def test_duplicate_requests_collapse(self, engine_handles):
        """Duplicate (amsha, variation) requests should collapse to one entry."""
        from ctara_dhruv.amsha import amsha_lagna_events
        from ctara_dhruv.engine import engine, eop

        result = amsha_lagna_events(
            engine(), eop(),
            (2024, 1, 15, 0, 0, 0.0), (2024, 1, 15, 6, 0, 0.0),
            self._DELHI, [1, 1, 9],
        )
        assert [(e.amsha_code, e.variation_code) for e in result.entries] == [
            (1, 0), (9, 0),
        ]

    def test_empty_request_raises(self, engine_handles):
        from ctara_dhruv import DhruvError
        from ctara_dhruv.amsha import amsha_lagna_events
        from ctara_dhruv.engine import engine, eop

        with pytest.raises(DhruvError):
            amsha_lagna_events(
                engine(), eop(),
                (2024, 1, 15, 0, 0, 0.0), (2024, 1, 16, 0, 0, 0.0),
                self._DELHI, [],
            )
