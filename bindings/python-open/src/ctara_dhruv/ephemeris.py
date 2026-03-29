"""Position queries: Cartesian, spherical, and ecliptic coordinates."""

from __future__ import annotations

from ._ffi import ffi, lib
from ._check import check
from .types import (
    QUERY_OUTPUT_CARTESIAN,
    QUERY_OUTPUT_SPHERICAL,
    QUERY_TIME_JD_TDB,
    QUERY_TIME_UTC,
    QueryRequest,
    QueryResult,
    SphericalCoords,
    SphericalState,
    StateVector,
    UtcTime,
)

_DHRUV_PATH_CAPACITY = 512
_DHRUV_MAX_SPK_PATHS = 8


def _build_engine_config(spk_paths: list[str], lsk_path: str | None):
    """Build a DhruvEngineConfig from Python arguments."""
    cfg = ffi.new("DhruvEngineConfig *")
    cfg.spk_path_count = len(spk_paths)
    for i, p in enumerate(spk_paths):
        p_bytes = p.encode("utf-8")
        ffi.memmove(cfg.spk_paths_utf8[i], p_bytes, len(p_bytes))
    if lsk_path:
        lsk_bytes = lsk_path.encode("utf-8")
        ffi.memmove(cfg.lsk_path_utf8, lsk_bytes, len(lsk_bytes))
    cfg.cache_capacity = 256
    cfg.strict_validation = 1
    return cfg


def _utc_struct(utc: UtcTime):
    """Build a DhruvUtcTime from a UtcTime dataclass."""
    u = ffi.new("DhruvUtcTime *")
    u.year = utc.year
    u.month = utc.month
    u.day = utc.day
    u.hour = utc.hour
    u.minute = utc.minute
    u.second = utc.second
    return u


def _read_state_vector(sv) -> StateVector:
    return StateVector(
        x=sv.position_km[0],
        y=sv.position_km[1],
        z=sv.position_km[2],
        vx=sv.velocity_km_s[0],
        vy=sv.velocity_km_s[1],
        vz=sv.velocity_km_s[2],
    )


def _read_spherical_state(ss) -> SphericalState:
    return SphericalState(
        lon_deg=ss.lon_deg,
        lat_deg=ss.lat_deg,
        distance_km=ss.distance_km,
        lon_speed=ss.lon_speed,
        lat_speed=ss.lat_speed,
        distance_speed=ss.distance_speed,
    )


def _query_request_struct(request: QueryRequest):
    q = ffi.new("DhruvQueryRequest *")
    q.target = request.target
    q.observer = request.observer
    q.frame = request.frame
    q.output_mode = request.output_mode

    time_kind = request.time_kind
    if time_kind is None:
        has_jd = request.epoch_tdb_jd is not None
        has_utc = request.utc_time is not None
        if has_jd == has_utc:
            raise ValueError(
                "QueryRequest must provide exactly one of epoch_tdb_jd or utc_time when time_kind is omitted"
            )
        time_kind = QUERY_TIME_UTC if has_utc else QUERY_TIME_JD_TDB

    q.time_kind = time_kind
    if time_kind == QUERY_TIME_JD_TDB:
        if request.epoch_tdb_jd is None:
            raise ValueError("epoch_tdb_jd is required for JD(TDB) queries")
        q.epoch_tdb_jd = request.epoch_tdb_jd
    elif time_kind == QUERY_TIME_UTC:
        if request.utc_time is None:
            raise ValueError("utc_time is required for UTC queries")
        q.utc = _utc_struct(request.utc_time)[0]
    else:
        q.time_kind = time_kind
    return q


def _read_query_result(result, output_mode: int) -> QueryResult:
    state = None
    spherical_state = None
    if output_mode != QUERY_OUTPUT_SPHERICAL:
        state = _read_state_vector(result.state_vector)
    if output_mode != QUERY_OUTPUT_CARTESIAN:
        spherical_state = _read_spherical_state(result.spherical_state)
    return QueryResult(
        state=state,
        spherical_state=spherical_state,
        output_mode=output_mode,
    )


def query(engine_handle, request: QueryRequest) -> QueryResult:
    """Unified ephemeris query with JD-vs-UTC input and output-mode selection."""

    q = _query_request_struct(request)
    out = ffi.new("DhruvQueryResult *")
    check(lib.dhruv_engine_query_request(engine_handle, q, out), "engine_query")
    return _read_query_result(out, request.output_mode)


def query_once(
    spk_paths: list[str],
    lsk_path: str | None,
    target: int,
    observer: int,
    jd_tdb: float,
    frame: int = 0,
) -> StateVector:
    """One-shot query: creates engine, queries, and tears down internally.

    Args:
        spk_paths: List of SPK kernel file paths.
        lsk_path: LSK file path (optional).
        target: NAIF body code of the target.
        observer: NAIF body code of the observer.
        jd_tdb: Julian Date in TDB.
        frame: Frame code (0=J2000/ICRF, 1=ecliptic J2000).

    Returns:
        StateVector with position (km) and velocity (km/s).
    """
    cfg = _build_engine_config(spk_paths, lsk_path)
    q = ffi.new("DhruvQuery *")
    q.target = target
    q.observer = observer
    q.frame = frame
    q.epoch_tdb_jd = jd_tdb

    out = ffi.new("DhruvStateVector *")
    check(lib.dhruv_query_once(cfg, q, out), "query_once")
    return _read_state_vector(out)


def body_ecliptic_lon_lat(
    engine_handle, body_code: int, jd_tdb: float
) -> tuple[float, float]:
    """Compute ecliptic longitude and latitude for a body.

    Args:
        engine_handle: DhruvEngineHandle pointer.
        body_code: NAIF body code.
        jd_tdb: Julian Date in TDB.

    Returns:
        Tuple of (lon_deg, lat_deg).
    """
    out_lon = ffi.new("double *")
    out_lat = ffi.new("double *")
    check(
        lib.dhruv_body_ecliptic_lon_lat(
            engine_handle, body_code, jd_tdb, out_lon, out_lat
        ),
        "body_ecliptic_lon_lat",
    )
    return (out_lon[0], out_lat[0])


def cartesian_to_spherical(x: float, y: float, z: float) -> SphericalCoords:
    """Convert Cartesian (km) to spherical coordinates. Pure math.

    Args:
        x, y, z: Cartesian position in km.

    Returns:
        SphericalCoords with lon_deg, lat_deg, distance_km.
    """
    pos = ffi.new("double[3]", [x, y, z])
    out = ffi.new("DhruvSphericalCoords *")
    check(
        lib.dhruv_cartesian_to_spherical(pos, out),
        "cartesian_to_spherical",
    )
    return SphericalCoords(
        lon_deg=out.lon_deg,
        lat_deg=out.lat_deg,
        distance_km=out.distance_km,
    )
