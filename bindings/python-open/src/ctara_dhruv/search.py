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
    StationaryEvent,
    MaxSpeedEvent,
    LunarPhaseEvent,
    SankrantiEvent,
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


def _conjunction_event(e) -> ConjunctionEvent:
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
    )


def _chandra_grahan(r) -> ChandraGrahanResult:
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
    )


def _surya_grahan(r) -> SuryaGrahanResult:
    path = []
    footprints = []
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
            ))
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
    )


def _stationary_event(e) -> StationaryEvent:
    return StationaryEvent(
        utc=_utc_from_c(e.utc),
        jd_tdb=e.jd_tdb,
        body_code=e.body_code,
        longitude_deg=e.longitude_deg,
        latitude_deg=e.latitude_deg,
        station_type=e.station_type,
    )


def _max_speed_event(e) -> MaxSpeedEvent:
    return MaxSpeedEvent(
        utc=_utc_from_c(e.utc),
        jd_tdb=e.jd_tdb,
        body_code=e.body_code,
        longitude_deg=e.longitude_deg,
        latitude_deg=e.latitude_deg,
        speed_deg_per_day=e.speed_deg_per_day,
        speed_type=e.speed_type,
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
) -> Optional[ConjunctionEvent]:
    """Find next conjunction after a ``UtcTime`` or JD(TDB) anchor."""
    req = ffi.new("DhruvConjunctionSearchRequest *")
    req.body1_code = body1_code
    req.body2_code = body2_code
    req.query_mode = _CONJUNCTION_NEXT
    _set_single_search_time(req, after_jd_tdb, arg_name="after_jd_tdb")
    req.config = config if config is not None else lib.dhruv_conjunction_config_default()

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
) -> Optional[ConjunctionEvent]:
    """Find previous conjunction before a ``UtcTime`` or JD(TDB) anchor."""
    req = ffi.new("DhruvConjunctionSearchRequest *")
    req.body1_code = body1_code
    req.body2_code = body2_code
    req.query_mode = _CONJUNCTION_PREV
    _set_single_search_time(req, before_jd_tdb, arg_name="before_jd_tdb")
    req.config = config if config is not None else lib.dhruv_conjunction_config_default()

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
) -> list[ConjunctionEvent]:
    """Search for conjunctions in a UTC or JD(TDB) range."""
    req = ffi.new("DhruvConjunctionSearchRequest *")
    req.body1_code = body1_code
    req.body2_code = body2_code
    req.query_mode = _CONJUNCTION_RANGE
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")
    req.config = config if config is not None else lib.dhruv_conjunction_config_default()

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


def grahan_config_default():
    """Return default DhruvGrahanConfig."""
    return lib.dhruv_grahan_config_default()


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


def _motion_single_stationary(engine, query_mode: int, body_code: int, when, config):
    """Internal: single stationary search."""
    req = ffi.new("DhruvMotionSearchRequest *")
    req.body_code = body_code
    req.motion_kind = _MOTION_STATIONARY
    req.query_mode = query_mode
    _set_single_search_time(req, when, arg_name="jd")
    req.config = config if config is not None else lib.dhruv_stationary_config_default()

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


def _motion_single_max_speed(engine, query_mode: int, body_code: int, when, config):
    """Internal: single max-speed search."""
    req = ffi.new("DhruvMotionSearchRequest *")
    req.body_code = body_code
    req.motion_kind = _MOTION_MAX_SPEED
    req.query_mode = query_mode
    _set_single_search_time(req, when, arg_name="jd")
    req.config = config if config is not None else lib.dhruv_stationary_config_default()

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
    engine, body_code: int, after_jd, config=None
) -> Optional[StationaryEvent]:
    """Find next stationary point after a ``UtcTime`` or JD(TDB) anchor."""
    return _motion_single_stationary(engine, _MOTION_NEXT, body_code, after_jd, config)


def prev_stationary(
    engine, body_code: int, before_jd, config=None
) -> Optional[StationaryEvent]:
    """Find previous stationary point before a ``UtcTime`` or JD(TDB) anchor."""
    return _motion_single_stationary(engine, _MOTION_PREV, body_code, before_jd, config)


def search_stationary(
    engine,
    body_code: int,
    start_jd,
    end_jd,
    config=None,
    max_results: int = 100,
) -> list[StationaryEvent]:
    """Search for stationary points in a UTC or JD(TDB) range."""
    req = ffi.new("DhruvMotionSearchRequest *")
    req.body_code = body_code
    req.motion_kind = _MOTION_STATIONARY
    req.query_mode = _MOTION_RANGE
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")
    req.config = config if config is not None else lib.dhruv_stationary_config_default()

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
    engine, body_code: int, after_jd, config=None
) -> Optional[MaxSpeedEvent]:
    """Find next max-speed event after a ``UtcTime`` or JD(TDB) anchor."""
    return _motion_single_max_speed(engine, _MOTION_NEXT, body_code, after_jd, config)


def prev_max_speed(
    engine, body_code: int, before_jd, config=None
) -> Optional[MaxSpeedEvent]:
    """Find previous max-speed event before a ``UtcTime`` or JD(TDB) anchor."""
    return _motion_single_max_speed(engine, _MOTION_PREV, body_code, before_jd, config)


def search_max_speeds(
    engine,
    body_code: int,
    start_jd,
    end_jd,
    config=None,
    max_results: int = 100,
) -> list[MaxSpeedEvent]:
    """Search for max-speed events in a UTC or JD(TDB) range."""
    req = ffi.new("DhruvMotionSearchRequest *")
    req.body_code = body_code
    req.motion_kind = _MOTION_MAX_SPEED
    req.query_mode = _MOTION_RANGE
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")
    req.config = config if config is not None else lib.dhruv_stationary_config_default()

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
    engine, after_jd, config=None
) -> Optional[SankrantiEvent]:
    """Find the next sankranti after a ``UtcTime`` or JD(TDB) anchor."""
    req = ffi.new("DhruvSankrantiSearchRequest *")
    req.target_kind = _SANKRANTI_TARGET_ANY
    req.query_mode = _SANKRANTI_NEXT
    _set_single_search_time(req, after_jd, arg_name="after_jd")
    req.config = config if config is not None else lib.dhruv_sankranti_config_default()

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
    engine, before_jd, config=None
) -> Optional[SankrantiEvent]:
    """Find the previous sankranti before a ``UtcTime`` or JD(TDB) anchor."""
    req = ffi.new("DhruvSankrantiSearchRequest *")
    req.target_kind = _SANKRANTI_TARGET_ANY
    req.query_mode = _SANKRANTI_PREV
    _set_single_search_time(req, before_jd, arg_name="before_jd")
    req.config = config if config is not None else lib.dhruv_sankranti_config_default()

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
    engine, at_jd, rashi_index: int, direction: str = "next", config=None
) -> Optional[SankrantiEvent]:
    """Find a direction-specific sankranti into a specific rashi.

    *rashi_index*: 0-based (0=Mesha .. 11=Meena).
    *direction*: ``"next"`` or ``"prev"``.
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
) -> list[SankrantiEvent]:
    """Search for sankrantis in a UTC or JD(TDB) range."""
    req = ffi.new("DhruvSankrantiSearchRequest *")
    req.target_kind = _SANKRANTI_TARGET_ANY
    req.query_mode = _SANKRANTI_RANGE
    _set_range_search_time(req, start_jd, end_jd, start_name="start_jd", end_name="end_jd")
    req.config = config if config is not None else lib.dhruv_sankranti_config_default()

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
