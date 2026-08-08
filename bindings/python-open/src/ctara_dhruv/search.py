"""Unified search APIs for conjunction, eclipse, motion, lunar phase, and sankranti.

All functions use the unified ``*_search_ex`` FFI entrypoints introduced in ABI v42.
"""

from __future__ import annotations

from typing import Optional

from ._ffi import ffi, lib
from ._check import check
from .types import (
    ConjunctionEvent,
    ChandraGrahanResult,
    EclipseGeoPoint,
    SuryaGrahanResult,
    SuryaGrahanPathPoint,
    SuryaGrahanFootprint,
    SuryaLocalGridSample,
    SuryaIsolineRing,
    SuryaRingSetLevel,
    SuryaIsolines,
    SuryaContactFootprint,
    SuryaUmbraFootprint,
    SuryaMagnitudeRing,
    StationaryEvent,
    MaxSpeedEvent,
    LunarPhaseEvent,
    SankrantiEvent,
    FixedLongitudeEvent,
    MasaInfo,
    UtcTime,
    GeoLocation,
    GocharNatalTarget,
    GocharReference,
    GocharEventWindow,
    TajakaReturnEvent,
    TithiPraveshaEvent,
    TransitToNatalAspectEvent,
    GocharEventsResult,
)
from .kundali import (
    _make_utc,
    _make_location,
    _make_bhava_config,
    _make_riseset_config,
    _make_sankranti_config,
    full_kundali_config_default,
    _extract_full_kundali_result_ffi,
)

_SEARCH_TIME_JD_TDB = 0
_SEARCH_TIME_UTC = 1
_JD_ABSENT = -1.0

# Maximum angles per multi-angle conjunction sweep
# (matches DHRUV_MAX_CONJUNCTION_TARGETS in the C ABI).
MAX_CONJUNCTION_TARGETS = 16

# Maximum angle offsets per fixed-longitude request
# (matches DHRUV_MAX_FIXED_LONGITUDE_ANGLES in the C ABI).
MAX_FIXED_LONGITUDE_ANGLES = 16


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


def _normalize_search_capacity(capacity: int) -> int:
    return max(1, int(capacity))


def _collect_full_range(fetch, initial_capacity: int):
    capacity = _normalize_search_capacity(initial_capacity)
    while True:
        items, count = fetch(capacity)
        if count < capacity:
            return items
        capacity *= 2


def _utc_from_c(u) -> UtcTime:
    """Convert a DhruvUtcTime C struct to a Python UtcTime."""
    return UtcTime(
        year=u.year,
        month=u.month,
        day=u.day,
        hour=u.hour,
        minute=u.minute,
        second=u.second,
    )


def _utc_struct(utc: UtcTime):
    out = ffi.new("DhruvUtcTime *")
    out.year = utc.year
    out.month = utc.month
    out.day = utc.day
    out.hour = utc.hour
    out.minute = utc.minute
    out.second = utc.second
    return out


def _set_single_search_time(req, when, *, arg_name: str) -> None:
    if isinstance(when, UtcTime):
        req.time_kind = _SEARCH_TIME_UTC
        req.at_utc = _utc_struct(when)[0]
        return
    if when is None:
        raise ValueError(f"{arg_name} is required")
    req.time_kind = _SEARCH_TIME_JD_TDB
    req.at_jd_tdb = float(when)


def _set_range_search_time(req, start, end, *, start_name: str, end_name: str) -> None:
    if start is None or end is None:
        missing = start_name if start is None else end_name
        raise ValueError(f"{missing} is required")
    start_is_utc = isinstance(start, UtcTime)
    end_is_utc = isinstance(end, UtcTime)
    if start_is_utc != end_is_utc:
        raise TypeError(f"{start_name} and {end_name} must use the same time input form")
    if start_is_utc:
        req.time_kind = _SEARCH_TIME_UTC
        req.start_utc = _utc_struct(start)[0]
        req.end_utc = _utc_struct(end)[0]
        return
    req.time_kind = _SEARCH_TIME_JD_TDB
    req.start_jd_tdb = float(start)
    req.end_jd_tdb = float(end)


def _coerce_sankranti_config(config):
    """Coerce a ``DhruvSankrantiConfig`` value, pointer, or dict to a struct value."""
    if isinstance(config, dict):
        return _make_sankranti_config(config)[0]
    if isinstance(config, ffi.CData) and ffi.typeof(config).kind == "pointer":
        return config[0]
    return config


def _set_sidereal_config(req, sidereal_config) -> None:
    """Fill the sidereal-echo request fields shared by conjunction/motion."""
    if sidereal_config is None:
        req.has_sidereal_config = 0
        return
    req.has_sidereal_config = 1
    req.sidereal_config = _coerce_sankranti_config(sidereal_config)


def _set_target_separations(req, target_separations) -> None:
    """Fill the multi-angle sweep fields (None/empty = single angle from config)."""
    if not target_separations:
        req.target_separation_count = 0
        return
    if len(target_separations) > MAX_CONJUNCTION_TARGETS:
        raise ValueError(
            f"target_separations supports at most {MAX_CONJUNCTION_TARGETS} angles"
        )
    req.target_separation_count = len(target_separations)
    for index, angle in enumerate(target_separations):
        req.target_separations_deg[index] = float(angle)


def _conjunction_event(e) -> ConjunctionEvent:
    has_sidereal = bool(e.has_sidereal)
    return ConjunctionEvent(
        utc=_utc_from_c(e.utc),
        jd_tdb=e.jd_tdb,
        actual_separation_deg=e.actual_separation_deg,
        body1_longitude_deg=e.body1_longitude_deg,
        body2_longitude_deg=e.body2_longitude_deg,
        body1_latitude_deg=e.body1_latitude_deg,
        body2_latitude_deg=e.body2_latitude_deg,
        body1_code=e.body1_code,
        body2_code=e.body2_code,
        target_separation_deg=e.target_separation_deg,
        has_sidereal=has_sidereal,
        body1_sidereal_longitude_deg=e.body1_sidereal_longitude_deg if has_sidereal else None,
        body2_sidereal_longitude_deg=e.body2_sidereal_longitude_deg if has_sidereal else None,
        body1_rashi_index=e.body1_rashi_index if has_sidereal else None,
        body2_rashi_index=e.body2_rashi_index if has_sidereal else None,
    )


def _chandra_grahan(r) -> ChandraGrahanResult:
    local_valid = bool(r.local_valid)

    def _contact_altitude(contact_jd, altitude_deg):
        """Altitude at a contact that may not exist for this eclipse."""
        if not local_valid or contact_jd == _JD_ABSENT:
            return None
        return altitude_deg

    return ChandraGrahanResult(
        grahan_type=r.grahan_type,
        magnitude=r.magnitude,
        penumbral_magnitude=r.penumbral_magnitude,
        greatest_grahan_utc=_utc_from_c(r.greatest_grahan_utc),
        greatest_grahan_jd=r.greatest_grahan_jd,
        p1_utc=_utc_from_c(r.p1_utc),
        p1_jd=r.p1_jd,
        u1_utc=None if r.u1_jd == _JD_ABSENT else _utc_from_c(r.u1_utc),
        u1_jd=r.u1_jd,
        u2_utc=None if r.u2_jd == _JD_ABSENT else _utc_from_c(r.u2_utc),
        u2_jd=r.u2_jd,
        u3_utc=None if r.u3_jd == _JD_ABSENT else _utc_from_c(r.u3_utc),
        u3_jd=r.u3_jd,
        u4_utc=None if r.u4_jd == _JD_ABSENT else _utc_from_c(r.u4_utc),
        u4_jd=r.u4_jd,
        p4_utc=_utc_from_c(r.p4_utc),
        p4_jd=r.p4_jd,
        moon_ecliptic_lat_deg=r.moon_ecliptic_lat_deg,
        angular_separation_deg=r.angular_separation_deg,
        moon_right_ascension_deg=r.moon_right_ascension_deg,
        moon_declination_deg=r.moon_declination_deg,
        local_visible=bool(r.local_visible) if local_valid else None,
        local_moon_altitude_deg=r.local_moon_altitude_deg,
        local_moon_azimuth_deg=r.local_moon_azimuth_deg,
        local_p1_altitude_deg=r.local_p1_altitude_deg,
        local_u1_altitude_deg=_contact_altitude(r.u1_jd, r.local_u1_altitude_deg),
        local_u2_altitude_deg=_contact_altitude(r.u2_jd, r.local_u2_altitude_deg),
        local_u3_altitude_deg=_contact_altitude(r.u3_jd, r.local_u3_altitude_deg),
        local_u4_altitude_deg=_contact_altitude(r.u4_jd, r.local_u4_altitude_deg),
        local_p4_altitude_deg=r.local_p4_altitude_deg,
        local_visible_start_utc=(None if r.local_visible_start_jd == _JD_ABSENT
                                 else _utc_from_c(r.local_visible_start_utc)),
        local_visible_start_jd=r.local_visible_start_jd,
        local_visible_end_utc=(None if r.local_visible_end_jd == _JD_ABSENT
                               else _utc_from_c(r.local_visible_end_utc)),
        local_visible_end_jd=r.local_visible_end_jd,
        local_visible_duration_seconds=r.local_visible_duration_seconds,
    )


def _magnitude_rings(geometry, footprint_index, ring_count, ring_at, ring_point_at) -> tuple:
    """Read the instantaneous iso-magnitude rings of one footprint."""
    rings = []
    for ring_index in range(int(ring_count)):
        raw_ring = ffi.new("DhruvSuryaMagnitudeRing *")
        check(ring_at(geometry, footprint_index, ring_index, raw_ring))
        boundary = []
        for point_index in range(int(raw_ring.point_count)):
            raw_point = ffi.new("DhruvEclipseGeoPoint *")
            check(ring_point_at(
                geometry, footprint_index, ring_index, point_index, raw_point
            ))
            boundary.append(EclipseGeoPoint(
                raw_point.latitude_deg, raw_point.longitude_deg
            ))
        rings.append(SuryaMagnitudeRing(
            level=float(raw_ring.level),
            boundary=tuple(boundary),
            contains_pole=int(raw_ring.contains_pole),
        ))
    return tuple(rings)


def _ring_set(geometry, set_kind) -> tuple:
    """Read one ring set (isoline levels or corridor segments)."""
    count = ffi.new("uint32_t *")
    check(lib.dhruv_surya_grahan_ring_set_level_count(geometry, set_kind, count))
    levels = []
    for level_index in range(int(count[0])):
        raw_level = ffi.new("DhruvSuryaRingSetLevel *")
        check(lib.dhruv_surya_grahan_ring_set_level_at(
            geometry, set_kind, level_index, raw_level
        ))
        rings = []
        for ring_index in range(int(raw_level.ring_count)):
            raw_ring = ffi.new("DhruvSuryaIsolineRing *")
            check(lib.dhruv_surya_grahan_ring_at(
                geometry, set_kind, level_index, ring_index, raw_ring
            ))
            boundary = []
            for point_index in range(int(raw_ring.point_count)):
                raw_point = ffi.new("DhruvEclipseGeoPoint *")
                check(lib.dhruv_surya_grahan_ring_point_at(
                    geometry, set_kind, level_index, ring_index, point_index, raw_point
                ))
                boundary.append(EclipseGeoPoint(
                    raw_point.latitude_deg, raw_point.longitude_deg
                ))
            rings.append(SuryaIsolineRing(
                contains_pole=int(raw_ring.contains_pole),
                boundary=tuple(boundary),
            ))
        levels.append(SuryaRingSetLevel(
            level_value=float(raw_level.level_value),
            grahan_type=int(raw_level.grahan_type),
            rings=tuple(rings),
        ))
    return tuple(levels)


def _surya_grahan(r) -> SuryaGrahanResult:
    path = []
    footprints = []
    local_grid = []
    isolines = None
    central_corridor = None
    contact_footprints = []
    umbra_footprints = []
    geometry = r.geometry_handle
    try:
        for index in range(int(r.path_count)):
            raw = ffi.new("DhruvSuryaGrahanPathPoint *")
            check(lib.dhruv_surya_grahan_path_point_at(geometry, index, raw))
            point = raw[0]
            path.append(SuryaGrahanPathPoint(
                jd_tdb=float(point.jd_tdb),
                utc=_utc_from_c(point.utc),
                center=EclipseGeoPoint(point.center.latitude_deg, point.center.longitude_deg),
                northern_limit=(EclipseGeoPoint(point.northern_limit.latitude_deg,
                                                point.northern_limit.longitude_deg)
                                if point.northern_limit_valid else None),
                southern_limit=(EclipseGeoPoint(point.southern_limit.latitude_deg,
                                                point.southern_limit.longitude_deg)
                                if point.southern_limit_valid else None),
                width_km=float(point.width_km),
                central_duration_seconds=float(point.central_duration_seconds),
                sun_altitude_deg=float(point.sun_altitude_deg),
                sun_azimuth_deg=float(point.sun_azimuth_deg),
                grahan_type=int(point.grahan_type),
            ))
        for footprint_index in range(int(r.footprint_count)):
            raw = ffi.new("DhruvSuryaGrahanFootprint *")
            check(lib.dhruv_surya_grahan_footprint_at(geometry, footprint_index, raw))
            footprint = raw[0]
            boundary = []
            for point_index in range(int(footprint.boundary_count)):
                raw_point = ffi.new("DhruvEclipseGeoPoint *")
                check(lib.dhruv_surya_grahan_footprint_point_at(
                    geometry, footprint_index, point_index, raw_point
                ))
                boundary.append(EclipseGeoPoint(
                    raw_point.latitude_deg, raw_point.longitude_deg
                ))
            footprints.append(SuryaGrahanFootprint(
                jd_tdb=float(footprint.jd_tdb),
                utc=_utc_from_c(footprint.utc),
                boundary=tuple(boundary),
                contains_pole=int(footprint.contains_pole),
                magnitude_rings=_magnitude_rings(
                    geometry, footprint_index, footprint.magnitude_ring_count,
                    lib.dhruv_surya_grahan_footprint_magnitude_ring_at,
                    lib.dhruv_surya_grahan_footprint_magnitude_ring_point_at,
                ),
            ))
        for contact_index in range(int(r.contact_footprint_count)):
            raw = ffi.new("DhruvSuryaContactFootprint *")
            check(lib.dhruv_surya_grahan_contact_footprint_at(geometry, contact_index, raw))
            footprint = raw[0]
            boundary = []
            for point_index in range(int(footprint.boundary_count)):
                raw_point = ffi.new("DhruvEclipseGeoPoint *")
                check(lib.dhruv_surya_grahan_contact_footprint_point_at(
                    geometry, contact_index, point_index, raw_point
                ))
                boundary.append(EclipseGeoPoint(
                    raw_point.latitude_deg, raw_point.longitude_deg
                ))
            contact_footprints.append(SuryaContactFootprint(
                contact=int(footprint.contact),
                jd_tdb=float(footprint.jd_tdb),
                utc=_utc_from_c(footprint.utc),
                boundary=tuple(boundary),
                contains_pole=int(footprint.contains_pole),
                magnitude_rings=_magnitude_rings(
                    geometry, contact_index, footprint.magnitude_ring_count,
                    lib.dhruv_surya_grahan_contact_magnitude_ring_at,
                    lib.dhruv_surya_grahan_contact_magnitude_ring_point_at,
                ),
            ))
        for umbra_index in range(int(r.umbra_footprint_count)):
            raw = ffi.new("DhruvSuryaUmbraFootprint *")
            check(lib.dhruv_surya_grahan_umbra_footprint_at(geometry, umbra_index, raw))
            footprint = raw[0]
            boundary = []
            for point_index in range(int(footprint.boundary_count)):
                raw_point = ffi.new("DhruvEclipseGeoPoint *")
                check(lib.dhruv_surya_grahan_umbra_footprint_point_at(
                    geometry, umbra_index, point_index, raw_point
                ))
                boundary.append(EclipseGeoPoint(
                    raw_point.latitude_deg, raw_point.longitude_deg
                ))
            umbra_footprints.append(SuryaUmbraFootprint(
                jd_tdb=float(footprint.jd_tdb),
                utc=_utc_from_c(footprint.utc),
                grahan_type=int(footprint.grahan_type),
                boundary=tuple(boundary),
                contains_pole=int(footprint.contains_pole),
            ))
        for sample_index in range(int(r.local_grid_count)):
            raw = ffi.new("DhruvSuryaLocalGridSample *")
            check(lib.dhruv_surya_grahan_local_grid_sample_at(geometry, sample_index, raw))
            sample = raw[0]
            local_grid.append(SuryaLocalGridSample(
                latitude_deg=float(sample.latitude_deg),
                longitude_deg=float(sample.longitude_deg),
                magnitude=float(sample.magnitude),
                obscuration=float(sample.obscuration),
                maximum_utc=_utc_from_c(sample.maximum_utc),
                maximum_jd=float(sample.maximum_jd),
                first_contact_utc=_utc_from_c(sample.first_contact_utc),
                first_contact_jd=float(sample.first_contact_jd),
                last_contact_utc=_utc_from_c(sample.last_contact_utc),
                last_contact_jd=float(sample.last_contact_jd),
                visible_duration_seconds=float(sample.visible_duration_seconds),
            ))
        if r.isolines_valid:
            isolines = SuryaIsolines(
                visibility_boundary=tuple(
                    ring
                    for level in _ring_set(geometry, RING_SET_VISIBILITY)
                    for ring in level.rings
                ),
                duration_isolines=_ring_set(geometry, RING_SET_DURATION),
                magnitude_isolines=_ring_set(geometry, RING_SET_MAGNITUDE),
            )
        if r.central_corridor_valid:
            central_corridor = _ring_set(geometry, RING_SET_CORRIDOR)
    finally:
        if geometry != ffi.NULL:
            lib.dhruv_surya_grahan_geometry_free(geometry)

    local_valid = bool(r.local_valid)
    return SuryaGrahanResult(
        grahan_type=r.grahan_type,
        magnitude=r.magnitude,
        greatest_grahan_utc=_utc_from_c(r.greatest_grahan_utc),
        greatest_grahan_jd=r.greatest_grahan_jd,
        c1_utc=None if r.c1_jd == _JD_ABSENT else _utc_from_c(r.c1_utc),
        c1_jd=r.c1_jd,
        c2_utc=None if r.c2_jd == _JD_ABSENT else _utc_from_c(r.c2_utc),
        c2_jd=r.c2_jd,
        c3_utc=None if r.c3_jd == _JD_ABSENT else _utc_from_c(r.c3_utc),
        c3_jd=r.c3_jd,
        c4_utc=None if r.c4_jd == _JD_ABSENT else _utc_from_c(r.c4_utc),
        c4_jd=r.c4_jd,
        moon_ecliptic_lat_deg=r.moon_ecliptic_lat_deg,
        angular_separation_deg=r.angular_separation_deg,
        sun_right_ascension_deg=r.sun_right_ascension_deg,
        sun_declination_deg=r.sun_declination_deg,
        obscuration=r.obscuration,
        apparent_diameter_ratio=r.apparent_diameter_ratio,
        gamma=r.gamma,
        greatest_location=(GeoLocation(r.greatest_latitude_deg, r.greatest_longitude_deg)
                           if r.greatest_location_valid else None),
        bessel_x=r.bessel_x,
        bessel_y=r.bessel_y,
        bessel_d_deg=r.bessel_d_deg,
        bessel_mu_deg=r.bessel_mu_deg,
        bessel_l1=r.bessel_l1,
        bessel_l2=r.bessel_l2,
        bessel_tan_f1=r.bessel_tan_f1,
        bessel_tan_f2=r.bessel_tan_f2,
        path_count=int(r.path_count),
        footprint_count=int(r.footprint_count),
        path=tuple(path),
        footprints=tuple(footprints),
        local_visible=bool(r.local_visible) if local_valid else None,
        local_grahan_type=(int(r.local_grahan_type)
                           if local_valid and r.local_grahan_type >= 0 else None),
        local_maximum_utc=(None if r.local_maximum_jd == _JD_ABSENT else _utc_from_c(r.local_maximum_utc)),
        local_maximum_jd=r.local_maximum_jd,
        local_c1_utc=None if r.local_c1_jd == _JD_ABSENT else _utc_from_c(r.local_c1_utc),
        local_c1_jd=r.local_c1_jd,
        local_c2_utc=None if r.local_c2_jd == _JD_ABSENT else _utc_from_c(r.local_c2_utc),
        local_c2_jd=r.local_c2_jd,
        local_c3_utc=None if r.local_c3_jd == _JD_ABSENT else _utc_from_c(r.local_c3_utc),
        local_c3_jd=r.local_c3_jd,
        local_c4_utc=None if r.local_c4_jd == _JD_ABSENT else _utc_from_c(r.local_c4_utc),
        local_c4_jd=r.local_c4_jd,
        local_magnitude=r.local_magnitude,
        local_obscuration=r.local_obscuration,
        local_sun_altitude_deg=r.local_sun_altitude_deg,
        local_sun_azimuth_deg=r.local_sun_azimuth_deg,
        local_central_duration_seconds=r.local_central_duration_seconds,
        local_first_visible_contact_utc=(None if r.local_first_visible_contact_jd == _JD_ABSENT
                                         else _utc_from_c(r.local_first_visible_contact_utc)),
        local_first_visible_contact_jd=r.local_first_visible_contact_jd,
        local_last_visible_contact_utc=(None if r.local_last_visible_contact_jd == _JD_ABSENT
                                        else _utc_from_c(r.local_last_visible_contact_utc)),
        local_last_visible_contact_jd=r.local_last_visible_contact_jd,
        local_visible_duration_seconds=r.local_visible_duration_seconds,
        centrality=int(r.centrality),
        local_grid=tuple(local_grid),
        isolines=isolines,
        central_corridor=central_corridor,
        contact_footprints=tuple(contact_footprints),
        umbra_footprints=tuple(umbra_footprints),
    )


def _stationary_event(e) -> StationaryEvent:
    has_sidereal = bool(e.has_sidereal)
    return StationaryEvent(
        utc=_utc_from_c(e.utc),
        jd_tdb=e.jd_tdb,
        body_code=e.body_code,
        longitude_deg=e.longitude_deg,
        latitude_deg=e.latitude_deg,
        station_type=e.station_type,
        has_sidereal=has_sidereal,
        sidereal_longitude_deg=e.sidereal_longitude_deg if has_sidereal else None,
        rashi_index=e.rashi_index if has_sidereal else None,
    )


def _max_speed_event(e) -> MaxSpeedEvent:
    has_sidereal = bool(e.has_sidereal)
    return MaxSpeedEvent(
        utc=_utc_from_c(e.utc),
        jd_tdb=e.jd_tdb,
        body_code=e.body_code,
        longitude_deg=e.longitude_deg,
        latitude_deg=e.latitude_deg,
        speed_deg_per_day=e.speed_deg_per_day,
        speed_type=e.speed_type,
        has_sidereal=has_sidereal,
        sidereal_longitude_deg=e.sidereal_longitude_deg if has_sidereal else None,
        rashi_index=e.rashi_index if has_sidereal else None,
    )


def _lunar_phase_event(e) -> LunarPhaseEvent:
    return LunarPhaseEvent(
        utc=_utc_from_c(e.utc),
        phase=e.phase,
        moon_longitude_deg=e.moon_longitude_deg,
        sun_longitude_deg=e.sun_longitude_deg,
    )


def _sankranti_event(e) -> SankrantiEvent:
    return SankrantiEvent(
        utc=_utc_from_c(e.utc),
        rashi_index=e.rashi_index,
        sun_sidereal_longitude_deg=e.sun_sidereal_longitude_deg,
        sun_tropical_longitude_deg=e.sun_tropical_longitude_deg,
        body_code=e.body_code,
        sidereal_longitude_deg=e.sidereal_longitude_deg,
        tropical_longitude_deg=e.tropical_longitude_deg,
        is_retrograde=bool(e.is_retrograde),
    )


def _masa_from_c(v) -> MasaInfo:
    return MasaInfo(
        masa_index=v.masa_index,
        adhika=bool(v.adhika),
        start=_utc_from_c(v.start),
        end=_utc_from_c(v.end),
    )


def gochar_events_config_default():
    cfg = lib.dhruv_gochar_events_config_default()
    return cfg


def _coerce_full_kundali_config_ptr(config):
    if config is None:
        return ffi.addressof(full_kundali_config_default())
    if isinstance(config, ffi.CData):
        if ffi.typeof(config) == ffi.typeof("DhruvFullKundaliConfig *"):
            return config
        return ffi.addressof(config)
    return config


# ---------------------------------------------------------------------------
# Conjunction search (dhruv_conjunction_search_ex)
# ---------------------------------------------------------------------------

# Query mode constants
_CONJUNCTION_NEXT = 0
_CONJUNCTION_PREV = 1
_CONJUNCTION_RANGE = 2


def conjunction_config_default():
    """Return default DhruvConjunctionConfig."""
    return lib.dhruv_conjunction_config_default()


def next_conjunction(
    engine,
    body1_code: int,
    body2_code: int,
    after_jd_tdb,
    config=None,
    target_separations: Optional[list[float]] = None,
    sidereal_config=None,
) -> Optional[ConjunctionEvent]:
    """Find next conjunction after a ``UtcTime`` or JD(TDB) anchor.

    Body codes accept NAIF codes plus 10007 (Rahu) / 10008 (Ketu).
    *target_separations*: optional list of target angles (deg, max
    ``MAX_CONJUNCTION_TARGETS``); ``None`` uses the single config angle.
    *sidereal_config*: optional ``DhruvSankrantiConfig`` (struct or dict);
    when given, events carry sidereal longitudes and rashi indices.
    """
    req = ffi.new("DhruvConjunctionSearchRequest *")
    req.body1_code = body1_code
    req.body2_code = body2_code
    req.query_mode = _CONJUNCTION_NEXT
    _set_single_search_time(req, after_jd_tdb, arg_name="after_jd_tdb")
    req.config = config if config is not None else lib.dhruv_conjunction_config_default()
    _set_target_separations(req, target_separations)
    _set_sidereal_config(req, sidereal_config)

    out_event = ffi.new("DhruvConjunctionEvent *")
    out_found = ffi.new("uint8_t *")
    check(
        lib.dhruv_conjunction_search_ex(
            engine, req, out_event, out_found,
            ffi.NULL, 0, ffi.NULL,
        ),
        "conjunction_search_ex(next)",
    )
    if out_found[0] == 0:
        return None
    return _conjunction_event(out_event[0])


def prev_conjunction(
    engine,
    body1_code: int,
    body2_code: int,
    before_jd_tdb,
    config=None,
    target_separations: Optional[list[float]] = None,
    sidereal_config=None,
) -> Optional[ConjunctionEvent]:
    """Find previous conjunction before a ``UtcTime`` or JD(TDB) anchor.

    See :func:`next_conjunction` for *target_separations* / *sidereal_config*.
    """
    req = ffi.new("DhruvConjunctionSearchRequest *")
    req.body1_code = body1_code
    req.body2_code = body2_code
    req.query_mode = _CONJUNCTION_PREV
    _set_single_search_time(req, before_jd_tdb, arg_name="before_jd_tdb")
    req.config = config if config is not None else lib.dhruv_conjunction_config_default()
    _set_target_separations(req, target_separations)
    _set_sidereal_config(req, sidereal_config)

    out_event = ffi.new("DhruvConjunctionEvent *")
    out_found = ffi.new("uint8_t *")
    check(
        lib.dhruv_conjunction_search_ex(
            engine, req, out_event, out_found,
            ffi.NULL, 0, ffi.NULL,
        ),
        "conjunction_search_ex(prev)",
    )
    if out_found[0] == 0:
        return None
    return _conjunction_event(out_event[0])


def search_conjunctions(
    engine,
    body1_code: int,
    body2_code: int,
    start_jd,
    end_jd,
    config=None,
    max_results: int = 100,
    target_separations: Optional[list[float]] = None,
    sidereal_config=None,
) -> list[ConjunctionEvent]:
    """Search for conjunctions in a UTC or JD(TDB) range.

    See :func:`next_conjunction` for *target_separations* / *sidereal_config*.
    """
    req = ffi.new("DhruvConjunctionSearchRequest *")
    req.body1_code = body1_code
    req.body2_code = body2_code
    req.query_mode = _CONJUNCTION_RANGE
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")
    req.config = config if config is not None else lib.dhruv_conjunction_config_default()
    _set_target_separations(req, target_separations)
    _set_sidereal_config(req, sidereal_config)

    def fetch(capacity: int):
        out_events = ffi.new("DhruvConjunctionEvent[]", capacity)
        out_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_conjunction_search_ex(
                engine, req,
                ffi.NULL, ffi.NULL,
                out_events, capacity, out_count,
            ),
            "conjunction_search_ex(range)",
        )
        count = int(out_count[0])
        return ([_conjunction_event(out_events[i]) for i in range(count)], count)

    return _collect_full_range(fetch, max_results)


# ---------------------------------------------------------------------------
# Eclipse search (dhruv_grahan_search_ex)
# ---------------------------------------------------------------------------

_GRAHAN_CHANDRA = 0
_GRAHAN_SURYA = 1
_GRAHAN_NEXT = 0
_GRAHAN_PREV = 1
_GRAHAN_RANGE = 2

# Ring-set selectors for surya isoline/corridor geometry.
RING_SET_VISIBILITY = 0
RING_SET_DURATION = 1
RING_SET_MAGNITUDE = 2
RING_SET_CORRIDOR = 3

# Surya centrality codes.
CENTRALITY_NONE = 0
CENTRALITY_PARTIAL = 1
CENTRALITY_FULL = 2


def grahan_config_default():
    """Return default DhruvGrahanConfig."""
    return lib.dhruv_grahan_config_default()


def grahan_config_effective(config):
    """Return the configuration actually applied after clamping/sanitizing.

    Build cache keys against this echo rather than the raw request.
    """
    out = ffi.new("DhruvGrahanConfig *")
    pointer = config if ffi.typeof(config).kind == "pointer" else ffi.addressof(config)
    check(lib.dhruv_grahan_config_effective(pointer, out))
    return out[0]


def _grahan_single(engine, grahan_kind: int, query_mode: int, when, config, location=None):
    """Internal: single grahan search (NEXT/PREV)."""
    req = ffi.new("DhruvGrahanSearchRequest *")
    req.grahan_kind = grahan_kind
    req.query_mode = query_mode
    _set_single_search_time(req, when, arg_name="jd")
    req.config = config if config is not None else lib.dhruv_grahan_config_default()
    if location is not None:
        req.location_valid = 1
        req.location.latitude_deg = location.lat_deg
        req.location.longitude_deg = location.lon_deg
        req.location.altitude_m = location.alt_m

    out_chandra = ffi.new("DhruvChandraGrahanResult *")
    out_surya = ffi.new("DhruvSuryaGrahanResult *")
    out_found = ffi.new("uint8_t *")
    check(
        lib.dhruv_grahan_search_ex(
            engine, req,
            out_chandra, out_surya, out_found,
            ffi.NULL, ffi.NULL, 0, ffi.NULL,
        ),
        "grahan_search_ex(single)",
    )
    if out_found[0] == 0:
        return None
    if grahan_kind == _GRAHAN_CHANDRA:
        return _chandra_grahan(out_chandra[0])
    return _surya_grahan(out_surya[0])


def next_lunar_eclipse(engine, after_jd, config=None) -> Optional[ChandraGrahanResult]:
    """Find the next lunar eclipse after a ``UtcTime`` or JD(TDB) anchor."""
    return _grahan_single(engine, _GRAHAN_CHANDRA, _GRAHAN_NEXT, after_jd, config)


def prev_lunar_eclipse(engine, before_jd, config=None) -> Optional[ChandraGrahanResult]:
    """Find the previous lunar eclipse before a ``UtcTime`` or JD(TDB) anchor."""
    return _grahan_single(engine, _GRAHAN_CHANDRA, _GRAHAN_PREV, before_jd, config)


def next_solar_eclipse(engine, after_jd, config=None, location=None) -> Optional[SuryaGrahanResult]:
    """Find the next solar eclipse after a ``UtcTime`` or JD(TDB) anchor."""
    return _grahan_single(engine, _GRAHAN_SURYA, _GRAHAN_NEXT, after_jd, config, location)


def prev_solar_eclipse(engine, before_jd, config=None, location=None) -> Optional[SuryaGrahanResult]:
    """Find the previous solar eclipse before a ``UtcTime`` or JD(TDB) anchor."""
    return _grahan_single(engine, _GRAHAN_SURYA, _GRAHAN_PREV, before_jd, config, location)


def search_lunar_eclipses(
    engine,
    start_jd,
    end_jd,
    config=None,
    max_results: int = 50,
) -> list[ChandraGrahanResult]:
    """Search for lunar eclipses in a UTC or JD(TDB) range."""
    req = ffi.new("DhruvGrahanSearchRequest *")
    req.grahan_kind = _GRAHAN_CHANDRA
    req.query_mode = _GRAHAN_RANGE
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")
    req.config = config if config is not None else lib.dhruv_grahan_config_default()

    def fetch(capacity: int):
        out_chandra = ffi.new("DhruvChandraGrahanResult[]", capacity)
        out_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_grahan_search_ex(
                engine, req,
                ffi.NULL, ffi.NULL, ffi.NULL,
                out_chandra, ffi.NULL, capacity, out_count,
            ),
            "grahan_search_ex(chandra_range)",
        )
        count = int(out_count[0])
        return ([_chandra_grahan(out_chandra[i]) for i in range(count)], count)

    return _collect_full_range(fetch, max_results)


def search_solar_eclipses(
    engine,
    start_jd,
    end_jd,
    config=None,
    max_results: int = 50,
    location=None,
) -> list[SuryaGrahanResult]:
    """Search for solar eclipses in a UTC or JD(TDB) range."""
    req = ffi.new("DhruvGrahanSearchRequest *")
    req.grahan_kind = _GRAHAN_SURYA
    req.query_mode = _GRAHAN_RANGE
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")
    req.config = config if config is not None else lib.dhruv_grahan_config_default()
    if location is not None:
        req.location_valid = 1
        req.location.latitude_deg = location.lat_deg
        req.location.longitude_deg = location.lon_deg
        req.location.altitude_m = location.alt_m

    def fetch(capacity: int):
        out_surya = ffi.new("DhruvSuryaGrahanResult[]", capacity)
        out_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_grahan_search_ex(
                engine, req,
                ffi.NULL, ffi.NULL, ffi.NULL,
                ffi.NULL, out_surya, capacity, out_count,
            ),
            "grahan_search_ex(surya_range)",
        )
        count = int(out_count[0])
        return ([_surya_grahan(out_surya[i]) for i in range(count)], count)

    return _collect_full_range(fetch, max_results)


# ---------------------------------------------------------------------------
# Motion search (dhruv_motion_search_ex)
# ---------------------------------------------------------------------------

_MOTION_STATIONARY = 0
_MOTION_MAX_SPEED = 1
_MOTION_NEXT = 0
_MOTION_PREV = 1
_MOTION_RANGE = 2


def stationary_config_default():
    """Return default DhruvStationaryConfig."""
    return lib.dhruv_stationary_config_default()


def _motion_single_stationary(engine, query_mode: int, body_code: int, when, config, sidereal_config=None):
    """Internal: single stationary search."""
    req = ffi.new("DhruvMotionSearchRequest *")
    req.body_code = body_code
    req.motion_kind = _MOTION_STATIONARY
    req.query_mode = query_mode
    _set_single_search_time(req, when, arg_name="jd")
    req.config = config if config is not None else lib.dhruv_stationary_config_default()
    _set_sidereal_config(req, sidereal_config)

    out_event = ffi.new("DhruvStationaryEvent *")
    out_found = ffi.new("uint8_t *")
    check(
        lib.dhruv_motion_search_ex(
            engine, req,
            out_event, ffi.NULL, out_found,
            ffi.NULL, ffi.NULL, 0, ffi.NULL,
        ),
        "motion_search_ex(stationary_single)",
    )
    if out_found[0] == 0:
        return None
    return _stationary_event(out_event[0])


def _motion_single_max_speed(engine, query_mode: int, body_code: int, when, config, sidereal_config=None):
    """Internal: single max-speed search."""
    req = ffi.new("DhruvMotionSearchRequest *")
    req.body_code = body_code
    req.motion_kind = _MOTION_MAX_SPEED
    req.query_mode = query_mode
    _set_single_search_time(req, when, arg_name="jd")
    req.config = config if config is not None else lib.dhruv_stationary_config_default()
    _set_sidereal_config(req, sidereal_config)

    out_event = ffi.new("DhruvMaxSpeedEvent *")
    out_found = ffi.new("uint8_t *")
    check(
        lib.dhruv_motion_search_ex(
            engine, req,
            ffi.NULL, out_event, out_found,
            ffi.NULL, ffi.NULL, 0, ffi.NULL,
        ),
        "motion_search_ex(max_speed_single)",
    )
    if out_found[0] == 0:
        return None
    return _max_speed_event(out_event[0])


def next_stationary(
    engine, body_code: int, after_jd, config=None, sidereal_config=None
) -> Optional[StationaryEvent]:
    """Find next stationary point after a ``UtcTime`` or JD(TDB) anchor.

    *body_code* accepts NAIF codes plus 10007 (Rahu) / 10008 (Ketu); node
    stations require the true node (``config.node_mode = 1``, the default).
    *sidereal_config*: optional ``DhruvSankrantiConfig`` (struct or dict);
    when given, events carry sidereal longitude and rashi index.
    """
    return _motion_single_stationary(engine, _MOTION_NEXT, body_code, after_jd, config, sidereal_config)


def prev_stationary(
    engine, body_code: int, before_jd, config=None, sidereal_config=None
) -> Optional[StationaryEvent]:
    """Find previous stationary point before a ``UtcTime`` or JD(TDB) anchor.

    See :func:`next_stationary` for *body_code* / *sidereal_config*.
    """
    return _motion_single_stationary(engine, _MOTION_PREV, body_code, before_jd, config, sidereal_config)


def search_stationary(
    engine,
    body_code: int,
    start_jd,
    end_jd,
    config=None,
    max_results: int = 100,
    sidereal_config=None,
) -> list[StationaryEvent]:
    """Search for stationary points in a UTC or JD(TDB) range.

    See :func:`next_stationary` for *body_code* / *sidereal_config*.
    """
    req = ffi.new("DhruvMotionSearchRequest *")
    req.body_code = body_code
    req.motion_kind = _MOTION_STATIONARY
    req.query_mode = _MOTION_RANGE
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")
    req.config = config if config is not None else lib.dhruv_stationary_config_default()
    _set_sidereal_config(req, sidereal_config)

    def fetch(capacity: int):
        out_events = ffi.new("DhruvStationaryEvent[]", capacity)
        out_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_motion_search_ex(
                engine, req,
                ffi.NULL, ffi.NULL, ffi.NULL,
                out_events, ffi.NULL, capacity, out_count,
            ),
            "motion_search_ex(stationary_range)",
        )
        count = int(out_count[0])
        return ([_stationary_event(out_events[i]) for i in range(count)], count)

    return _collect_full_range(fetch, max_results)


def next_max_speed(
    engine, body_code: int, after_jd, config=None, sidereal_config=None
) -> Optional[MaxSpeedEvent]:
    """Find next max-speed event after a ``UtcTime`` or JD(TDB) anchor.

    See :func:`next_stationary` for *body_code* / *sidereal_config*.
    """
    return _motion_single_max_speed(engine, _MOTION_NEXT, body_code, after_jd, config, sidereal_config)


def prev_max_speed(
    engine, body_code: int, before_jd, config=None, sidereal_config=None
) -> Optional[MaxSpeedEvent]:
    """Find previous max-speed event before a ``UtcTime`` or JD(TDB) anchor.

    See :func:`next_stationary` for *body_code* / *sidereal_config*.
    """
    return _motion_single_max_speed(engine, _MOTION_PREV, body_code, before_jd, config, sidereal_config)


def search_max_speeds(
    engine,
    body_code: int,
    start_jd,
    end_jd,
    config=None,
    max_results: int = 100,
    sidereal_config=None,
) -> list[MaxSpeedEvent]:
    """Search for max-speed events in a UTC or JD(TDB) range.

    See :func:`next_stationary` for *body_code* / *sidereal_config*.
    """
    req = ffi.new("DhruvMotionSearchRequest *")
    req.body_code = body_code
    req.motion_kind = _MOTION_MAX_SPEED
    req.query_mode = _MOTION_RANGE
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")
    req.config = config if config is not None else lib.dhruv_stationary_config_default()
    _set_sidereal_config(req, sidereal_config)

    def fetch(capacity: int):
        out_events = ffi.new("DhruvMaxSpeedEvent[]", capacity)
        out_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_motion_search_ex(
                engine, req,
                ffi.NULL, ffi.NULL, ffi.NULL,
                ffi.NULL, out_events, capacity, out_count,
            ),
            "motion_search_ex(max_speed_range)",
        )
        count = int(out_count[0])
        return ([_max_speed_event(out_events[i]) for i in range(count)], count)

    return _collect_full_range(fetch, max_results)


# ---------------------------------------------------------------------------
# Lunar phase search (dhruv_lunar_phase_search_ex)
# ---------------------------------------------------------------------------

_LUNAR_PHASE_AMAVASYA = 0
_LUNAR_PHASE_PURNIMA = 1
_LUNAR_PHASE_NEXT = 0
_LUNAR_PHASE_PREV = 1
_LUNAR_PHASE_RANGE = 2


def _lunar_phase_single(engine, phase_kind: int, query_mode: int, when):
    """Internal: single lunar-phase search."""
    req = ffi.new("DhruvLunarPhaseSearchRequest *")
    req.phase_kind = phase_kind
    req.query_mode = query_mode
    _set_single_search_time(req, when, arg_name="jd")

    out_event = ffi.new("DhruvLunarPhaseEvent *")
    out_found = ffi.new("uint8_t *")
    check(
        lib.dhruv_lunar_phase_search_ex(
            engine, req,
            out_event, out_found,
            ffi.NULL, 0, ffi.NULL,
        ),
        "lunar_phase_search_ex(single)",
    )
    if out_found[0] == 0:
        return None
    return _lunar_phase_event(out_event[0])


def next_purnima(engine, after_jd) -> Optional[LunarPhaseEvent]:
    """Find the next Purnima after a ``UtcTime`` or JD(TDB) anchor."""
    return _lunar_phase_single(engine, _LUNAR_PHASE_PURNIMA, _LUNAR_PHASE_NEXT, after_jd)


def prev_purnima(engine, before_jd) -> Optional[LunarPhaseEvent]:
    """Find the previous Purnima before a ``UtcTime`` or JD(TDB) anchor."""
    return _lunar_phase_single(engine, _LUNAR_PHASE_PURNIMA, _LUNAR_PHASE_PREV, before_jd)


def next_amavasya(engine, after_jd) -> Optional[LunarPhaseEvent]:
    """Find the next Amavasya after a ``UtcTime`` or JD(TDB) anchor."""
    return _lunar_phase_single(engine, _LUNAR_PHASE_AMAVASYA, _LUNAR_PHASE_NEXT, after_jd)


def prev_amavasya(engine, before_jd) -> Optional[LunarPhaseEvent]:
    """Find the previous Amavasya before a ``UtcTime`` or JD(TDB) anchor."""
    return _lunar_phase_single(engine, _LUNAR_PHASE_AMAVASYA, _LUNAR_PHASE_PREV, before_jd)


def search_lunar_phases(
    engine,
    phase_kind: int,
    start_jd,
    end_jd,
    max_results: int = 50,
) -> list[LunarPhaseEvent]:
    """Search for lunar phase events in a UTC or JD(TDB) range.

    *phase_kind*: 0=Amavasya, 1=Purnima.
    """
    req = ffi.new("DhruvLunarPhaseSearchRequest *")
    req.phase_kind = phase_kind
    req.query_mode = _LUNAR_PHASE_RANGE
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")

    def fetch(capacity: int):
        out_events = ffi.new("DhruvLunarPhaseEvent[]", capacity)
        out_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_lunar_phase_search_ex(
                engine, req,
                ffi.NULL, ffi.NULL,
                out_events, capacity, out_count,
            ),
            "lunar_phase_search_ex(range)",
        )
        count = int(out_count[0])
        return ([_lunar_phase_event(out_events[i]) for i in range(count)], count)

    return _collect_full_range(fetch, max_results)


# ---------------------------------------------------------------------------
# Sankranti search (dhruv_sankranti_search_ex)
# ---------------------------------------------------------------------------

_SANKRANTI_TARGET_ANY = 0
_SANKRANTI_TARGET_SPECIFIC = 1
_SANKRANTI_NEXT = 0
_SANKRANTI_PREV = 1
_SANKRANTI_RANGE = 2


def sankranti_config_default():
    """Return default DhruvSankrantiConfig."""
    return lib.dhruv_sankranti_config_default()


def next_sankranti(
    engine, after_jd, config=None, body_code: int = 0
) -> Optional[SankrantiEvent]:
    """Find the next sankranti after a ``UtcTime`` or JD(TDB) anchor.

    *body_code*: 0 = Sun (classical sankranti, default), otherwise any NAIF
    code or 10007 (Rahu) / 10008 (Ketu) for that body's rashi ingress.
    """
    req = ffi.new("DhruvSankrantiSearchRequest *")
    req.target_kind = _SANKRANTI_TARGET_ANY
    req.query_mode = _SANKRANTI_NEXT
    _set_single_search_time(req, after_jd, arg_name="after_jd")
    req.config = config if config is not None else lib.dhruv_sankranti_config_default()
    req.body_code = int(body_code)

    out_event = ffi.new("DhruvSankrantiEvent *")
    out_found = ffi.new("uint8_t *")
    check(
        lib.dhruv_sankranti_search_ex(
            engine, req,
            out_event, out_found,
            ffi.NULL, 0, ffi.NULL,
        ),
        "sankranti_search_ex(next)",
    )
    if out_found[0] == 0:
        return None
    return _sankranti_event(out_event[0])


def prev_sankranti(
    engine, before_jd, config=None, body_code: int = 0
) -> Optional[SankrantiEvent]:
    """Find the previous sankranti before a ``UtcTime`` or JD(TDB) anchor.

    See :func:`next_sankranti` for *body_code*.
    """
    req = ffi.new("DhruvSankrantiSearchRequest *")
    req.target_kind = _SANKRANTI_TARGET_ANY
    req.query_mode = _SANKRANTI_PREV
    _set_single_search_time(req, before_jd, arg_name="before_jd")
    req.config = config if config is not None else lib.dhruv_sankranti_config_default()
    req.body_code = int(body_code)

    out_event = ffi.new("DhruvSankrantiEvent *")
    out_found = ffi.new("uint8_t *")
    check(
        lib.dhruv_sankranti_search_ex(
            engine, req,
            out_event, out_found,
            ffi.NULL, 0, ffi.NULL,
        ),
        "sankranti_search_ex(prev)",
    )
    if out_found[0] == 0:
        return None
    return _sankranti_event(out_event[0])


def specific_sankranti(
    engine, at_jd, rashi_index: int, direction: str = "next", config=None,
    body_code: int = 0,
) -> Optional[SankrantiEvent]:
    """Find a direction-specific sankranti into a specific rashi.

    *rashi_index*: 0-based (0=Mesha .. 11=Meena).
    *direction*: ``"next"`` or ``"prev"``.
    See :func:`next_sankranti` for *body_code*.
    """
    if direction == "next":
        query_mode = _SANKRANTI_NEXT
        op_name = "specific_next"
    elif direction == "prev":
        query_mode = _SANKRANTI_PREV
        op_name = "specific_prev"
    else:
        raise ValueError("direction must be 'next' or 'prev'")

    req = ffi.new("DhruvSankrantiSearchRequest *")
    req.target_kind = _SANKRANTI_TARGET_SPECIFIC
    req.query_mode = query_mode
    req.rashi_index = rashi_index
    _set_single_search_time(req, at_jd, arg_name="at_jd")
    req.config = config if config is not None else lib.dhruv_sankranti_config_default()
    req.body_code = int(body_code)

    out_event = ffi.new("DhruvSankrantiEvent *")
    out_found = ffi.new("uint8_t *")
    check(
        lib.dhruv_sankranti_search_ex(
            engine, req,
            out_event, out_found,
            ffi.NULL, 0, ffi.NULL,
        ),
        f"sankranti_search_ex({op_name})",
    )
    if out_found[0] == 0:
        return None
    return _sankranti_event(out_event[0])


def search_sankrantis(
    engine,
    start_jd,
    end_jd,
    config=None,
    max_results: int = 50,
    body_code: int = 0,
) -> list[SankrantiEvent]:
    """Search for sankrantis in a UTC or JD(TDB) range.

    See :func:`next_sankranti` for *body_code*. Retrograde bodies can
    re-enter a rashi; such events carry ``is_retrograde=True``.
    """
    req = ffi.new("DhruvSankrantiSearchRequest *")
    req.target_kind = _SANKRANTI_TARGET_ANY
    req.query_mode = _SANKRANTI_RANGE
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")
    req.config = config if config is not None else lib.dhruv_sankranti_config_default()
    req.body_code = int(body_code)

    def fetch(capacity: int):
        out_events = ffi.new("DhruvSankrantiEvent[]", capacity)
        out_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_sankranti_search_ex(
                engine, req,
                ffi.NULL, ffi.NULL,
                out_events, capacity, out_count,
            ),
            "sankranti_search_ex(range)",
        )
        count = int(out_count[0])
        return ([_sankranti_event(out_events[i]) for i in range(count)], count)

    return _collect_full_range(fetch, max_results)


# ---------------------------------------------------------------------------
# Fixed-longitude search (dhruv_fixed_longitude_search, ABI v88)
# ---------------------------------------------------------------------------

_FIXED_LONGITUDE_NEXT = 0
_FIXED_LONGITUDE_PREV = 1
_FIXED_LONGITUDE_RANGE = 2


def _fixed_longitude_event(e) -> FixedLongitudeEvent:
    return FixedLongitudeEvent(
        utc=_utc_from_c(e.utc),
        jd_tdb=e.jd_tdb,
        body_code=e.body_code,
        target_longitude_deg=e.target_longitude_deg,
        angle_deg=e.angle_deg,
        matched_longitude_deg=e.matched_longitude_deg,
        sidereal_longitude_deg=e.sidereal_longitude_deg,
        tropical_longitude_deg=e.tropical_longitude_deg,
        actual_separation_deg=e.actual_separation_deg,
    )


def _fixed_longitude_request(
    query_mode: int,
    target_longitude_deg: float,
    angles_deg,
    include_special_angles: bool,
    config,
    body_code: int,
):
    angles = [float(a) for a in (angles_deg or [])]
    if len(angles) > MAX_FIXED_LONGITUDE_ANGLES:
        raise ValueError(
            f"at most {MAX_FIXED_LONGITUDE_ANGLES} angles are supported"
        )
    req = ffi.new("DhruvFixedLongitudeRequest *")
    req.query_mode = query_mode
    req.config = (
        _coerce_sankranti_config(config)
        if config is not None
        else lib.dhruv_sankranti_config_default()
    )
    req.body_code = int(body_code)
    req.target_longitude_deg = float(target_longitude_deg)
    req.angle_count = len(angles)
    for i, angle in enumerate(angles):
        req.target_angles_deg[i] = angle
    req.include_special_angles = 1 if include_special_angles else 0
    return req


def next_fixed_longitude(
    engine,
    after_jd,
    target_longitude_deg: float,
    angles_deg=None,
    include_special_angles: bool = False,
    config=None,
    body_code: int = 0,
) -> Optional[FixedLongitudeEvent]:
    """Find when a moving body next reaches a fixed sidereal longitude.

    *after_jd*: a ``UtcTime`` or JD(TDB) anchor. *angles_deg*: offsets added
    to the target (mod 360); ``None``/empty = conjunction only.
    *include_special_angles*: also search the body's classical
    special-aspect angles (Mars 90/210, Jupiter 120/240, Saturn 60/270)
    applied so the moving body casts that aspect onto the target.
    *body_code*: 0 = Sun (default), otherwise any NAIF code or 10007
    (Rahu) / 10008 (Ketu).
    """
    req = _fixed_longitude_request(
        _FIXED_LONGITUDE_NEXT, target_longitude_deg, angles_deg,
        include_special_angles, config, body_code,
    )
    _set_single_search_time(req, after_jd, arg_name="after_jd")
    out_event = ffi.new("DhruvFixedLongitudeEvent *")
    out_found = ffi.new("uint8_t *")
    check(
        lib.dhruv_fixed_longitude_search(
            engine, req,
            out_event, out_found,
            ffi.NULL, 0, ffi.NULL,
        ),
        "fixed_longitude_search(next)",
    )
    if out_found[0] == 0:
        return None
    return _fixed_longitude_event(out_event[0])


def prev_fixed_longitude(
    engine,
    before_jd,
    target_longitude_deg: float,
    angles_deg=None,
    include_special_angles: bool = False,
    config=None,
    body_code: int = 0,
) -> Optional[FixedLongitudeEvent]:
    """Find when a moving body previously reached a fixed sidereal longitude.

    See :func:`next_fixed_longitude` for the parameters.
    """
    req = _fixed_longitude_request(
        _FIXED_LONGITUDE_PREV, target_longitude_deg, angles_deg,
        include_special_angles, config, body_code,
    )
    _set_single_search_time(req, before_jd, arg_name="before_jd")
    out_event = ffi.new("DhruvFixedLongitudeEvent *")
    out_found = ffi.new("uint8_t *")
    check(
        lib.dhruv_fixed_longitude_search(
            engine, req,
            out_event, out_found,
            ffi.NULL, 0, ffi.NULL,
        ),
        "fixed_longitude_search(prev)",
    )
    if out_found[0] == 0:
        return None
    return _fixed_longitude_event(out_event[0])


def search_fixed_longitudes(
    engine,
    start_jd,
    end_jd,
    target_longitude_deg: float,
    angles_deg=None,
    include_special_angles: bool = False,
    config=None,
    body_code: int = 0,
    max_results: int = 50,
) -> list[FixedLongitudeEvent]:
    """Find every fixed-longitude reach event in a UTC or JD(TDB) range.

    See :func:`next_fixed_longitude` for the parameters. A range reaching
    past the loaded ephemeris coverage returns the events found up to the
    edge rather than raising.
    """
    req = _fixed_longitude_request(
        _FIXED_LONGITUDE_RANGE, target_longitude_deg, angles_deg,
        include_special_angles, config, body_code,
    )
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")

    def fetch(capacity: int):
        out_events = ffi.new("DhruvFixedLongitudeEvent[]", capacity)
        out_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_fixed_longitude_search(
                engine, req,
                ffi.NULL, ffi.NULL,
                out_events, capacity, out_count,
            ),
            "fixed_longitude_search(range)",
        )
        count = int(out_count[0])
        return ([_fixed_longitude_event(out_events[i]) for i in range(count)], count)

    return _collect_full_range(fetch, max_results)


def gochar_events(
    engine,
    eop,
    birth_utc,
    at_utc,
    location,
    *,
    transit_body_codes: list[int],
    natal_targets: list[GocharNatalTarget],
    bhava_config=None,
    riseset_config=None,
    sankranti_config=None,
    kundali_config=None,
    config=None,
) -> GocharEventsResult:
    req = ffi.new("DhruvGocharEventsRequest *")
    req.birth_utc = _make_utc(birth_utc)[0]
    req.at_utc = _make_utc(at_utc)[0]
    req.location = _make_location(location)[0]
    req.bhava_config = _make_bhava_config(bhava_config)[0]
    req.riseset_config = _make_riseset_config(riseset_config)[0]
    req.sankranti_config = _make_sankranti_config(sankranti_config)[0]

    if kundali_config is None:
        kundali_cfg = full_kundali_config_default()
    else:
        kundali_cfg = kundali_config
    req.kundali_config = kundali_cfg if not isinstance(kundali_cfg, ffi.CData) or ffi.typeof(kundali_cfg) != ffi.typeof("DhruvFullKundaliConfig *") else kundali_cfg[0]

    req.config = config if config is not None else lib.dhruv_gochar_events_config_default()

    body_codes = ffi.new("uint32_t[]", [int(code) for code in transit_body_codes])
    req.transit_body_codes = body_codes if len(transit_body_codes) else ffi.NULL
    req.transit_body_count = len(transit_body_codes)

    target_rows = ffi.new("DhruvGocharNatalTarget[]", len(natal_targets))
    target_names = []
    for idx, target in enumerate(natal_targets):
        name_ptr = ffi.NULL
        if target.name:
            name_ptr = ffi.new("char[]", target.name.encode("utf-8"))
            target_names.append(name_ptr)
        target_rows[idx].kind = int(target.kind)
        target_rows[idx].index = int(target.index)
        target_rows[idx].name_utf8 = name_ptr
        target_rows[idx].longitude_deg = float(target.longitude_deg)
    req.natal_targets = target_rows if len(natal_targets) else ffi.NULL
    req.natal_target_count = len(natal_targets)

    handle = ffi.new("DhruvGocharEventsHandle *")
    check(lib.dhruv_gochar_events(engine._ptr, eop, req, handle), "gochar_events")

    def _decode_tajaka(monthly: int, before: int) -> list[TajakaReturnEvent]:
        out_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_gochar_events_tajaka_count(handle[0], monthly, before, out_count),
            "gochar_events_tajaka_count",
        )
        events = []
        for i in range(int(out_count[0])):
            row = ffi.new("DhruvTajakaReturnEventRow *")
            check(
                lib.dhruv_gochar_events_tajaka_at(handle[0], monthly, before, i, row),
                "gochar_events_tajaka_at",
            )
            chart = None
            if row[0].has_chart:
                chart_out = ffi.new("DhruvFullKundaliResult *")
                check(
                    lib.dhruv_gochar_events_tajaka_chart_at(handle[0], monthly, before, i, chart_out),
                    "gochar_events_tajaka_chart_at",
                )
                try:
                    chart = _extract_full_kundali_result_ffi(chart_out[0])
                finally:
                    lib.dhruv_full_kundali_result_free(chart_out)
            events.append(TajakaReturnEvent(
                utc=_utc_from_c(row[0].utc),
                jd_tdb=row[0].jd_tdb,
                basis=row[0].basis,
                target_solar_longitude_deg=row[0].target_solar_longitude_deg,
                event_solar_longitude_deg=row[0].event_solar_longitude_deg,
                chart=chart,
            ))
        return events

    def _decode_tithi(monthly: int, before: int) -> list[TithiPraveshaEvent]:
        out_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_gochar_events_tithi_count(handle[0], monthly, before, out_count),
            "gochar_events_tithi_count",
        )
        events = []
        for i in range(int(out_count[0])):
            row = ffi.new("DhruvTithiPraveshaEventRow *")
            check(
                lib.dhruv_gochar_events_tithi_at(handle[0], monthly, before, i, row),
                "gochar_events_tithi_at",
            )
            chart = None
            if row[0].has_chart:
                chart_out = ffi.new("DhruvFullKundaliResult *")
                check(
                    lib.dhruv_gochar_events_tithi_chart_at(handle[0], monthly, before, i, chart_out),
                    "gochar_events_tithi_chart_at",
                )
                try:
                    chart = _extract_full_kundali_result_ffi(chart_out[0])
                finally:
                    lib.dhruv_full_kundali_result_free(chart_out)
            events.append(TithiPraveshaEvent(
                utc=_utc_from_c(row[0].utc),
                jd_tdb=row[0].jd_tdb,
                target_elongation_deg=row[0].target_elongation_deg,
                event_elongation_deg=row[0].event_elongation_deg,
                masa=_masa_from_c(row[0].masa),
                chart=chart,
            ))
        return events

    try:
        summary = ffi.new("DhruvGocharEventsSummary *")
        check(lib.dhruv_gochar_events_summary(handle[0], summary), "gochar_events_summary")

        transit_count = ffi.new("uint32_t *")
        check(lib.dhruv_gochar_events_transit_count(handle[0], transit_count), "gochar_events_transit_count")
        transit_events = []
        for i in range(int(transit_count[0])):
            row = ffi.new("DhruvTransitToNatalAspectEventRow *")
            check(lib.dhruv_gochar_events_transit_at(handle[0], i, row), "gochar_events_transit_at")
            transit_events.append(TransitToNatalAspectEvent(
                transit_body_code=row[0].transit_body_code,
                target_kind=row[0].target_kind,
                target_index=row[0].target_index,
                target_name=ffi.string(row[0].target_name).decode("utf-8"),
                aspect_kind=row[0].aspect_kind,
                aspect_owner=row[0].aspect_owner,
                aspect_angle_deg=row[0].aspect_angle_deg,
                utc=_utc_from_c(row[0].utc),
                jd_tdb=row[0].jd_tdb,
                transit_longitude_deg=row[0].transit_longitude_deg,
                target_longitude_deg=row[0].target_longitude_deg,
                actual_separation_deg=row[0].actual_separation_deg,
            ))

        return GocharEventsResult(
            birth_utc=_utc_from_c(summary[0].birth_utc),
            at_utc=_utc_from_c(summary[0].at_utc),
            reference=GocharReference(
                natal_tropical_solar_longitude_deg=summary[0].reference.natal_tropical_solar_longitude_deg,
                natal_sidereal_solar_longitude_deg=summary[0].reference.natal_sidereal_solar_longitude_deg,
                natal_elongation_deg=summary[0].reference.natal_elongation_deg,
                natal_masa=_masa_from_c(summary[0].reference.natal_masa),
            ),
            yearly_tajaka=GocharEventWindow(before=_decode_tajaka(0, 1), after=_decode_tajaka(0, 0)),
            yearly_tithi_pravesha=GocharEventWindow(before=_decode_tithi(0, 1), after=_decode_tithi(0, 0)),
            monthly_tajaka=GocharEventWindow(before=_decode_tajaka(1, 1), after=_decode_tajaka(1, 0)),
            monthly_tithi_pravesha=GocharEventWindow(before=_decode_tithi(1, 1), after=_decode_tithi(1, 0)),
            transit_events=transit_events,
        )
    finally:
        lib.dhruv_gochar_events_free(handle[0])
