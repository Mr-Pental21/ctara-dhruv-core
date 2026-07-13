"""Tests for panchang computation."""

import pytest
from conftest import skip_no_kernels, skip_no_eop


@skip_no_kernels
@skip_no_eop
class TestPanchangCompute:
    def test_panchang_basic(self, engine_handles):
        """Compute panchang at Delhi for 2024-01-15 with core elements."""
        from ctara_dhruv.panchang import panchang, INCLUDE_ALL_CORE
        from ctara_dhruv.types import UtcTime, GeoLocation
        from ctara_dhruv.engine import engine, lsk, eop
        utc = UtcTime(2024, 1, 15, 12, 0, 0.0)
        delhi = GeoLocation(lat_deg=28.6139, lon_deg=77.2090)
        result = panchang(
            engine()._ptr, eop(), lsk(), utc, delhi,
            include_mask=INCLUDE_ALL_CORE,
        )
        assert result.tithi is not None
        assert 0 <= result.tithi.tithi_index <= 29
        assert result.karana is not None
        assert result.yoga is not None
        assert 0 <= result.yoga.yoga_index <= 26
        assert result.vaar is not None
        assert 0 <= result.vaar.vaar_index <= 6
        assert result.nakshatra is not None
        assert 0 <= result.nakshatra.nakshatra_index <= 26

    def test_panchang_with_calendar(self, engine_handles):
        """Compute panchang with calendar elements (masa, ayana, varsha)."""
        from ctara_dhruv.panchang import panchang, INCLUDE_ALL
        from ctara_dhruv.types import UtcTime, GeoLocation
        from ctara_dhruv.engine import engine, lsk, eop
        utc = UtcTime(2024, 6, 15, 12, 0, 0.0)
        delhi = GeoLocation(lat_deg=28.6139, lon_deg=77.2090)
        result = panchang(
            engine()._ptr, eop(), lsk(), utc, delhi,
            include_mask=INCLUDE_ALL,
        )
        assert result.masa is not None
        assert 0 <= result.masa.masa_index <= 11
        assert result.ayana is not None
        assert result.ayana.ayana in (0, 1)

    def test_panchang_from_jd(self, engine_handles):
        """Compute panchang from JD TDB float input."""
        from ctara_dhruv.panchang import panchang, INCLUDE_TITHI
        from ctara_dhruv.types import GeoLocation
        from ctara_dhruv.engine import engine, lsk, eop
        delhi = GeoLocation(lat_deg=28.6139, lon_deg=77.2090)
        result = panchang(
            engine()._ptr, eop(), lsk(), 2460310.5, delhi,
            include_mask=INCLUDE_TITHI,
        )
        assert result.tithi is not None


class TestAbiVersion:
    def test_api_version_is_81(self):
        """Library and embedded header agree on ABI v80."""
        from ctara_dhruv._ffi import lib
        from ctara_dhruv._cdef import EXPECTED_API_VERSION
        assert EXPECTED_API_VERSION == 81
        assert lib.dhruv_api_version() == 81


@skip_no_kernels
@skip_no_eop
class TestKnownCalendarReuse:
    """known_masa/known_ayana/known_varsha caller-cache reuse (ABI v78)."""

    def _compute(self, utc, **known):
        from ctara_dhruv.panchang import panchang, INCLUDE_ALL_CALENDAR
        from ctara_dhruv.engine import engine, lsk, eop
        return panchang(
            engine()._ptr, eop(), lsk(), utc,
            include_mask=INCLUDE_ALL_CALENDAR,
            **known,
        )

    def test_known_values_reused_at_nearby_date(self, engine_handles):
        """Fed-back values inside their windows are returned verbatim."""
        from ctara_dhruv.types import UtcTime
        first = self._compute(UtcTime(2024, 6, 15, 12, 0, 0.0))
        assert first.masa is not None
        assert first.ayana is not None
        assert first.varsha is not None

        # One day later: well inside the masa/ayana/varsha windows.
        second = self._compute(
            UtcTime(2024, 6, 16, 12, 0, 0.0),
            known_masa=first.masa,
            known_ayana=first.ayana,
            known_varsha=first.varsha,
        )
        assert second.masa == first.masa
        assert second.ayana == first.ayana
        assert second.varsha == first.varsha

    def test_stale_known_values_recomputed_at_far_date(self, engine_handles):
        """Values whose windows do not cover the moment are ignored."""
        from ctara_dhruv.types import UtcTime
        first = self._compute(UtcTime(2024, 6, 15, 12, 0, 0.0))

        # Six months later: masa window no longer covers the moment.
        far = self._compute(
            UtcTime(2024, 12, 15, 12, 0, 0.0),
            known_masa=first.masa,
            known_ayana=first.ayana,
            known_varsha=first.varsha,
        )
        assert far.masa is not None
        assert far.masa.masa_index != first.masa.masa_index
        assert far.masa.start != first.masa.start
        # Recomputed result matches a plain (no known_*) computation.
        plain = self._compute(UtcTime(2024, 12, 15, 12, 0, 0.0))
        assert far.masa == plain.masa
        assert far.ayana == plain.ayana
        assert far.varsha == plain.varsha


@skip_no_kernels
@skip_no_eop
class TestIndividualPanchang:
    def test_tithi_for_date(self, engine_handles):
        from ctara_dhruv.panchang import tithi_for_date
        from ctara_dhruv.types import UtcTime
        from ctara_dhruv.engine import engine
        utc = UtcTime(2024, 1, 15, 12, 0, 0.0)
        t = tithi_for_date(engine()._ptr, utc)
        assert 0 <= t.tithi_index <= 29
        assert t.paksha in (0, 1)
        assert 1 <= t.tithi_in_paksha <= 15

    def test_karana_for_date(self, engine_handles):
        from ctara_dhruv.panchang import karana_for_date
        from ctara_dhruv.types import UtcTime
        from ctara_dhruv.engine import engine
        utc = UtcTime(2024, 1, 15, 12, 0, 0.0)
        k = karana_for_date(engine()._ptr, utc)
        assert 0 <= k.karana_index <= 59

    def test_yoga_for_date(self, engine_handles):
        from ctara_dhruv.panchang import yoga_for_date
        from ctara_dhruv.types import UtcTime
        from ctara_dhruv.engine import engine
        utc = UtcTime(2024, 1, 15, 12, 0, 0.0)
        y = yoga_for_date(engine()._ptr, utc)
        assert 0 <= y.yoga_index <= 26


@skip_no_kernels
class TestPanchangIntermediates:
    """Test JD-based composable intermediate functions."""

    def test_elongation_at(self, engine_handles):
        from ctara_dhruv.panchang import elongation_at
        from ctara_dhruv.engine import engine
        # Use J2000 epoch as a known valid JD
        elong = elongation_at(engine()._ptr, 2451545.0)
        assert -360 < elong < 360

    def test_tithi_at(self, engine_handles):
        from ctara_dhruv.panchang import elongation_at, tithi_at
        from ctara_dhruv.engine import engine
        jd = 2451545.0
        elong = elongation_at(engine()._ptr, jd)
        t = tithi_at(engine()._ptr, jd, elong)
        assert 0 <= t.tithi_index <= 29
        assert t.paksha in (0, 1)

    def test_karana_at(self, engine_handles):
        from ctara_dhruv.panchang import elongation_at, karana_at
        from ctara_dhruv.engine import engine
        jd = 2451545.0
        elong = elongation_at(engine()._ptr, jd)
        k = karana_at(engine()._ptr, jd, elong)
        assert 0 <= k.karana_index <= 59

    def test_yoga_at(self, engine_handles):
        from ctara_dhruv.panchang import yoga_at
        from ctara_dhruv.ayanamsha import sidereal_sum_at
        from ctara_dhruv.engine import engine
        jd = 2451545.0
        ssum = sidereal_sum_at(engine()._ptr, jd)
        y = yoga_at(engine()._ptr, jd, ssum)
        assert 0 <= y.yoga_index <= 26


@skip_no_kernels
@skip_no_eop
class TestFromSunrises:
    """Test pre-computed sunrise pair helpers."""

    def _get_sunrises(self, engine_handles):
        """Get actual sunrise pair for Delhi 2024-01-15."""
        from ctara_dhruv.vedic import vedic_day_sunrises
        from ctara_dhruv.types import UtcTime, GeoLocation
        from ctara_dhruv.engine import engine, eop
        delhi = GeoLocation(lat_deg=28.6139, lon_deg=77.2090)
        utc = UtcTime(2024, 1, 15, 6, 0, 0.0)
        return vedic_day_sunrises(engine()._ptr, eop(), utc, delhi)

    def test_vaar_from_sunrises(self, engine_handles):
        from ctara_dhruv.panchang import vaar_from_sunrises
        from ctara_dhruv.engine import lsk
        sr, nsr = self._get_sunrises(engine_handles)
        v = vaar_from_sunrises(lsk(), sr, nsr)
        assert 0 <= v.vaar_index <= 6

    def test_hora_from_sunrises(self, engine_handles):
        from ctara_dhruv.panchang import hora_from_sunrises
        from ctara_dhruv.engine import lsk
        sr, nsr = self._get_sunrises(engine_handles)
        query_jd = sr + 0.1  # ~2.4h after sunrise
        h = hora_from_sunrises(lsk(), query_jd, sr, nsr)
        assert 0 <= h.hora_index <= 6

    def test_ghatika_from_sunrises(self, engine_handles):
        from ctara_dhruv.panchang import ghatika_from_sunrises
        from ctara_dhruv.engine import lsk
        sr, nsr = self._get_sunrises(engine_handles)
        query_jd = sr + 0.1
        g = ghatika_from_sunrises(lsk(), query_jd, sr, nsr)
        assert g.value >= 0


class TestSamvatsara:
    def test_samvatsara_2024(self):
        """2024 CE should map to a valid 60-year cycle position."""
        from ctara_dhruv.panchang import samvatsara_from_year
        s = samvatsara_from_year(2024)
        assert 0 <= s.samvatsara_index <= 59
        assert 1 <= s.cycle_position <= 60

    def test_samvatsara_2000(self):
        from ctara_dhruv.panchang import samvatsara_from_year
        s = samvatsara_from_year(2000)
        assert 0 <= s.samvatsara_index <= 59


@skip_no_kernels
@skip_no_eop
class TestPanchangEvents:
    def test_tithi_chaining_35_days(self, engine_handles):
        """Tithi segments over ~35 days chain exactly and cover the range."""
        from ctara_dhruv.panchang import panchang_events, INCLUDE_TITHI
        from ctara_dhruv.types import UtcTime
        from ctara_dhruv.engine import engine, eop

        from_utc = UtcTime(2024, 1, 1, 0, 0, 0.0)
        to_utc = UtcTime(2024, 2, 5, 0, 0, 0.0)
        result = panchang_events(
            engine()._ptr, eop(), from_utc, to_utc, include_mask=INCLUDE_TITHI
        )
        assert not result.truncated
        assert result.next_from is None

        tithis = result.tithis
        # ~35 days of tithis (a tithi averages slightly under a day).
        assert len(tithis) >= 33
        # Kinds not selected stay empty.
        assert result.karanas == []
        assert result.yogas == []
        assert result.nakshatras == []

        # First segment may start before `from`, last may end after `to`.
        assert tithis[0].start.to_datetime() <= from_utc.to_datetime()
        assert tithis[-1].end.to_datetime() >= to_utc.to_datetime()

        # Segments chain exactly and the tithi index advances by 1 mod 30.
        for a, b in zip(tithis, tithis[1:]):
            assert a.end == b.start
            assert (a.tithi_index + 1) % 30 == b.tithi_index
        for t in tithis:
            assert 0 <= t.tithi_index <= 29
            assert t.paksha in (0, 1)

    def test_truncation_and_resume(self, engine_handles):
        """Truncated sweep resumes from next_from; dedup on start recovers all."""
        from ctara_dhruv.panchang import panchang_events, INCLUDE_TITHI
        from ctara_dhruv.types import UtcTime
        from ctara_dhruv.engine import engine, eop

        from_utc = UtcTime(2024, 1, 1, 0, 0, 0.0)
        to_utc = UtcTime(2024, 1, 20, 0, 0, 0.0)

        full = panchang_events(
            engine()._ptr, eop(), from_utc, to_utc, include_mask=INCLUDE_TITHI
        )
        assert not full.truncated

        first = panchang_events(
            engine()._ptr, eop(), from_utc, to_utc,
            include_mask=INCLUDE_TITHI, max_events=5,
        )
        assert first.truncated
        assert first.next_from is not None
        assert len(first.tithis) == 5

        rest = panchang_events(
            engine()._ptr, eop(), first.next_from, to_utc,
            include_mask=INCLUDE_TITHI,
        )
        seen = {t.start for t in first.tithis}
        merged = first.tithis + [t for t in rest.tithis if t.start not in seen]

        assert [t.tithi_index for t in merged] == [t.tithi_index for t in full.tithis]
        for a, b in zip(merged, full.tithis):
            assert abs((a.start.to_datetime() - b.start.to_datetime()).total_seconds()) < 1.0
            assert abs((a.end.to_datetime() - b.end.to_datetime()).total_seconds()) < 1.0

    def test_multi_kind_smoke(self, engine_handles):
        """A masa+ayana+nakshatra sweep populates each selected kind."""
        from ctara_dhruv.panchang import (
            panchang_events,
            INCLUDE_NAKSHATRA,
            INCLUDE_MASA,
            INCLUDE_AYANA,
        )
        from ctara_dhruv.types import UtcTime
        from ctara_dhruv.engine import engine, eop

        result = panchang_events(
            engine()._ptr, eop(),
            UtcTime(2024, 1, 1, 0, 0, 0.0), UtcTime(2024, 3, 1, 0, 0, 0.0),
            include_mask=INCLUDE_NAKSHATRA | INCLUDE_MASA | INCLUDE_AYANA,
        )
        assert not result.truncated
        assert len(result.nakshatras) >= 55  # ~1 nakshatra/day over 60 days
        assert len(result.masas) >= 2
        assert len(result.ayanas) >= 1
        assert result.tithis == []
        for a, b in zip(result.nakshatras, result.nakshatras[1:]):
            assert a.end == b.start

    def test_location_dependent_sweep(self, engine_handles):
        """3-day vaar+hora sweep with a location: counts, chaining, cycling."""
        from ctara_dhruv.panchang import (
            panchang_events,
            INCLUDE_VAAR,
            INCLUDE_HORA,
        )
        from ctara_dhruv.types import UtcTime, GeoLocation
        from ctara_dhruv.engine import engine, eop

        delhi = GeoLocation(lat_deg=28.6139, lon_deg=77.2090)
        result = panchang_events(
            engine()._ptr, eop(),
            UtcTime(2024, 1, 1, 0, 0, 0.0), UtcTime(2024, 1, 4, 0, 0, 0.0),
            include_mask=INCLUDE_VAAR | INCLUDE_HORA,
            location=delhi,
        )
        assert not result.truncated
        assert result.next_from is None

        # ~3 Vedic days: 3-5 vaar segments, 24 horas per vaar.
        vaars = result.vaars
        horas = result.horas
        assert 3 <= len(vaars) <= 5
        assert 72 <= len(horas) <= 120
        # Unselected kinds stay empty.
        assert result.tithis == []
        assert result.ghatikas == []

        # First segment may start before `from`, last may end after `to`.
        assert vaars[0].start.to_datetime() <= UtcTime(2024, 1, 1, 0, 0, 0.0).to_datetime()
        assert vaars[-1].end.to_datetime() >= UtcTime(2024, 1, 4, 0, 0, 0.0).to_datetime()

        # Vaar segments chain exactly (sunrise-to-sunrise Vedic days) and the
        # weekday advances by 1 mod 7.
        for a, b in zip(vaars, vaars[1:]):
            assert a.end == b.start
            assert (a.vaar_index + 1) % 7 == b.vaar_index
        for v in vaars:
            assert 0 <= v.vaar_index <= 6

        # Hora segments chain exactly, including across Vedic-day rolls, and
        # the 0-based hora position cycles 0..23 within each Vedic day.
        for a, b in zip(horas, horas[1:]):
            assert a.end == b.start
            assert b.hora_position == (a.hora_position + 1) % 24
        for h in horas:
            assert 0 <= h.hora_index <= 6
            assert 0 <= h.hora_position <= 23

    def test_vaar_without_location_raises(self, engine_handles):
        """Location-dependent bits without a location must be rejected."""
        from ctara_dhruv._check import InvalidSearchConfigError
        from ctara_dhruv.panchang import panchang_events, INCLUDE_VAAR
        from ctara_dhruv.types import UtcTime
        from ctara_dhruv.engine import engine, eop

        with pytest.raises(InvalidSearchConfigError):
            panchang_events(
                engine()._ptr, eop(),
                UtcTime(2024, 1, 1, 0, 0, 0.0), UtcTime(2024, 1, 2, 0, 0, 0.0),
                include_mask=INCLUDE_VAAR,
            )
