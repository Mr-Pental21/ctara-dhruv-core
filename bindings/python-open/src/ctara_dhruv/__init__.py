"""ctara-dhruv: Python bindings for the ctara-dhruv-core ephemeris engine.

Usage::

    import ctara_dhruv as cd

    with cd.Engine(["de442s.bsp"], "naif0012.tls") as eng:
        result = cd.query(
            eng._ptr,
            cd.QueryRequest(
                target=cd.Body.MARS,
                observer=cd.Body.SSB,
                epoch_tdb_jd=2451545.0,
                output_mode=cd.QUERY_OUTPUT_CARTESIAN,
            ),
        )
        print(result.state.x, result.state.y, result.state.z)
"""

# Engine lifecycle
from ctara_dhruv.engine import Engine, init, engine, lsk, eop

# Enums
from ctara_dhruv.enums import (
    Body,
    DhruvStatus,
    AyanamshaSystem,
    AyanamshaMode,
    BhavaSystem,
    Graha,
    SunLimb,
    RiseSetEvent,
    RiseSetResultType,
    StationType,
    MaxSpeedType,
    DashaSystem,
    ReferencePlane,
    SearchQueryMode,
    GrahanKind,
    MotionKind,
    LunarPhaseKind,
    SankrantiTargetKind,
    ChandraGrahanType,
    SuryaGrahanType,
    CharakarakaScheme,
    CharakarakaRole,
    TaraOutputKind,
)

# Core types
from ctara_dhruv.types import (
    QUERY_OUTPUT_BOTH,
    QUERY_OUTPUT_CARTESIAN,
    QUERY_OUTPUT_SPHERICAL,
    QUERY_TIME_JD_TDB,
    QUERY_TIME_UTC,
    QueryRequest,
    QueryResult,
    StateVector,
    SphericalCoords,
    SphericalState,
    UtcTime,
    GeoLocation,
    Dms,
    RashiInfo,
    NakshatraInfo,
    Nakshatra28Info,
    BhavaEntry,
    BhavaResult,
    ConjunctionEvent,
    ChandraGrahanResult,
    SuryaGrahanResult,
    StationaryEvent,
    MaxSpeedEvent,
    LunarPhaseEvent,
    SankrantiEvent,
    GrahaEntry,
    GrahaPositions,
    CharakarakaEntry,
    CharakarakaResult,
    DashaPeriod,
    DashaSnapshot,
)

# Errors
from ctara_dhruv._check import DhruvError

# Ephemeris
from ctara_dhruv.ephemeris import (
    query,
    body_ecliptic_lon_lat,
    cartesian_to_spherical,
)

# Time
from ctara_dhruv.time import utc_to_jd_tdb, jd_tdb_to_utc, nutation

# Ayanamsha
from ctara_dhruv.ayanamsha import ayanamsha, system_count, reference_plane_default

# Tara
from ctara_dhruv.tara import TaraCatalog

# Dasha
from ctara_dhruv.dasha import (
    DashaLevel,
    DashaHierarchy,
    dasha_selection_config_default,
    dasha_variation_config_default,
    dasha_hierarchy,
    dasha_snapshot,
    dasha_level0,
    dasha_level0_entity,
    dasha_children,
    dasha_child_period,
    dasha_complete_level,
)

__all__ = [
    # Engine
    "Engine", "init", "engine", "lsk", "eop",
    # Enums
    "Body", "DhruvStatus", "AyanamshaSystem", "AyanamshaMode",
    "BhavaSystem", "Graha",
    "SunLimb", "RiseSetEvent", "RiseSetResultType",
    "StationType", "MaxSpeedType", "DashaSystem", "ReferencePlane",
    "SearchQueryMode", "GrahanKind", "MotionKind", "LunarPhaseKind",
    "SankrantiTargetKind", "ChandraGrahanType", "SuryaGrahanType",
    "CharakarakaScheme", "CharakarakaRole", "TaraOutputKind",
    "QUERY_TIME_JD_TDB", "QUERY_TIME_UTC",
    "QUERY_OUTPUT_CARTESIAN", "QUERY_OUTPUT_SPHERICAL", "QUERY_OUTPUT_BOTH",
    # Types
    "QueryRequest", "QueryResult",
    "StateVector", "SphericalCoords", "SphericalState", "UtcTime",
    "GeoLocation", "Dms", "RashiInfo", "NakshatraInfo", "Nakshatra28Info",
    "BhavaEntry", "BhavaResult", "ConjunctionEvent",
    "ChandraGrahanResult", "SuryaGrahanResult",
    "StationaryEvent", "MaxSpeedEvent",
    "LunarPhaseEvent", "SankrantiEvent",
    "GrahaEntry", "GrahaPositions", "CharakarakaEntry", "CharakarakaResult", "DashaPeriod",
    "DashaSnapshot",
    # Errors
    "DhruvError",
    # Functions
    "query", "body_ecliptic_lon_lat",
    "cartesian_to_spherical",
    "utc_to_jd_tdb", "jd_tdb_to_utc", "nutation",
    "ayanamsha", "system_count", "reference_plane_default",
    "TaraCatalog",
    "DashaLevel", "DashaHierarchy",
    "dasha_selection_config_default", "dasha_variation_config_default",
    "dasha_hierarchy", "dasha_snapshot",
    "dasha_level0", "dasha_level0_entity",
    "dasha_children", "dasha_child_period", "dasha_complete_level",
]
