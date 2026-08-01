"""Frozen dataclasses mirroring the C ABI result types from dhruv_ffi_c.

Every type is ``@dataclass(frozen=True)`` so instances are immutable and
hashable.  Field names follow the C struct field names converted to
``snake_case`` where practical.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from typing import TYPE_CHECKING, Optional

if TYPE_CHECKING:
    from .dasha import DashaHierarchy


# ---------------------------------------------------------------------------
# Core types
# ---------------------------------------------------------------------------


QUERY_TIME_JD_TDB = 0
QUERY_TIME_UTC = 1

QUERY_OUTPUT_CARTESIAN = 0
QUERY_OUTPUT_SPHERICAL = 1
QUERY_OUTPUT_BOTH = 2

TIME_POLICY_STRICT_LSK = 0
TIME_POLICY_HYBRID_DELTA_T = 1

DELTA_T_MODEL_LEGACY_ESPENAK_MEEUS_2006 = 0
DELTA_T_MODEL_SMH2016_WITH_PRE720_QUADRATIC = 1

FUTURE_DELTA_T_TRANSITION_LEGACY_TT_UTC_BLEND = 0
FUTURE_DELTA_T_TRANSITION_BRIDGE_FROM_MODERN_ENDPOINT = 1

SMH_FUTURE_FAMILY_ADDENDUM_2020_PIECEWISE = 0
SMH_FUTURE_FAMILY_CONSTANT_C_MINUS20 = 1
SMH_FUTURE_FAMILY_CONSTANT_C_MINUS17P52 = 2
SMH_FUTURE_FAMILY_CONSTANT_C_MINUS15P32 = 3
SMH_FUTURE_FAMILY_STEPHENSON_1997 = 4
SMH_FUTURE_FAMILY_STEPHENSON_2016 = 5

TT_UTC_SOURCE_LSK_DELTA_AT = 0
TT_UTC_SOURCE_DELTA_T_MODEL = 1

TIME_WARNING_LSK_FUTURE_FROZEN = 0
TIME_WARNING_LSK_PRE_RANGE_FALLBACK = 1
TIME_WARNING_EOP_FUTURE_FROZEN = 2
TIME_WARNING_EOP_PRE_RANGE_FALLBACK = 3
TIME_WARNING_DELTA_T_MODEL_USED = 4


@dataclass(frozen=True)
class StateVector:
    """Cartesian state vector (km and km/s)."""

    x: float
    y: float
    z: float
    vx: float
    vy: float
    vz: float


@dataclass(frozen=True)
class SphericalCoords:
    """Spherical position: longitude, latitude (degrees), distance (km)."""

    lon_deg: float
    lat_deg: float
    distance_km: float


@dataclass(frozen=True)
class SphericalState:
    """Spherical state with angular velocities.

    Speeds: ``lon_speed`` and ``lat_speed`` in deg/day,
    ``distance_speed`` in km/s.
    """

    lon_deg: float
    lat_deg: float
    distance_km: float
    lon_speed: float
    lat_speed: float
    distance_speed: float


@dataclass(frozen=True)
class QueryRequest:
    """Unified ephemeris query request.

    Use ``epoch_tdb_jd`` for JD(TDB) queries or ``utc_time`` for UTC queries.
    ``time_kind`` is optional and inferred when exactly one input form is set.
    """

    target: int
    observer: int
    frame: int = 0
    epoch_tdb_jd: Optional[float] = None
    utc_time: Optional["UtcTime"] = None
    time_kind: Optional[int] = None
    output_mode: int = QUERY_OUTPUT_CARTESIAN


@dataclass(frozen=True)
class QueryResult:
    """Unified ephemeris query result."""

    state: Optional[StateVector]
    spherical_state: Optional[SphericalState]
    output_mode: int


@dataclass(frozen=True)
class UtcTime:
    """Broken-down UTC calendar time matching ``DhruvUtcTime``."""

    year: int
    month: int
    day: int
    hour: int
    minute: int
    second: float

    def to_datetime(self) -> datetime:
        """Convert to a ``datetime.datetime``, truncating to microseconds."""
        whole_sec = int(self.second)
        microsecond = int((self.second - whole_sec) * 1_000_000)
        return datetime(
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            whole_sec,
            microsecond,
        )

    @classmethod
    def from_datetime(cls, dt: datetime) -> UtcTime:
        """Create a ``UtcTime`` from a ``datetime.datetime``."""
        sec = dt.second + dt.microsecond / 1_000_000.0
        return cls(dt.year, dt.month, dt.day, dt.hour, dt.minute, sec)


@dataclass(frozen=True)
class TimeConversionOptions:
    """Hybrid UTC conversion behavior and fallback settings."""

    warn_on_fallback: bool = True
    delta_t_model: int = DELTA_T_MODEL_SMH2016_WITH_PRE720_QUADRATIC
    freeze_future_dut1: bool = True
    pre_range_dut1: float = 0.0
    future_delta_t_transition: int = FUTURE_DELTA_T_TRANSITION_LEGACY_TT_UTC_BLEND
    future_transition_years: float = 100.0
    smh_future_family: int = SMH_FUTURE_FAMILY_ADDENDUM_2020_PIECEWISE


@dataclass(frozen=True)
class TimePolicy:
    """UTC conversion policy selector plus optional hybrid settings."""

    mode: int = TIME_POLICY_HYBRID_DELTA_T
    options: TimeConversionOptions = field(default_factory=TimeConversionOptions)


@dataclass(frozen=True)
class TimeWarning:
    """One warning entry emitted during UTC conversion."""

    kind: int
    utc_seconds: float
    first_entry_utc_seconds: float
    last_entry_utc_seconds: float
    used_delta_at_seconds: float
    mjd: float
    first_entry_mjd: float
    last_entry_mjd: float
    used_dut1_seconds: float
    delta_t_model: int
    delta_t_segment: int


@dataclass(frozen=True)
class TimeDiagnostics:
    """Diagnostics emitted by UTC conversion."""

    source: int
    tt_minus_utc_s: float
    warnings: list[TimeWarning]


@dataclass(frozen=True)
class UtcToTdbRequest:
    """Typed UTC->JD(TDB) request with policy."""

    utc: UtcTime
    time_policy: TimePolicy = field(default_factory=TimePolicy)


@dataclass(frozen=True)
class UtcToTdbResult:
    """UTC->JD(TDB) result plus diagnostics."""

    jd_tdb: float
    diagnostics: TimeDiagnostics


@dataclass(frozen=True)
class GrahaLongitudesConfig:
    """Config for unified graha longitude computation."""

    kind: int = 0
    ayanamsha_system: int = 0
    use_nutation: bool = False
    precession_model: int = 3
    reference_plane: int = -1


@dataclass(frozen=True)
class GeoLocation:
    """Observer geographic location."""

    lat_deg: float
    lon_deg: float
    alt_m: float = 0.0


@dataclass(frozen=True)
class Dms:
    """Degrees-minutes-seconds representation."""

    degrees: int
    minutes: int
    seconds: float


# ---------------------------------------------------------------------------
# Rashi / Nakshatra
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class RashiInfo:
    """Rashi (zodiac sign) classification.

    ``rashi_index``: 0-based (0=Mesha .. 11=Meena).
    ``degrees_in_rashi``: decimal degrees within the rashi [0, 30).
    ``dms``: position within rashi as DMS.
    """

    rashi_index: int
    degrees_in_rashi: float
    dms: Dms


@dataclass(frozen=True)
class NakshatraInfo:
    """Nakshatra (lunar mansion) classification, 27-scheme.

    ``nakshatra_index``: 0-based (0=Ashwini .. 26=Revati).
    ``pada``: quarter 1-4.
    ``degrees_in_nakshatra``: decimal degrees within the nakshatra.
    ``degrees_in_pada``: decimal degrees within the pada.
    """

    nakshatra_index: int
    pada: int
    degrees_in_nakshatra: float
    degrees_in_pada: float


@dataclass(frozen=True)
class Nakshatra28Info:
    """Nakshatra classification, 28-scheme (with Abhijit).

    ``nakshatra_index``: 0-based (0=Ashwini, 21=Abhijit, 27=Revati).
    ``pada``: quarter 1-4 (0 for Abhijit).
    """

    nakshatra_index: int
    pada: int
    degrees_in_nakshatra: float


# ---------------------------------------------------------------------------
# Bhava (House Systems)
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class BhavaEntry:
    """A single bhava (house).

    ``number``: bhava number 1-12.
    ``cusp_deg``: cusp longitude [0, 360).
    ``start_deg`` / ``end_deg``: span in degrees.
    """

    number: int
    cusp_deg: float
    start_deg: float
    end_deg: float


@dataclass(frozen=True)
class BhavaResult:
    """Complete bhava computation result with 12 houses plus lagna and MC."""

    bhavas: list[BhavaEntry]
    lagna_deg: float
    mc_deg: float
    rashi_bhava: Optional[BhavaResult] = None


# ---------------------------------------------------------------------------
# Rise / Set
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class RiseSetResult:
    """Single rise/set event result.

    ``result_type``: 0=event, 1=never rises, 2=never sets.
    ``event_code``: DHRUV_EVENT_* constant (valid when result_type==0).
    ``jd_tdb``: event time in JD TDB (valid when result_type==0).
    """

    result_type: int
    event_code: int
    jd_tdb: float


# ---------------------------------------------------------------------------
# Search results
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class ConjunctionEvent:
    """Conjunction / aspect event.

    ``body1_code`` / ``body2_code``: NAIF body codes, or 10007 (Rahu) /
    10008 (Ketu).
    ``target_separation_deg``: the target angle this event matched (equals
    the config angle for single-angle searches).
    Sidereal echo fields are ``None`` unless the request carried a
    ``sidereal_config``.
    """

    utc: UtcTime
    jd_tdb: float
    actual_separation_deg: float
    body1_longitude_deg: float
    body2_longitude_deg: float
    body1_latitude_deg: float
    body2_latitude_deg: float
    body1_code: int
    body2_code: int
    target_separation_deg: float = 0.0
    has_sidereal: bool = False
    body1_sidereal_longitude_deg: Optional[float] = None
    body2_sidereal_longitude_deg: Optional[float] = None
    body1_rashi_index: Optional[int] = None
    body2_rashi_index: Optional[int] = None


@dataclass(frozen=True)
class ChandraGrahanResult:
    """Lunar eclipse (Chandra Grahan) result.

    ``grahan_type``: 0=penumbral, 1=partial, 2=total.
    Contact JDs use ``DHRUV_JD_ABSENT`` (-1.0) when not applicable.
    """

    grahan_type: int
    magnitude: float
    penumbral_magnitude: float
    greatest_grahan_utc: UtcTime
    greatest_grahan_jd: float
    p1_utc: UtcTime
    p1_jd: float
    u1_utc: Optional[UtcTime]
    u1_jd: float
    u2_utc: Optional[UtcTime]
    u2_jd: float
    u3_utc: Optional[UtcTime]
    u3_jd: float
    u4_utc: Optional[UtcTime]
    u4_jd: float
    p4_utc: UtcTime
    p4_jd: float
    moon_ecliptic_lat_deg: float
    angular_separation_deg: float
    # Moon's apparent geocentric RA/declination at greatest grahan, degrees
    # (equinox of date, nutation applied).
    moon_right_ascension_deg: float = 0.0
    moon_declination_deg: float = 0.0


@dataclass(frozen=True)
class EclipseGeoPoint:
    latitude_deg: float
    longitude_deg: float


@dataclass(frozen=True)
class SuryaGrahanPathPoint:
    jd_tdb: float
    utc: UtcTime
    center: EclipseGeoPoint
    northern_limit: Optional[EclipseGeoPoint]
    southern_limit: Optional[EclipseGeoPoint]
    width_km: float
    central_duration_seconds: float
    sun_altitude_deg: float
    sun_azimuth_deg: float
    grahan_type: int


@dataclass(frozen=True)
class SuryaMagnitudeRing:
    """One instantaneous iso-magnitude contour ring."""

    level: float
    boundary: tuple[EclipseGeoPoint, ...]
    # Pole containment: 0=none, 1=north, 2=south.
    contains_pole: int = 0


@dataclass(frozen=True)
class SuryaGrahanFootprint:
    jd_tdb: float
    utc: UtcTime
    boundary: tuple[EclipseGeoPoint, ...]
    # Pole containment of the shadow region: 0=none, 1=north, 2=south.
    contains_pole: int = 0
    magnitude_rings: tuple[SuryaMagnitudeRing, ...] = ()


@dataclass(frozen=True)
class SuryaContactFootprint:
    """Penumbral footprint at one of the event's own contact moments.

    ``contact``: 0=C1, 1=C2, 2=greatest, 3=C3, 4=C4. ``boundary`` may be
    empty at exact C1/C4 tangency; fall back to the nearest sampled
    footprint in that case.
    """

    contact: int
    jd_tdb: float
    utc: UtcTime
    boundary: tuple[EclipseGeoPoint, ...]
    contains_pole: int = 0
    magnitude_rings: tuple[SuryaMagnitudeRing, ...] = ()


@dataclass(frozen=True)
class SuryaUmbraFootprint:
    """Instantaneous umbral/antumbral shadow outline at one moment.

    ``grahan_type``: 2=total (umbra) or 1=annular (antumbra).
    """

    jd_tdb: float
    utc: UtcTime
    grahan_type: int
    boundary: tuple[EclipseGeoPoint, ...]
    contains_pole: int = 0


@dataclass(frozen=True)
class SuryaLocalGridSample:
    """One visible sample of the per-event local-circumstance grid."""

    latitude_deg: float
    longitude_deg: float
    magnitude: float
    obscuration: float
    maximum_utc: UtcTime
    maximum_jd: float
    first_contact_utc: UtcTime
    first_contact_jd: float
    last_contact_utc: UtcTime
    last_contact_jd: float
    visible_duration_seconds: float


@dataclass(frozen=True)
class SuryaIsolineRing:
    """One closed boundary ring.

    ``contains_pole``: 0=none, 1=north, 2=south.
    """

    contains_pole: int
    boundary: tuple[EclipseGeoPoint, ...]


@dataclass(frozen=True)
class SuryaRingSetLevel:
    """One isoline level or corridor segment with its rings.

    ``grahan_type`` is a corridor segment type code, or -1 for isolines.
    """

    level_value: float
    grahan_type: int
    rings: tuple[SuryaIsolineRing, ...]


@dataclass(frozen=True)
class SuryaIsolines:
    visibility_boundary: tuple[SuryaIsolineRing, ...]
    duration_isolines: tuple[SuryaRingSetLevel, ...]
    magnitude_isolines: tuple[SuryaRingSetLevel, ...]


@dataclass(frozen=True)
class SuryaGrahanResult:
    """Solar eclipse (Surya Grahan) result.

    ``grahan_type``: 0=partial, 1=annular, 2=total, 3=hybrid.
    """

    grahan_type: int
    magnitude: float
    greatest_grahan_utc: UtcTime
    greatest_grahan_jd: float
    c1_utc: Optional[UtcTime]
    c1_jd: float
    c2_utc: Optional[UtcTime]
    c2_jd: float
    c3_utc: Optional[UtcTime]
    c3_jd: float
    c4_utc: Optional[UtcTime]
    c4_jd: float
    moon_ecliptic_lat_deg: float
    angular_separation_deg: float
    # Sun's apparent geocentric RA/declination at greatest grahan, degrees
    # (equinox of date, nutation applied).
    sun_right_ascension_deg: float = 0.0
    sun_declination_deg: float = 0.0
    obscuration: float = 0.0
    apparent_diameter_ratio: float = 0.0
    gamma: float = 0.0
    greatest_location: Optional[GeoLocation] = None
    bessel_x: float = 0.0
    bessel_y: float = 0.0
    bessel_d_deg: float = 0.0
    bessel_mu_deg: float = 0.0
    bessel_l1: float = 0.0
    bessel_l2: float = 0.0
    bessel_tan_f1: float = 0.0
    bessel_tan_f2: float = 0.0
    path_count: int = 0
    footprint_count: int = 0
    path: tuple[SuryaGrahanPathPoint, ...] = ()
    footprints: tuple[SuryaGrahanFootprint, ...] = ()
    local_visible: Optional[bool] = None
    local_grahan_type: Optional[int] = None
    local_maximum_utc: Optional[UtcTime] = None
    local_maximum_jd: float = -1.0
    local_c1_utc: Optional[UtcTime] = None
    local_c1_jd: float = -1.0
    local_c2_utc: Optional[UtcTime] = None
    local_c2_jd: float = -1.0
    local_c3_utc: Optional[UtcTime] = None
    local_c3_jd: float = -1.0
    local_c4_utc: Optional[UtcTime] = None
    local_c4_jd: float = -1.0
    local_magnitude: float = 0.0
    local_obscuration: float = 0.0
    local_sun_altitude_deg: float = 0.0
    local_sun_azimuth_deg: float = 0.0
    local_central_duration_seconds: float = 0.0
    # Whether/how the central shadow reaches Earth: 0=none, 1=partial, 2=full.
    centrality: int = 0
    local_grid: tuple[SuryaLocalGridSample, ...] = ()
    isolines: Optional[SuryaIsolines] = None
    # Swept central corridor segments (grahan_type set per segment).
    central_corridor: Optional[tuple[SuryaRingSetLevel, ...]] = None
    contact_footprints: tuple[SuryaContactFootprint, ...] = ()
    umbra_footprints: tuple[SuryaUmbraFootprint, ...] = ()


@dataclass(frozen=True)
class StationaryEvent:
    """Planetary station event.

    ``station_type``: 0=retrograde, 1=direct.
    Sidereal echo fields are ``None`` unless the request carried a
    ``sidereal_config``.
    """

    utc: UtcTime
    jd_tdb: float
    body_code: int
    longitude_deg: float
    latitude_deg: float
    station_type: int
    has_sidereal: bool = False
    sidereal_longitude_deg: Optional[float] = None
    rashi_index: Optional[int] = None


@dataclass(frozen=True)
class MaxSpeedEvent:
    """Peak-speed event.

    ``speed_type``: 0=direct, 1=retrograde.
    Sidereal echo fields are ``None`` unless the request carried a
    ``sidereal_config``.
    """

    utc: UtcTime
    jd_tdb: float
    body_code: int
    longitude_deg: float
    latitude_deg: float
    speed_deg_per_day: float
    speed_type: int
    has_sidereal: bool = False
    sidereal_longitude_deg: Optional[float] = None
    rashi_index: Optional[int] = None


@dataclass(frozen=True)
class LunarPhaseEvent:
    """Lunar phase event (Purnima / Amavasya).

    ``phase``: DHRUV_LUNAR_PHASE_NEW_MOON or _FULL_MOON.
    """

    utc: UtcTime
    phase: int
    moon_longitude_deg: float
    sun_longitude_deg: float


@dataclass(frozen=True)
class SankrantiEvent:
    """Sankranti (rashi ingress) event.

    ``rashi_index``: 0-based (0=Mesha .. 11=Meena).
    ``sun_sidereal_longitude_deg`` / ``sun_tropical_longitude_deg`` are
    legacy aliases for the tracked body's longitudes (the Sun for classical
    sankranti requests); they always equal ``sidereal_longitude_deg`` /
    ``tropical_longitude_deg``.
    ``body_code``: the tracked body (NAIF code, or 10007 Rahu / 10008 Ketu).
    ``is_retrograde``: the rashi boundary was crossed in retrograde motion.
    """

    utc: UtcTime
    rashi_index: int
    sun_sidereal_longitude_deg: float
    sun_tropical_longitude_deg: float
    body_code: int = 10
    sidereal_longitude_deg: float = 0.0
    tropical_longitude_deg: float = 0.0
    is_retrograde: bool = False


# ---------------------------------------------------------------------------
# Pure-math Panchang classifiers
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class TithiPosition:
    """Tithi from elongation (pure math).

    ``tithi_index``: 0-based (0..29).
    ``paksha``: 0=Shukla, 1=Krishna.
    ``tithi_in_paksha``: 1-based (1..15).
    ``degrees_in_tithi``: [0, 12).
    """

    tithi_index: int
    paksha: int
    tithi_in_paksha: int
    degrees_in_tithi: float


@dataclass(frozen=True)
class KaranaPosition:
    """Karana from elongation (pure math).

    ``karana_index``: 0-based (0..59).
    ``degrees_in_karana``: [0, 6).
    """

    karana_index: int
    degrees_in_karana: float


@dataclass(frozen=True)
class YogaPosition:
    """Yoga from sidereal sum (pure math).

    ``yoga_index``: 0-based (0..26).
    ``degrees_in_yoga``: [0, 13.333...).
    """

    yoga_index: int
    degrees_in_yoga: float


@dataclass(frozen=True)
class SamvatsaraResult:
    """Jovian year (samvatsara) result.

    ``samvatsara_index``: 0-based (0..59).
    ``cycle_position``: 1-based (1..60).
    """

    samvatsara_index: int
    cycle_position: int


# ---------------------------------------------------------------------------
# Panchang (engine-computed, with time boundaries)
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class TithiInfo:
    """Tithi with time boundaries.

    ``tithi_index``: 0-based (0=Shukla Pratipada .. 29=Amavasya).
    ``paksha``: 0=Shukla, 1=Krishna.
    ``tithi_in_paksha``: 1-based (1-15).
    """

    tithi_index: int
    paksha: int
    tithi_in_paksha: int
    start: UtcTime
    end: UtcTime


@dataclass(frozen=True)
class KaranaInfo:
    """Karana with time boundaries.

    ``karana_index``: 0-based sequence index (0-59) within the synodic month.
    ``karana_name_index``: name index in ALL_KARANAS (0=Bava .. 10=Kinstugna).
    """

    karana_index: int
    karana_name_index: int
    start: UtcTime
    end: UtcTime


@dataclass(frozen=True)
class YogaInfo:
    """Yoga with time boundaries.

    ``yoga_index``: 0-based (0=Vishkumbha .. 26=Vaidhriti).
    """

    yoga_index: int
    start: UtcTime
    end: UtcTime


@dataclass(frozen=True)
class VaarInfo:
    """Vaar (weekday) with time boundaries.

    ``vaar_index``: 0=Ravivaar(Sunday) .. 6=Shanivaar(Saturday).
    """

    vaar_index: int
    start: UtcTime
    end: UtcTime


@dataclass(frozen=True)
class HoraInfo:
    """Hora with time boundaries.

    ``hora_index``: Chaldean sequence lord index (0=Surya .. 6=Mangal).
    ``hora_position``: 0-based position within the Vedic day (0-23).
    """

    hora_index: int
    hora_position: int
    start: UtcTime
    end: UtcTime


@dataclass(frozen=True)
class GhatikaInfo:
    """Ghatika with time boundaries.

    ``value``: ghatika number (1-60).
    """

    value: int
    start: UtcTime
    end: UtcTime


@dataclass(frozen=True)
class PanchangNakshatraInfo:
    """Moon's nakshatra with time boundaries.

    ``nakshatra_index``: 0-based (0=Ashwini .. 26=Revati).
    ``pada``: quarter 1-4.
    """

    nakshatra_index: int
    pada: int
    start: UtcTime
    end: UtcTime


@dataclass(frozen=True)
class MasaInfo:
    """Lunar month (masa) with time boundaries.

    ``masa_index``: 0-based (0=Chaitra .. 11=Phalguna).
    ``adhika``: True if intercalary month.
    """

    masa_index: int
    adhika: bool
    start: UtcTime
    end: UtcTime


@dataclass(frozen=True)
class AyanaInfo:
    """Ayana with time boundaries.

    ``ayana``: 0=Uttarayana, 1=Dakshinayana.
    """

    ayana: int
    start: UtcTime
    end: UtcTime


@dataclass(frozen=True)
class VarshaInfo:
    """Varsha (Jovian year) with time boundaries.

    ``samvatsara_index``: 0-based (0=Prabhava .. 59=Akshaya).
    ``order``: 1-based position in the 60-year cycle (1-60).
    """

    samvatsara_index: int
    order: int
    start: UtcTime
    end: UtcTime


@dataclass(frozen=True)
class PanchangResult:
    """Combined panchang result with optional calendar fields.

    Each field is ``None`` when not requested or not computed.
    """

    tithi: Optional[TithiInfo] = None
    karana: Optional[KaranaInfo] = None
    yoga: Optional[YogaInfo] = None
    vaar: Optional[VaarInfo] = None
    hora: Optional[HoraInfo] = None
    ghatika: Optional[GhatikaInfo] = None
    nakshatra: Optional[PanchangNakshatraInfo] = None
    masa: Optional[MasaInfo] = None
    ayana: Optional[AyanaInfo] = None
    varsha: Optional[VarshaInfo] = None


@dataclass(frozen=True)
class PanchangEventsResult:
    """Exact panchang element segments overlapping a UTC range.

    Each per-kind list chains exactly within its kind
    (``item.end == next_item.start``), including across Vedic-day rolls for
    the location-dependent kinds. The first segment of each kind may start
    before the requested ``from_utc`` and the last may end after ``to_utc``.
    Kinds not selected by the include mask are empty lists.

    The location-dependent kinds (``vaars``, ``horas``, ``ghatikas``) are
    populated only when the sweep was given an observer location.

    ``truncated``: True when the sweep hit the event cap before covering the
    full range. ``next_from``: resume point (only set when truncated) —
    re-issue the call from here and deduplicate on ``(kind, start)``.
    """

    tithis: list[TithiInfo]
    karanas: list[KaranaInfo]
    yogas: list[YogaInfo]
    vaars: list[VaarInfo]
    horas: list[HoraInfo]
    ghatikas: list[GhatikaInfo]
    nakshatras: list[PanchangNakshatraInfo]
    masas: list[MasaInfo]
    ayanas: list[AyanaInfo]
    varshas: list[VarshaInfo]
    truncated: bool = False
    next_from: Optional[UtcTime] = None


# ---------------------------------------------------------------------------
# Sphuta
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class SphutalResult:
    """All 16 sphuta longitudes (indexed 0-15 matching ALL_SPHUTAS order)."""

    longitudes: list[float]


# ---------------------------------------------------------------------------
# Special Lagnas
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class SpecialLagnas:
    """All 8 special lagnas (sidereal degrees)."""

    bhava_lagna: float
    hora_lagna: float
    ghati_lagna: float
    vighati_lagna: float
    varnada_lagna: float
    sree_lagna: float
    pranapada_lagna: float
    indu_lagna: float


# ---------------------------------------------------------------------------
# Arudha Padas
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class ArudhaResult:
    """Single arudha pada result."""

    bhava_number: int
    longitude_deg: float
    rashi_index: int


# ---------------------------------------------------------------------------
# Upagrahas
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class AllUpagrahas:
    """All 11 upagraha sidereal longitudes."""

    gulika: float
    maandi: float
    kaala: float
    mrityu: float
    artha_prahara: float
    yama_ghantaka: float
    dhooma: float
    vyatipata: float
    parivesha: float
    indra_chapa: float
    upaketu: float


# ---------------------------------------------------------------------------
# Ashtakavarga
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class BhinnaAshtakavarga:
    """Bhinna Ashtakavarga for a single graha.

    ``graha_index``: 0=Sun through 6=Saturn.
    ``points``: benefic points per rashi (12 entries, max 8 each).
    ``contributors``: attribution matrix ``[rashi][contributor]`` (12x8, 0/1).
      Contributor order: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Lagna.
    """

    graha_index: int
    points: list[int]
    contributors: list[list[int]] = field(default_factory=lambda: [[0] * 8 for _ in range(12)])


@dataclass(frozen=True)
class SarvaAshtakavarga:
    """Sarva Ashtakavarga with sodhana.

    ``total_points``: SAV per rashi (sum of all 7 BAVs).
    ``after_trikona``: after Trikona Sodhana.
    ``after_ekadhipatya``: after Ekadhipatya Sodhana.
    """

    total_points: list[int]
    after_trikona: list[int]
    after_ekadhipatya: list[int]


@dataclass(frozen=True)
class AshtakavargaResult:
    """Complete ashtakavarga result."""

    bavs: list[BhinnaAshtakavarga]
    sav: SarvaAshtakavarga


# ---------------------------------------------------------------------------
# Drishti (Planetary Aspects)
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class DrishtiEntry:
    """Single drishti (aspect) measurement between two points."""

    angular_distance: float
    base_virupa: float
    special_virupa: float
    total_virupa: float


@dataclass(frozen=True)
class GrahaDrishtiMatrix:
    """9x9 graha-to-graha drishti matrix."""

    matrix: list[list[DrishtiEntry]]


@dataclass(frozen=True)
class DrishtiResult:
    """Complete drishti result.

    ``graha_to_graha``: 9x9 matrix.
    ``graha_to_bhava``: 9x12 matrix.
    ``graha_to_rashi_bhava``: 9x12 matrix.
    ``graha_to_lagna``: 9 entries.
    ``graha_to_bindus``: 9x19 matrix.
    """

    graha_to_graha: list[list[DrishtiEntry]]
    graha_to_bhava: list[list[DrishtiEntry]]
    graha_to_rashi_bhava: list[list[DrishtiEntry]]
    graha_to_lagna: list[DrishtiEntry]
    graha_to_bindus: list[list[DrishtiEntry]]


# ---------------------------------------------------------------------------
# Graha Positions
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class BasicStates:
    exalted: bool
    debilitated: bool
    combust: bool
    retrograde: bool
    moolatrikone: bool
    marankarak_sthana: bool
    mrityubhaga: bool
    pushkaramsha: bool
    pushkarbhaga: bool


@dataclass(frozen=True)
class SensitivePointDistances:
    mrityubhaga: float
    pushkarbhaga: float


@dataclass(frozen=True)
class GrahaEntry:
    """Single graha position entry.

    ``sidereal_longitude``: degrees [0, 360).
    ``rashi_index``: 0-based (0-11).
    ``nakshatra_index``: 0-based (0-26), 255 if not computed.
    ``pada``: 1-4, 0 if not computed.
    ``bhava_number``: 1-12, 0 if not computed.
    ``rashi_bhava_number``: 1-12, 0 if not computed.
    ``equatorial_valid``: True when equatorial output was requested; then
    ``right_ascension_deg`` (geocentric RA, degrees [0, 360)),
    ``declination_deg`` (geocentric declination, degrees [-90, +90]) and
    ``ecliptic_latitude_deg`` (geocentric ecliptic latitude, degrees) are
    populated. Equinox of date, nutation per the request's ``use_nutation``
    flag; geometric (no light-time/aberration). Lagna and Rahu/Ketu report
    ecliptic latitude exactly 0.
    """

    sidereal_longitude: float
    rashi_index: int
    nakshatra_index: int
    pada: int
    bhava_number: int
    rashi_bhava_number: int = 0
    basic_states_valid: bool = False
    basic_states: Optional[BasicStates] = None
    sensitive_point_distances_valid: bool = False
    sensitive_point_distances: Optional[SensitivePointDistances] = None
    equatorial_valid: bool = False
    right_ascension_deg: float = 0.0
    declination_deg: float = 0.0
    ecliptic_latitude_deg: float = 0.0


@dataclass(frozen=True)
class GrahaPositions:
    """Comprehensive graha positions result.

    ``grahas``: 9 Vedic grahas indexed by graha index 0-8.
    ``lagna``: lagna entry (sentinel if not computed).
    ``outer_planets``: [Uranus, Neptune, Pluto].
    ``earth_orientation_valid``: True when equatorial output was requested
    and ``gmst_deg``/``gast_deg`` are populated.
    ``gmst_deg``/``gast_deg``: Greenwich mean/apparent sidereal time in
    degrees [0, 360) at the request instant.
    """

    grahas: list[GrahaEntry]
    lagna: GrahaEntry
    outer_planets: list[GrahaEntry]
    earth_orientation_valid: bool = False
    gmst_deg: float = 0.0
    gast_deg: float = 0.0


@dataclass(frozen=True)
class GrahaPositionsPoint:
    """One epoch of a fixed-cadence graha-positions series.

    ``utc``: epoch as a (year, month, day, hour, minute, second) tuple.
    ``jd_utc``: epoch as JD UTC.
    ``positions``: same shape as the single-epoch ``GrahaPositions``.
    """

    utc: tuple
    jd_utc: float
    positions: GrahaPositions


# ---------------------------------------------------------------------------
# Core Bindus
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class BindusResult:
    """Curated sensitive points (bindus) result.

    Contains 12 arudha padas and 7 special points, each as a
    ``GrahaEntry`` with optional nakshatra/bhava enrichment.
    """

    arudha_padas: list[GrahaEntry]
    bhrigu_bindu: GrahaEntry
    pranapada_lagna: GrahaEntry
    gulika: GrahaEntry
    maandi: GrahaEntry
    hora_lagna: GrahaEntry
    ghati_lagna: GrahaEntry
    sree_lagna: GrahaEntry
    rashi_bhava_arudha_padas: Optional[list[GrahaEntry]] = None


# ---------------------------------------------------------------------------
# Amsha (Divisional Charts)
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class AmshaEntry:
    """Position in a divisional chart.

    ``name`` / ``display_name``: identity of this point, resolved from the
    entry's fixed position in its chart array. ``name`` is a stable
    snake_case key (``"sree_lagna"``, ``"a1"``, ``"bhava_3"``, ``"surya"``);
    ``display_name`` is the human-readable form.
    ``family`` / ``point_index``: the ``DHRUV_AMSHA_POINT_FAMILY_*`` code and
    the index inside that family, which together address the point.
    ``sidereal_longitude``: degrees [0, 360).
    ``rashi_index``: 0-based (0-11).
    ``dms_degrees`` / ``dms_minutes`` / ``dms_seconds``: DMS within rashi.
    ``degrees_in_rashi``: decimal degrees within rashi [0, 30).
    ``nakshatra_index``: 0-based (0-26). ``pada``: 1-4.
    ``rashi_bhava_number``: whole-sign bhava (1-12) from the varga lagna. A
    varga transform is not monotonic, so the transformed cusps in
    ``bhava_cusps`` are not ordered house boundaries and there is no
    cusp-based bhava inside a varga.
    """

    sidereal_longitude: float
    rashi_index: int
    dms_degrees: int
    dms_minutes: int
    dms_seconds: float
    degrees_in_rashi: float
    nakshatra_index: int = 0
    pada: int = 0
    rashi_bhava_number: int = 0
    family: int = 0
    point_index: int = 0
    name: Optional[str] = None
    display_name: Optional[str] = None


@dataclass(frozen=True)
class AmshaChart:
    """Single amsha (divisional) chart result.

    ``amsha_code``: D-number of this chart.
    ``variation_code``: amsha-specific variation code; 0=default for that amsha.
    """

    amsha_code: int
    variation_code: int
    grahas: list[AmshaEntry]
    lagna: AmshaEntry
    outer_planets: Optional[list[AmshaEntry]] = None
    bhava_cusps: Optional[list[AmshaEntry]] = None
    rashi_bhava_cusps: Optional[list[AmshaEntry]] = None
    arudha_padas: Optional[list[AmshaEntry]] = None
    rashi_bhava_arudha_padas: Optional[list[AmshaEntry]] = None
    upagrahas: Optional[list[AmshaEntry]] = None
    sphutas: Optional[list[AmshaEntry]] = None
    special_lagnas: Optional[list[AmshaEntry]] = None


@dataclass(frozen=True)
class AmshaVariationInfo:
    """Variation metadata for one amsha-specific variation code."""

    amsha_code: int
    variation_code: int
    name: str
    label: str
    is_default: bool
    description: str


@dataclass(frozen=True)
class AmshaVariationCatalog:
    """Variation catalog for a single amsha."""

    amsha_code: int
    default_variation_code: int
    variations: list[AmshaVariationInfo]


@dataclass(frozen=True)
class AmshaSeriesChart:
    """Slim varga chart within one amsha series point.

    ``grahas`` holds the 9 navagraha entries when the series was requested
    with ``include_grahas``; ``None`` otherwise. The varga lagna is always
    present.
    """

    amsha_code: int
    variation_code: int
    lagna: AmshaEntry
    grahas: Optional[list[AmshaEntry]] = None


@dataclass(frozen=True)
class AmshaSeriesPoint:
    """One epoch of a fixed-cadence amsha series.

    ``utc``: epoch as a (year, month, day, hour, minute, second) tuple.
    ``jd_utc``: epoch as JD UTC.
    ``charts``: one ``AmshaSeriesChart`` per request, in request order
    (duplicate requests repeated).
    """

    utc: tuple
    jd_utc: float
    charts: list[AmshaSeriesChart]


@dataclass(frozen=True)
class AmshaLagnaSegment:
    """One varga-lagna rashi segment with exact transition boundaries.

    The first segment of a sweep starts at the requested ``from_utc``; later
    segments start at the exact transition. ``end`` is the exact transition
    time (the last segment's end is the first transition at or after
    ``to_utc``).
    """

    rashi_index: int
    start: UtcTime
    end: UtcTime


@dataclass(frozen=True)
class AmshaLagnaEntry:
    """Varga-lagna segments for one unique (amsha, variation) request."""

    amsha_code: int
    variation_code: int
    segments: list[AmshaLagnaSegment]


@dataclass(frozen=True)
class AmshaLagnaEventsResult:
    """Result of an amsha-lagna events sweep.

    ``entries``: one entry per unique request (duplicates collapsed), in
    request order. ``truncated``: True when the total-segment cap was hit.
    ``next_from``: resume point (only set when truncated).
    """

    entries: list[AmshaLagnaEntry]
    truncated: bool = False
    next_from: Optional[UtcTime] = None


# ---------------------------------------------------------------------------
# Shadbala
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class SthanaBalaBreakdown:
    """Sthana Bala sub-components."""

    uchcha: float
    saptavargaja: float
    ojhayugma: float
    kendradi: float
    drekkana: float
    total: float


@dataclass(frozen=True)
class KalaBalaBreakdown:
    """Kala Bala sub-components."""

    nathonnatha: float
    paksha: float
    tribhaga: float
    abda: float
    masa: float
    vara: float
    hora: float
    ayana: float
    yuddha: float
    total: float


@dataclass(frozen=True)
class ShadbalaEntry:
    """Shadbala for a single sapta graha.

    ``graha_index``: 0-6 (Sun through Saturn).
    ``is_strong``: True if total meets required strength.
    """

    graha_index: int
    sthana: SthanaBalaBreakdown
    dig: float
    kala: KalaBalaBreakdown
    cheshta: float
    naisargika: float
    drik: float
    total_shashtiamsas: float
    total_rupas: float
    required_strength: float
    is_strong: bool


@dataclass(frozen=True)
class ShadbalaResult:
    """Shadbala result for all 7 sapta grahas."""

    entries: list[ShadbalaEntry]


@dataclass(frozen=True)
class BhavaBalaEntry:
    """Bhava Bala for a single house."""

    bhava_number: int
    cusp_sidereal_lon: float
    rashi_index: int
    lord_graha_index: int
    bhavadhipati: float
    dig: float
    drishti: float
    occupation_bonus: float
    rising_bonus: float
    total_virupas: float
    total_rupas: float


@dataclass(frozen=True)
class BhavaBalaResult:
    """Bhava Bala result for all 12 houses."""

    entries: list[BhavaBalaEntry]


# ---------------------------------------------------------------------------
# Vimsopaka
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class VimsopakaEntry:
    """Vimsopaka Bala for a single graha.

    ``graha_index``: 0-8 (all 9 navagrahas).
    Scores for 4 varga groupings (each out of 20).
    """

    graha_index: int
    shadvarga: float
    saptavarga: float
    dashavarga: float
    shodasavarga: float


@dataclass(frozen=True)
class VimsopakaResult:
    """Vimsopaka result for all 9 navagrahas."""

    entries: list[VimsopakaEntry]


@dataclass(frozen=True)
class BalaBundleResult:
    """Combined bala surfaces for one chart."""

    shadbala: ShadbalaResult
    vimsopaka: VimsopakaResult
    ashtakavarga: AshtakavargaResult
    bhavabala: BhavaBalaResult


# ---------------------------------------------------------------------------
# Avastha (Planetary State)
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class SayanadiResult:
    """Sayanadi avastha for a single graha.

    ``avastha``: SayanadiAvastha code (1=Sayana..11=Kautuka, 0=Nidra).
    ``sub_states``: 5 sub-state indices (Ka/Cha/Ta-retroflex/Ta-dental/Pa).
    """

    avastha: int
    sub_states: list[int]


@dataclass(frozen=True)
class GrahaAvasthas:
    """All avasthas for a single graha.

    ``baladi``: BaladiAvastha index (0-4).
    ``jagradadi``: JagradadiAvastha index (0-2).
    ``deeptadi``: primary DeeptadiAvastha index (0-8).
    ``deeptadi_states``: all applicable DeeptadiAvastha indices.
    ``deeptadi_mask``: bit mask of all applicable DeeptadiAvastha indices.
    ``lajjitadi``: primary LajjitadiAvastha index (0-5), or ``None``.
    ``lajjitadi_states``: all applicable LajjitadiAvastha indices.
    ``lajjitadi_mask``: bit mask of all applicable LajjitadiAvastha indices.
    """

    baladi: int
    jagradadi: int
    deeptadi: int
    deeptadi_states: list[int]
    deeptadi_mask: int
    lajjitadi: int | None
    lajjitadi_states: list[int]
    lajjitadi_mask: int
    sayanadi: SayanadiResult


@dataclass(frozen=True)
class AllGrahaAvasthas:
    """Avasthas for all 9 grahas."""

    entries: list[GrahaAvasthas]


# ---------------------------------------------------------------------------
# Dasha
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class DashaPeriod:
    """Single dasha period.

    ``entity_type``: 0=Graha, 1=Rashi, 2=Yogini.
    ``entity_index``: Graha (0-8), rashi (0-11), or yogini (0-7).
    ``entity_name``: exact canonical entity name when available.
    ``level``: hierarchical level (0-4).
    ``start_utc`` / ``end_utc``: structured Gregorian UTC [start, end) interval.
    ``start_jd`` / ``end_jd``: JD UTC kept alongside UTC.
    ``order``: 1-indexed position among siblings.
    ``parent_idx``: index into parent level's array (0 for level 0).
    """

    entity_type: int
    entity_index: int
    start_jd: float
    end_jd: float
    level: int
    order: int
    parent_idx: int
    entity_name: Optional[str] = None
    start_utc: Optional[UtcTime] = None
    end_utc: Optional[UtcTime] = None


@dataclass(frozen=True)
class DashaSnapshot:
    """Dasha snapshot at a point in time (max 5 levels).

    ``system``: DashaSystem code.
    ``query_utc``: structured Gregorian UTC query instant.
    ``query_jd``: query JD UTC kept alongside UTC.
    ``periods``: one period per active level.
    """

    system: int
    query_jd: float
    periods: list[DashaPeriod]
    query_utc: Optional[UtcTime] = None


# ---------------------------------------------------------------------------
# Charakaraka
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class CharakarakaEntry:
    """Single charakaraka assignment entry."""

    role_code: int
    graha_index: int
    rank: int
    longitude_deg: float
    degrees_in_rashi: float
    effective_degrees_in_rashi: float


@dataclass(frozen=True)
class CharakarakaResult:
    """Charakaraka assignment result for one scheme."""

    scheme: int
    used_eight_karakas: bool
    entries: list[CharakarakaEntry]


# ---------------------------------------------------------------------------
# Tara (Fixed Stars)
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class EquatorialPosition:
    """Equatorial position of a fixed star."""

    ra_deg: float
    dec_deg: float
    distance_au: float


@dataclass(frozen=True)
class EarthState:
    """Earth state vector in AU and AU/day."""

    position_au: list[float]
    velocity_au_day: list[float]


@dataclass(frozen=True)
class TaraComputeResult:
    """Unified tara (fixed star) computation result.

    ``output_kind``: 0=equatorial, 1=ecliptic, 2=sidereal.
    Only the field matching ``output_kind`` is meaningful.
    """

    output_kind: int
    equatorial: Optional[EquatorialPosition] = None
    ecliptic: Optional[SphericalCoords] = None
    sidereal_longitude_deg: Optional[float] = None


# ---------------------------------------------------------------------------
# Graha Longitudes
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class GrahaLongitudes:
    """Sidereal or tropical longitudes for all 9 grahas plus outer planets.

    Indexed by Graha order: Surya=0, Chandra=1, Mangal=2, Buddh=3,
    Guru=4, Shukra=5, Shani=6, Rahu=7, Ketu=8.
    ``outer_planets`` is [Uranus, Neptune, Pluto] when populated.
    """

    longitudes: list[float]
    outer_planets: Optional[list[float]] = None


@dataclass(frozen=True)
class MovingOsculatingApogeeEntry:
    """Moving osculating apogee longitude for one graha."""

    graha_index: int
    sidereal_longitude: float
    ayanamsha_deg: float
    reference_plane_longitude: float


@dataclass(frozen=True)
class MovingOsculatingApogees:
    """Batch moving osculating apogee result."""

    entries: list[MovingOsculatingApogeeEntry]


# ---------------------------------------------------------------------------
# Full Kundali
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class FullKundaliResult:
    """Complete kundali (birth chart) result.

    Each section is ``None`` when not requested or computation failed.
    """

    ayanamsha_deg: float
    bhava_cusps: Optional[BhavaResult] = None
    rashi_bhava_cusps: Optional[BhavaResult] = None
    bhava_cusp_sensitive_point_distances: Optional[list[SensitivePointDistances]] = None
    rashi_bhava_cusp_sensitive_point_distances: Optional[list[SensitivePointDistances]] = None
    graha_positions: Optional[GrahaPositions] = None
    bindus: Optional[BindusResult] = None
    drishti: Optional[DrishtiResult] = None
    ashtakavarga: Optional[AshtakavargaResult] = None
    upagrahas: Optional[AllUpagrahas] = None
    sphutas: Optional[SphutalResult] = None
    special_lagnas: Optional[SpecialLagnas] = None
    amshas: Optional[list[AmshaChart]] = None
    shadbala: Optional[ShadbalaResult] = None
    bhavabala: Optional[BhavaBalaResult] = None
    vimsopaka: Optional[VimsopakaResult] = None
    avastha: Optional[AllGrahaAvasthas] = None
    charakaraka: Optional[CharakarakaResult] = None
    panchang: Optional[PanchangResult] = None
    dasha: Optional[list[DashaHierarchy]] = None
    dasha_snapshots: Optional[list[DashaSnapshot]] = None


@dataclass(frozen=True)
class GocharNatalTarget:
    kind: int
    index: int
    name: str
    longitude_deg: float


@dataclass(frozen=True)
class GocharEventsConfig:
    tajaka_return_basis: int
    yearly_count: int
    monthly_count: int
    transit_window_days: float
    include_return_charts: bool
    solar_step_size_days: float
    lunar_step_size_days: float
    solar_convergence_days: float
    lunar_convergence_days: float
    max_iterations: int


@dataclass(frozen=True)
class GocharReference:
    natal_tropical_solar_longitude_deg: float
    natal_sidereal_solar_longitude_deg: float
    natal_elongation_deg: float
    natal_masa: MasaInfo


@dataclass(frozen=True)
class GocharEventWindow:
    before: list
    after: list


@dataclass(frozen=True)
class TajakaReturnEvent:
    utc: UtcTime
    jd_tdb: float
    basis: int
    target_solar_longitude_deg: float
    event_solar_longitude_deg: float
    chart: Optional[FullKundaliResult] = None


@dataclass(frozen=True)
class TithiPraveshaEvent:
    utc: UtcTime
    jd_tdb: float
    target_elongation_deg: float
    event_elongation_deg: float
    masa: MasaInfo
    chart: Optional[FullKundaliResult] = None


@dataclass(frozen=True)
class TransitToNatalAspectEvent:
    transit_body_code: int
    target_kind: int
    target_index: int
    target_name: str
    aspect_kind: int
    aspect_owner: int
    aspect_angle_deg: float
    utc: UtcTime
    jd_tdb: float
    transit_longitude_deg: float
    target_longitude_deg: float
    actual_separation_deg: float


@dataclass(frozen=True)
class GocharEventsResult:
    birth_utc: UtcTime
    at_utc: UtcTime
    reference: GocharReference
    yearly_tajaka: GocharEventWindow
    yearly_tithi_pravesha: GocharEventWindow
    monthly_tajaka: GocharEventWindow
    monthly_tithi_pravesha: GocharEventWindow
    transit_events: list[TransitToNatalAspectEvent]
