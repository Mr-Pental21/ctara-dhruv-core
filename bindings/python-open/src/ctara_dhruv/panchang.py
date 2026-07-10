"""Panchang computation: unified ``dhruv_panchang_compute_ex`` and individual
``_for_date`` wrappers for tithi, karana, yoga, vaar, hora, ghatika,
nakshatra, masa, ayana, and varsha.
"""

from __future__ import annotations

from typing import Optional, Union

from ._ffi import ffi, lib
from ._check import check
from .types import (
    GeoLocation,
    PanchangEventsResult,
    PanchangResult,
    SamvatsaraResult,
    TithiInfo,
    KaranaInfo,
    YogaInfo,
    VaarInfo,
    HoraInfo,
    GhatikaInfo,
    PanchangNakshatraInfo,
    MasaInfo,
    AyanaInfo,
    VarshaInfo,
    UtcTime,
)


# ---------------------------------------------------------------------------
# Include-mask constants (match C ABI)
# ---------------------------------------------------------------------------

INCLUDE_TITHI = 1 << 0
INCLUDE_KARANA = 1 << 1
INCLUDE_YOGA = 1 << 2
INCLUDE_VAAR = 1 << 3
INCLUDE_HORA = 1 << 4
INCLUDE_GHATIKA = 1 << 5
INCLUDE_NAKSHATRA = 1 << 6
INCLUDE_MASA = 1 << 7
INCLUDE_AYANA = 1 << 8
INCLUDE_VARSHA = 1 << 9
INCLUDE_ALL_CORE = 0x7F
INCLUDE_ALL_CALENDAR = 0x380
INCLUDE_ALL = 0x3FF
INCLUDE_LOCATION_INDEPENDENT = 0x3C7
INCLUDE_LOCATION_DEPENDENT = 0x38

# Hard ceiling on total events per panchang_events sweep
# (matches DHRUV_MAX_PANCHANG_EVENTS in the C ABI).
MAX_PANCHANG_EVENTS = 50000

# Time kind constants
_TIME_JD_TDB = 0
_TIME_UTC = 1


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


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


def _fill_utc(dst, utc: UtcTime) -> None:
    """Copy Python UtcTime fields into a C DhruvUtcTime struct."""
    dst.year = utc.year
    dst.month = utc.month
    dst.day = utc.day
    dst.hour = utc.hour
    dst.minute = utc.minute
    dst.second = utc.second


def _fill_location(dst, loc: GeoLocation) -> None:
    """Copy Python GeoLocation fields into a C DhruvGeoLocation struct."""
    dst.latitude_deg = loc.lat_deg
    dst.longitude_deg = loc.lon_deg
    dst.altitude_m = loc.alt_m


def _make_utc_c(utc: UtcTime):
    """Create a new C DhruvUtcTime from Python UtcTime."""
    c = ffi.new("DhruvUtcTime *")
    _fill_utc(c[0], utc)
    return c


def _make_location_c(loc: GeoLocation):
    """Create a new C DhruvGeoLocation from Python GeoLocation."""
    c = ffi.new("DhruvGeoLocation *")
    _fill_location(c[0], loc)
    return c


def _tithi_from_c(t) -> TithiInfo:
    return TithiInfo(
        tithi_index=t.tithi_index,
        paksha=t.paksha,
        tithi_in_paksha=t.tithi_in_paksha,
        start=_utc_from_c(t.start),
        end=_utc_from_c(t.end),
    )


def _karana_from_c(k) -> KaranaInfo:
    return KaranaInfo(
        karana_index=k.karana_index,
        karana_name_index=k.karana_name_index,
        start=_utc_from_c(k.start),
        end=_utc_from_c(k.end),
    )


def _yoga_from_c(y) -> YogaInfo:
    return YogaInfo(
        yoga_index=y.yoga_index,
        start=_utc_from_c(y.start),
        end=_utc_from_c(y.end),
    )


def _vaar_from_c(v) -> VaarInfo:
    return VaarInfo(
        vaar_index=v.vaar_index,
        start=_utc_from_c(v.start),
        end=_utc_from_c(v.end),
    )


def _hora_from_c(h) -> HoraInfo:
    return HoraInfo(
        hora_index=h.hora_index,
        hora_position=h.hora_position,
        start=_utc_from_c(h.start),
        end=_utc_from_c(h.end),
    )


def _ghatika_from_c(g) -> GhatikaInfo:
    return GhatikaInfo(
        value=g.value,
        start=_utc_from_c(g.start),
        end=_utc_from_c(g.end),
    )


def _nakshatra_from_c(n) -> PanchangNakshatraInfo:
    return PanchangNakshatraInfo(
        nakshatra_index=n.nakshatra_index,
        pada=n.pada,
        start=_utc_from_c(n.start),
        end=_utc_from_c(n.end),
    )


def _masa_from_c(m) -> MasaInfo:
    return MasaInfo(
        masa_index=m.masa_index,
        adhika=bool(m.adhika),
        start=_utc_from_c(m.start),
        end=_utc_from_c(m.end),
    )


def _ayana_from_c(a) -> AyanaInfo:
    return AyanaInfo(
        ayana=a.ayana,
        start=_utc_from_c(a.start),
        end=_utc_from_c(a.end),
    )


def _varsha_from_c(v) -> VarshaInfo:
    return VarshaInfo(
        samvatsara_index=v.samvatsara_index,
        order=v.order,
        start=_utc_from_c(v.start),
        end=_utc_from_c(v.end),
    )


def _panchang_result_from_c(out) -> PanchangResult:
    """Convert DhruvPanchangOperationResult to PanchangResult."""
    return PanchangResult(
        tithi=_tithi_from_c(out.tithi) if out.tithi_valid else None,
        karana=_karana_from_c(out.karana) if out.karana_valid else None,
        yoga=_yoga_from_c(out.yoga) if out.yoga_valid else None,
        vaar=_vaar_from_c(out.vaar) if out.vaar_valid else None,
        hora=_hora_from_c(out.hora) if out.hora_valid else None,
        ghatika=_ghatika_from_c(out.ghatika) if out.ghatika_valid else None,
        nakshatra=_nakshatra_from_c(out.nakshatra) if out.nakshatra_valid else None,
        masa=_masa_from_c(out.masa) if out.masa_valid else None,
        ayana=_ayana_from_c(out.ayana) if out.ayana_valid else None,
        varsha=_varsha_from_c(out.varsha) if out.varsha_valid else None,
    )


# ---------------------------------------------------------------------------
# Unified panchang (dhruv_panchang_compute_ex)
# ---------------------------------------------------------------------------


def panchang(
    engine,
    eop,
    lsk,
    utc_or_jd: Union[UtcTime, float],
    location: Optional[GeoLocation] = None,
    include_mask: int = INCLUDE_ALL,
    riseset_config=None,
    sankranti_config=None,
) -> PanchangResult:
    """Compute panchang for a given time and (optionally) location.

    Args:
        engine: DhruvEngineHandle pointer.
        eop: DhruvEopHandle pointer.
        lsk: DhruvLskHandle pointer (required when *utc_or_jd* is a JD float).
        utc_or_jd: Either a ``UtcTime`` or a JD TDB float.
        location: Optional observer location.  Required only when
            *include_mask* selects location-dependent elements (vaar, hora,
            ghatika -- ``INCLUDE_LOCATION_DEPENDENT``); requesting those
            without a location raises a ``DhruvError``.
        include_mask: Bitmask of INCLUDE_* constants selecting which fields
            to compute.  Defaults to ``INCLUDE_ALL``.
        riseset_config: Optional ``DhruvRiseSetConfig`` (C struct).  Uses
            library default when ``None``.
        sankranti_config: Optional ``DhruvSankrantiConfig`` (C struct).  Uses
            library default when ``None``.

    Returns:
        A ``PanchangResult`` with requested fields populated.
    """
    req = ffi.new("DhruvPanchangComputeRequest *")
    req.include_mask = include_mask

    if isinstance(utc_or_jd, UtcTime):
        req.time_kind = _TIME_UTC
        _fill_utc(req.utc, utc_or_jd)
    else:
        req.time_kind = _TIME_JD_TDB
        req.jd_tdb = float(utc_or_jd)

    if location is not None:
        req.has_location = 1
        _fill_location(req.location, location)
    else:
        req.has_location = 0

    if riseset_config is not None:
        req.riseset_config = riseset_config
    if sankranti_config is not None:
        req.sankranti_config = sankranti_config
    else:
        req.sankranti_config = lib.dhruv_sankranti_config_default()

    out = ffi.new("DhruvPanchangOperationResult *")
    check(
        lib.dhruv_panchang_compute_ex(engine, eop, lsk, req, out),
        "panchang_compute_ex",
    )
    return _panchang_result_from_c(out[0])


# ---------------------------------------------------------------------------
# Panchang events (range sweep)
# ---------------------------------------------------------------------------


def _collect_events(handle, count_fn, at_fn, ctype, conv, name):
    """Read all rows of one event kind from a panchang events handle."""
    n = ffi.new("uint32_t *")
    check(count_fn(handle, n), f"panchang_events_{name}_count")
    row = ffi.new(ctype)
    items = []
    for i in range(n[0]):
        check(at_fn(handle, i, row), f"panchang_events_{name}_at")
        items.append(conv(row[0]))
    return items


def panchang_events(
    engine,
    eop,
    from_utc: UtcTime,
    to_utc: UtcTime,
    include_mask: int = INCLUDE_LOCATION_INDEPENDENT,
    sankranti_config=None,
    max_events: int = 0,
) -> PanchangEventsResult:
    """Stream exact panchang element segments overlapping [from_utc, to_utc].

    Only location-independent kinds are supported: *include_mask* must be a
    subset of ``INCLUDE_LOCATION_INDEPENDENT`` (tithi, karana, yoga,
    nakshatra, masa, ayana, varsha); any other bit raises ``DhruvError``.

    Consecutive segments of one kind chain exactly (``end == next.start``);
    the first segment of each kind may start before *from_utc* and the last
    may end after *to_utc*.

    Args:
        engine: DhruvEngineHandle pointer.
        eop: DhruvEopHandle pointer.
        from_utc: Range start as ``UtcTime``.
        to_utc: Range end as ``UtcTime`` (must be after *from_utc*).
        include_mask: Bitmask of INCLUDE_* constants selecting kinds.
        sankranti_config: Optional ``DhruvSankrantiConfig`` pointer (e.g.
            ``ffi.new("DhruvSankrantiConfig *", ...)``). Library default
            when ``None``.
        max_events: Cap on total events across all kinds; ``0`` selects the
            hard ceiling ``MAX_PANCHANG_EVENTS`` (50,000).

    Returns:
        ``PanchangEventsResult``. When ``truncated`` is True, resume by
        calling again with ``from_utc=result.next_from`` and deduplicating
        on ``(kind, start)``.
    """
    c_from = _make_utc_c(from_utc)
    c_to = _make_utc_c(to_utc)
    cfg = sankranti_config if sankranti_config is not None else ffi.NULL

    handle = ffi.new("DhruvPanchangEventsHandle *")
    check(
        lib.dhruv_panchang_events(
            engine, eop, c_from, c_to, include_mask, cfg, max_events, handle
        ),
        "panchang_events",
    )
    try:
        h = handle[0]
        tithis = _collect_events(
            h, lib.dhruv_panchang_events_tithi_count,
            lib.dhruv_panchang_events_tithi_at,
            "DhruvTithiInfo *", _tithi_from_c, "tithi",
        )
        karanas = _collect_events(
            h, lib.dhruv_panchang_events_karana_count,
            lib.dhruv_panchang_events_karana_at,
            "DhruvKaranaInfo *", _karana_from_c, "karana",
        )
        yogas = _collect_events(
            h, lib.dhruv_panchang_events_yoga_count,
            lib.dhruv_panchang_events_yoga_at,
            "DhruvYogaInfo *", _yoga_from_c, "yoga",
        )
        nakshatras = _collect_events(
            h, lib.dhruv_panchang_events_nakshatra_count,
            lib.dhruv_panchang_events_nakshatra_at,
            "DhruvPanchangNakshatraInfo *", _nakshatra_from_c, "nakshatra",
        )
        masas = _collect_events(
            h, lib.dhruv_panchang_events_masa_count,
            lib.dhruv_panchang_events_masa_at,
            "DhruvMasaInfo *", _masa_from_c, "masa",
        )
        ayanas = _collect_events(
            h, lib.dhruv_panchang_events_ayana_count,
            lib.dhruv_panchang_events_ayana_at,
            "DhruvAyanaInfo *", _ayana_from_c, "ayana",
        )
        varshas = _collect_events(
            h, lib.dhruv_panchang_events_varsha_count,
            lib.dhruv_panchang_events_varsha_at,
            "DhruvVarshaInfo *", _varsha_from_c, "varsha",
        )

        truncated = ffi.new("uint8_t *")
        next_valid = ffi.new("uint8_t *")
        next_from_c = ffi.new("DhruvUtcTime *")
        check(
            lib.dhruv_panchang_events_meta(h, truncated, next_valid, next_from_c),
            "panchang_events_meta",
        )
        next_from = _utc_from_c(next_from_c[0]) if next_valid[0] else None

        return PanchangEventsResult(
            tithis=tithis,
            karanas=karanas,
            yogas=yogas,
            nakshatras=nakshatras,
            masas=masas,
            ayanas=ayanas,
            varshas=varshas,
            truncated=bool(truncated[0]),
            next_from=next_from,
        )
    finally:
        lib.dhruv_panchang_events_free(handle[0])


# ---------------------------------------------------------------------------
# Individual _for_date functions
# ---------------------------------------------------------------------------


def tithi_for_date(engine, utc: UtcTime) -> TithiInfo:
    """Compute tithi for a UTC date. No config needed."""
    c_utc = _make_utc_c(utc)
    out = ffi.new("DhruvTithiInfo *")
    check(lib.dhruv_tithi_for_date(engine, c_utc, out), "tithi_for_date")
    return _tithi_from_c(out[0])


def karana_for_date(engine, utc: UtcTime) -> KaranaInfo:
    """Compute karana for a UTC date. No config needed."""
    c_utc = _make_utc_c(utc)
    out = ffi.new("DhruvKaranaInfo *")
    check(lib.dhruv_karana_for_date(engine, c_utc, out), "karana_for_date")
    return _karana_from_c(out[0])


def yoga_for_date(engine, utc: UtcTime, config=None) -> YogaInfo:
    """Compute yoga for a UTC date. Requires SankrantiConfig for ayanamsha."""
    c_utc = _make_utc_c(utc)
    if config is None:
        config = ffi.new("DhruvSankrantiConfig *", lib.dhruv_sankranti_config_default())
    out = ffi.new("DhruvYogaInfo *")
    check(lib.dhruv_yoga_for_date(engine, c_utc, config, out), "yoga_for_date")
    return _yoga_from_c(out[0])


def nakshatra_for_date(engine, utc: UtcTime, config=None) -> PanchangNakshatraInfo:
    """Compute Moon's nakshatra for a UTC date. Requires SankrantiConfig for ayanamsha."""
    c_utc = _make_utc_c(utc)
    if config is None:
        config = ffi.new("DhruvSankrantiConfig *", lib.dhruv_sankranti_config_default())
    out = ffi.new("DhruvPanchangNakshatraInfo *")
    check(
        lib.dhruv_nakshatra_for_date(engine, c_utc, config, out),
        "nakshatra_for_date",
    )
    return _nakshatra_from_c(out[0])


def vaar_for_date(
    engine, eop, utc: UtcTime, location: GeoLocation, riseset_config=None
) -> VaarInfo:
    """Compute vaar (weekday) for a UTC date and location."""
    c_utc = _make_utc_c(utc)
    c_loc = _make_location_c(location)
    rs_cfg = riseset_config if riseset_config is not None else ffi.NULL
    out = ffi.new("DhruvVaarInfo *")
    check(
        lib.dhruv_vaar_for_date(engine, eop, c_utc, c_loc, rs_cfg, out),
        "vaar_for_date",
    )
    return _vaar_from_c(out[0])


def hora_for_date(
    engine, eop, utc: UtcTime, location: GeoLocation, riseset_config=None
) -> HoraInfo:
    """Compute hora (planetary hour) for a UTC date and location."""
    c_utc = _make_utc_c(utc)
    c_loc = _make_location_c(location)
    rs_cfg = riseset_config if riseset_config is not None else ffi.NULL
    out = ffi.new("DhruvHoraInfo *")
    check(
        lib.dhruv_hora_for_date(engine, eop, c_utc, c_loc, rs_cfg, out),
        "hora_for_date",
    )
    return _hora_from_c(out[0])


def ghatika_for_date(
    engine, eop, utc: UtcTime, location: GeoLocation, riseset_config=None
) -> GhatikaInfo:
    """Compute ghatika for a UTC date and location."""
    c_utc = _make_utc_c(utc)
    c_loc = _make_location_c(location)
    rs_cfg = riseset_config if riseset_config is not None else ffi.NULL
    out = ffi.new("DhruvGhatikaInfo *")
    check(
        lib.dhruv_ghatika_for_date(engine, eop, c_utc, c_loc, rs_cfg, out),
        "ghatika_for_date",
    )
    return _ghatika_from_c(out[0])


def masa_for_date(engine, utc: UtcTime, config=None) -> MasaInfo:
    """Compute masa (lunar month) for a UTC date."""
    c_utc = _make_utc_c(utc)
    if config is None:
        config = ffi.new("DhruvSankrantiConfig *", lib.dhruv_sankranti_config_default())
    out = ffi.new("DhruvMasaInfo *")
    check(lib.dhruv_masa_for_date(engine, c_utc, config, out), "masa_for_date")
    return _masa_from_c(out[0])


def ayana_for_date(engine, utc: UtcTime, config=None) -> AyanaInfo:
    """Compute ayana (solstice period) for a UTC date."""
    c_utc = _make_utc_c(utc)
    if config is None:
        config = ffi.new("DhruvSankrantiConfig *", lib.dhruv_sankranti_config_default())
    out = ffi.new("DhruvAyanaInfo *")
    check(lib.dhruv_ayana_for_date(engine, c_utc, config, out), "ayana_for_date")
    return _ayana_from_c(out[0])


def varsha_for_date(engine, utc: UtcTime, config=None) -> VarshaInfo:
    """Compute varsha (Jovian year) for a UTC date."""
    c_utc = _make_utc_c(utc)
    if config is None:
        config = ffi.new("DhruvSankrantiConfig *", lib.dhruv_sankranti_config_default())
    out = ffi.new("DhruvVarshaInfo *")
    check(lib.dhruv_varsha_for_date(engine, c_utc, config, out), "varsha_for_date")
    return _varsha_from_c(out[0])


# ---------------------------------------------------------------------------
# Nakshatra at (dhruv_nakshatra_at)
# ---------------------------------------------------------------------------


def nakshatra_at(
    engine,
    jd_tdb: float,
    moon_sidereal_deg: float,
    config=None,
) -> PanchangNakshatraInfo:
    """Determine Moon's nakshatra from a pre-computed sidereal longitude.

    The engine is still needed for boundary bisection (start/end times).

    Args:
        engine: DhruvEngineHandle pointer.
        jd_tdb: Julian date (TDB).
        moon_sidereal_deg: Moon's sidereal longitude [0, 360).
        config: Optional DhruvSankrantiConfig pointer. Uses default if None.

    Returns:
        PanchangNakshatraInfo with nakshatra index, pada, and time boundaries.
    """
    if config is None:
        config = ffi.new("DhruvSankrantiConfig *", lib.dhruv_sankranti_config_default())
    out = ffi.new("DhruvPanchangNakshatraInfo *")
    check(
        lib.dhruv_nakshatra_at(engine, jd_tdb, moon_sidereal_deg, config, out),
        "nakshatra_at",
    )
    return _nakshatra_from_c(out[0])


# ---------------------------------------------------------------------------
# Samvatsara (dhruv_samvatsara_from_year)
# ---------------------------------------------------------------------------


def samvatsara_from_year(ce_year: int) -> SamvatsaraResult:
    """Determine Samvatsara (Jovian year) from a CE year.

    Pure math, no engine needed.

    Args:
        ce_year: Common Era year (e.g. 2025).

    Returns:
        SamvatsaraResult with 0-based index and 1-based cycle position.
    """
    out = ffi.new("DhruvSamvatsaraResult *")
    check(lib.dhruv_samvatsara_from_year(ce_year, out), "samvatsara_from_year")
    return SamvatsaraResult(
        samvatsara_index=out.samvatsara_index,
        cycle_position=out.cycle_position,
    )


# ---------------------------------------------------------------------------
# Composable intermediates (JD-based)
# ---------------------------------------------------------------------------


def elongation_at(engine, jd_tdb: float) -> float:
    """Compute Sun-Moon elongation at a JD TDB instant.

    Args:
        engine: DhruvEngineHandle pointer.
        jd_tdb: Julian date in TDB.

    Returns:
        Elongation in degrees.
    """
    out = ffi.new("double *")
    check(lib.dhruv_elongation_at(engine, jd_tdb, out), "elongation_at")
    return out[0]


def tithi_at(engine, jd_tdb: float, elongation_deg: float) -> TithiInfo:
    """Compute tithi from a pre-computed Sun-Moon elongation.

    Args:
        engine: DhruvEngineHandle pointer.
        jd_tdb: Julian date in TDB.
        elongation_deg: Pre-computed elongation (from ``elongation_at``).

    Returns:
        TithiInfo with index, paksha, start/end times.
    """
    out = ffi.new("DhruvTithiInfo *")
    check(lib.dhruv_tithi_at(engine, jd_tdb, elongation_deg, out), "tithi_at")
    return _tithi_from_c(out[0])


def karana_at(engine, jd_tdb: float, elongation_deg: float) -> KaranaInfo:
    """Compute karana from a pre-computed Sun-Moon elongation.

    Args:
        engine: DhruvEngineHandle pointer.
        jd_tdb: Julian date in TDB.
        elongation_deg: Pre-computed elongation (from ``elongation_at``).

    Returns:
        KaranaInfo with index, name index, start/end times.
    """
    out = ffi.new("DhruvKaranaInfo *")
    check(
        lib.dhruv_karana_at(engine, jd_tdb, elongation_deg, out), "karana_at"
    )
    return _karana_from_c(out[0])


def yoga_at(engine, jd_tdb: float, sidereal_sum_deg: float,
            config=None) -> YogaInfo:
    """Compute yoga from a pre-computed sidereal sum (Sun + Moon).

    Args:
        engine: DhruvEngineHandle pointer.
        jd_tdb: Julian date in TDB.
        sidereal_sum_deg: Pre-computed sidereal sum (from
            ``ayanamsha.sidereal_sum_at``).
        config: Optional DhruvSankrantiConfig pointer. Uses default if None.

    Returns:
        YogaInfo with index, start/end times.
    """
    if config is None:
        config = ffi.new(
            "DhruvSankrantiConfig *", lib.dhruv_sankranti_config_default()
        )
    out = ffi.new("DhruvYogaInfo *")
    check(
        lib.dhruv_yoga_at(engine, jd_tdb, sidereal_sum_deg, config, out),
        "yoga_at",
    )
    return _yoga_from_c(out[0])


# ---------------------------------------------------------------------------
# From-sunrises helpers (pre-computed sunrise pair)
# ---------------------------------------------------------------------------


def vaar_from_sunrises(lsk, sunrise_jd: float,
                       next_sunrise_jd: float) -> VaarInfo:
    """Compute vaar from a pre-computed sunrise pair.

    Args:
        lsk: DhruvLskHandle pointer.
        sunrise_jd: JD of today's sunrise.
        next_sunrise_jd: JD of tomorrow's sunrise.

    Returns:
        VaarInfo with weekday index, start/end times.
    """
    out = ffi.new("DhruvVaarInfo *")
    check(
        lib.dhruv_vaar_from_sunrises(lsk, sunrise_jd, next_sunrise_jd, out),
        "vaar_from_sunrises",
    )
    return _vaar_from_c(out[0])


def hora_from_sunrises(lsk, query_jd: float, sunrise_jd: float,
                       next_sunrise_jd: float) -> HoraInfo:
    """Compute hora from a pre-computed sunrise pair.

    Args:
        lsk: DhruvLskHandle pointer.
        query_jd: JD of the query instant.
        sunrise_jd: JD of today's sunrise.
        next_sunrise_jd: JD of tomorrow's sunrise.

    Returns:
        HoraInfo with hora index, position, start/end times.
    """
    out = ffi.new("DhruvHoraInfo *")
    check(
        lib.dhruv_hora_from_sunrises(
            lsk, query_jd, sunrise_jd, next_sunrise_jd, out
        ),
        "hora_from_sunrises",
    )
    return _hora_from_c(out[0])


def ghatika_from_sunrises(lsk, query_jd: float, sunrise_jd: float,
                          next_sunrise_jd: float) -> GhatikaInfo:
    """Compute ghatika from a pre-computed sunrise pair.

    Args:
        lsk: DhruvLskHandle pointer.
        query_jd: JD of the query instant.
        sunrise_jd: JD of today's sunrise.
        next_sunrise_jd: JD of tomorrow's sunrise.

    Returns:
        GhatikaInfo with value, start/end times.
    """
    out = ffi.new("DhruvGhatikaInfo *")
    check(
        lib.dhruv_ghatika_from_sunrises(
            lsk, query_jd, sunrise_jd, next_sunrise_jd, out
        ),
        "ghatika_from_sunrises",
    )
    return _ghatika_from_c(out[0])
