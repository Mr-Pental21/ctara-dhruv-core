"""Tests for ephemeris position queries."""

import pytest
from conftest import skip_no_kernels


J2000 = 2451545.0


@skip_no_kernels
class TestQueryState:
    def test_mars_position_at_j2000(self, engine_handles):
        """Mars (499) relative to SSB at J2000 should be non-zero."""
        from ctara_dhruv.ephemeris import query_state
        state = query_state(engine_handles._ptr, target=499, observer=0, jd_tdb=J2000)
        assert abs(state.x) > 1e3
        assert abs(state.y) > 1e3
        assert abs(state.z) > 1e3

    def test_mars_velocity_at_j2000(self, engine_handles):
        """Mars should have non-zero velocity at J2000."""
        from ctara_dhruv.ephemeris import query_state
        state = query_state(engine_handles._ptr, target=499, observer=0, jd_tdb=J2000)
        assert abs(state.vx) > 0
        assert abs(state.vy) > 0

    def test_sun_relative_to_ssb(self, engine_handles):
        """Sun (10) relative to SSB (0) should be close to origin (< 0.03 AU)."""
        from ctara_dhruv.ephemeris import query_state
        import math
        state = query_state(engine_handles._ptr, target=10, observer=0, jd_tdb=J2000)
        dist_km = math.sqrt(state.x**2 + state.y**2 + state.z**2)
        au_km = 1.496e8
        assert dist_km / au_km < 0.03

    def test_earth_distance_from_sun(self, engine_handles):
        """Earth-Sun distance at J2000 should be ~1 AU."""
        from ctara_dhruv.ephemeris import query_state
        import math
        state = query_state(engine_handles._ptr, target=399, observer=10, jd_tdb=J2000)
        dist_km = math.sqrt(state.x**2 + state.y**2 + state.z**2)
        au_km = 1.496e8
        assert 0.98 < dist_km / au_km < 1.02


@skip_no_kernels
class TestQueryOnce:
    def test_query_once(self, bsp_path, lsk_path):
        """One-shot query should return valid position."""
        from ctara_dhruv.ephemeris import query_once
        state = query_once([bsp_path], lsk_path, target=499, observer=0, jd_tdb=J2000)
        assert abs(state.x) > 1e3


@skip_no_kernels
class TestQueryUtc:
    def test_query_utc_spherical(self, engine_handles):
        """UTC spherical query should return valid lon/lat/distance."""
        from ctara_dhruv.ephemeris import query_utc_spherical
        ss = query_utc_spherical(
            engine_handles._ptr,
            target=499, observer=10, frame=1,
            year=2024, month=1, day=1, hour=12,
        )
        assert 0 <= ss.lon_deg < 360
        assert -90 <= ss.lat_deg <= 90
        assert ss.distance_km > 0

    def test_query_utc_with_utctime(self, engine_handles):
        """UTC query with UtcTime should return valid spherical state."""
        from ctara_dhruv.ephemeris import query_utc
        from ctara_dhruv.types import UtcTime
        utc = UtcTime(2024, 1, 1, 12, 0, 0.0)
        ss = query_utc(engine_handles._ptr, target=499, observer=10, utc_time=utc)
        assert ss.distance_km > 0


@skip_no_kernels
class TestCoordinateConversion:
    def test_body_ecliptic_lon_lat(self, engine_handles):
        """Ecliptic lon/lat for Sun should be near zero latitude."""
        from ctara_dhruv.ephemeris import body_ecliptic_lon_lat
        lon, lat = body_ecliptic_lon_lat(engine_handles._ptr, 10, J2000)
        assert 0 <= lon < 360
        assert abs(lat) < 1.0  # Sun near ecliptic plane

    def test_cartesian_to_spherical(self):
        """Convert a known Cartesian to spherical."""
        from ctara_dhruv.ephemeris import cartesian_to_spherical
        import math
        sc = cartesian_to_spherical(1.0, 0.0, 0.0)
        assert abs(sc.lon_deg - 0.0) < 0.01
        assert abs(sc.lat_deg - 0.0) < 0.01
        assert abs(sc.distance_km - 1.0) < 0.01

    def test_cartesian_to_spherical_z_axis(self):
        """Point on +Z axis should have lat=90."""
        from ctara_dhruv.ephemeris import cartesian_to_spherical
        sc = cartesian_to_spherical(0.0, 0.0, 100.0)
        assert abs(sc.lat_deg - 90.0) < 0.01
        assert abs(sc.distance_km - 100.0) < 0.01
