"""Amsha (divisional chart) computation.

Pure-math amsha transforms and engine-backed amsha chart orchestration.
"""

from __future__ import annotations

from ._ffi import ffi, lib
from ._check import check
from .types import (
    AmshaChart,
    AmshaEntry,
    AmshaLagnaEntry,
    AmshaLagnaEventsResult,
    AmshaLagnaSegment,
    AmshaSeriesChart,
    AmshaSeriesPoint,
    AmshaVariationCatalog,
    AmshaVariationInfo,
    Dms,
    RashiInfo,
    UtcTime,
)


# Hard ceilings matching the C ABI constants.
# points * unique requests per amsha_series call:
MAX_AMSHA_SERIES_CELLS = 100000
# total segments across all amshas per amsha_lagna_events call:
MAX_AMSHA_LAGNA_SEGMENTS = 50000

# Amsha point families. Each code names one AmshaChart section; a point's
# identity is (family, index within that section).
AMSHA_POINT_FAMILY_LAGNA = 0
AMSHA_POINT_FAMILY_GRAHA = 1
AMSHA_POINT_FAMILY_OUTER_PLANET = 2
AMSHA_POINT_FAMILY_BHAVA_CUSP = 3
AMSHA_POINT_FAMILY_RASHI_BHAVA_CUSP = 4
AMSHA_POINT_FAMILY_ARUDHA_PADA = 5
AMSHA_POINT_FAMILY_RASHI_BHAVA_ARUDHA_PADA = 6
AMSHA_POINT_FAMILY_UPAGRAHA = 7
AMSHA_POINT_FAMILY_SPHUTA = 8
AMSHA_POINT_FAMILY_SPECIAL_LAGNA = 9
AMSHA_POINT_FAMILY_COUNT = 10


def amsha_point_count(family):
    """Number of points in an amsha point family; 0 for an unknown family."""
    return int(lib.dhruv_amsha_point_count(family))


def amsha_point_name(family, index):
    """Display name of the point at (family, index), or None if out of range."""
    ptr = lib.dhruv_amsha_point_name(family, index)
    if ptr == ffi.NULL:
        return None
    return ffi.string(ptr).decode("utf-8")


def amsha_point_key(family, index):
    """Stable snake_case key of the point at (family, index), or None."""
    ptr = lib.dhruv_amsha_point_key(family, index)
    if ptr == ffi.NULL:
        return None
    return ffi.string(ptr).decode("utf-8")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_utc(jd_utc):
    utc = ffi.new("DhruvUtcTime *")
    utc.year = jd_utc[0]
    utc.month = jd_utc[1]
    utc.day = jd_utc[2]
    utc.hour = jd_utc[3] if len(jd_utc) > 3 else 0
    utc.minute = jd_utc[4] if len(jd_utc) > 4 else 0
    utc.second = jd_utc[5] if len(jd_utc) > 5 else 0.0
    return utc


def _make_location(location):
    loc = ffi.new("DhruvGeoLocation *")
    loc.latitude_deg = location[0]
    loc.longitude_deg = location[1]
    loc.altitude_m = location[2] if len(location) > 2 else 0.0
    return loc


def _make_bhava_config(bhava_config):
    if bhava_config is None:
        return ffi.NULL
    cfg = ffi.new("DhruvBhavaConfig *")
    cfg.system = bhava_config.get("system", 0)
    cfg.starting_point = bhava_config.get("starting_point", -1)
    cfg.custom_start_deg = bhava_config.get("custom_start_deg", 0.0)
    cfg.reference_mode = bhava_config.get("reference_mode", 0)
    cfg.use_rashi_bhava_for_bala_avastha = bhava_config.get(
        "use_rashi_bhava_for_bala_avastha", 1
    )
    cfg.include_node_aspects_for_drik_bala = bhava_config.get(
        "include_node_aspects_for_drik_bala", 0
    )
    cfg.include_special_bhavabala_rules = bhava_config.get("include_special_bhavabala_rules", 1)
    cfg.divide_guru_buddh_drishti_by_4_for_drik_bala = bhava_config.get(
        "divide_guru_buddh_drishti_by_4_for_drik_bala", 1
    )
    cfg.chandra_benefic_rule = bhava_config.get("chandra_benefic_rule", 0)
    cfg.sayanadi_ghatika_rounding = bhava_config.get("sayanadi_ghatika_rounding", 0)
    cfg.include_rashi_bhava_results = bhava_config.get("include_rashi_bhava_results", 1)
    return cfg


def _make_riseset_config(riseset_config):
    if riseset_config is None:
        return ffi.NULL
    cfg = ffi.new("DhruvRiseSetConfig *")
    cfg.use_refraction = riseset_config.get("use_refraction", 1)
    cfg.sun_limb = riseset_config.get("sun_limb", 0)
    cfg.altitude_correction = riseset_config.get("altitude_correction", 0)
    return cfg


def _extract_amsha_entry(e, family=AMSHA_POINT_FAMILY_LAGNA, point_index=0):
    return AmshaEntry(
        sidereal_longitude=e.sidereal_longitude,
        rashi_index=e.rashi_index,
        dms_degrees=e.dms_degrees,
        dms_minutes=e.dms_minutes,
        dms_seconds=e.dms_seconds,
        degrees_in_rashi=e.degrees_in_rashi,
        nakshatra_index=e.nakshatra_index,
        pada=e.pada,
        rashi_bhava_number=e.rashi_bhava_number,
        family=family,
        point_index=point_index,
        name=amsha_point_key(family, point_index),
        display_name=amsha_point_name(family, point_index),
    )


def _extract_amsha_family(entries, family, count):
    """Extract a whole point family, tagging each entry with its position."""
    return [_extract_amsha_entry(entries[i], family, i) for i in range(count)]


def _decode_c_string(buf):
    return ffi.string(buf).decode("utf-8")


def _extract_amsha_variation_catalog(catalog):
    variations = []
    for i in range(catalog.count):
        info = catalog.variations[i]
        variations.append(
            AmshaVariationInfo(
                amsha_code=info.amsha_code,
                variation_code=info.variation_code,
                name=_decode_c_string(info.name),
                label=_decode_c_string(info.label),
                is_default=bool(info.is_default),
                description=_decode_c_string(info.description),
            )
        )
    return AmshaVariationCatalog(
        amsha_code=catalog.amsha_code,
        default_variation_code=catalog.default_variation_code,
        variations=variations,
    )


# ---------------------------------------------------------------------------
# Pure math: single longitude
# ---------------------------------------------------------------------------


def amsha_longitude(sidereal_lon_deg, amsha_number, variation=0):
    """Compute amsha longitude for a single sidereal longitude.

    Args:
        sidereal_lon_deg: Sidereal longitude in degrees [0, 360).
        amsha_number: D-number (e.g. 9 for Navamsha, 12 for Dwadashamsha).
        variation: amsha-specific variation code; 0=default for that amsha.

    Returns:
        Amsha longitude in degrees [0, 360).
    """
    out = ffi.new("double *")
    check(
        lib.dhruv_amsha_longitude(sidereal_lon_deg, amsha_number, variation, out),
        "amsha_longitude",
    )
    return out[0]


# ---------------------------------------------------------------------------
# Pure math: batch longitudes
# ---------------------------------------------------------------------------


def amsha_longitudes(sidereal_lons, amsha_codes, variation_codes=None):
    """Compute amsha longitudes for multiple points and/or multiple amshas.

    This is a low-level batch function. For each index i, it transforms
    sidereal_lons[0] through amsha_codes[i] (the FFI function takes a single
    longitude and array of codes).

    For 9 grahas + lagna, call once per amsha with individual longitudes,
    or use amsha_chart_for_date for the full orchestration.

    Args:
        sidereal_lons: Single sidereal longitude in degrees (scalar float).
        amsha_codes: List of D-numbers (u16).
        variation_codes: Optional list of amsha-specific variation codes (u8),
                        same length as amsha_codes. None = all default.

    Returns:
        List of amsha longitudes (one per amsha_code).
    """
    count = len(amsha_codes)
    c_codes = ffi.new("uint16_t[]", count)
    for i, code in enumerate(amsha_codes):
        c_codes[i] = code

    c_variations = ffi.NULL
    if variation_codes is not None:
        c_variations = ffi.new("uint8_t[]", count)
        for i, vc in enumerate(variation_codes):
            c_variations[i] = vc

    c_out = ffi.new("double[]", count)
    check(
        lib.dhruv_amsha_longitudes(sidereal_lons, c_codes, c_variations, count, c_out),
        "amsha_longitudes",
    )
    return [c_out[i] for i in range(count)]


# ---------------------------------------------------------------------------
# Pure math: rashi info
# ---------------------------------------------------------------------------


def amsha_rashi_info(sidereal_lon_deg, amsha_number, variation=0):
    """Get rashi info for an amsha longitude.

    Args:
        sidereal_lon_deg: Sidereal longitude in degrees.
        amsha_number: D-number.
        variation: Amsha-specific variation code.

    Returns:
        RashiInfo dataclass.
    """
    out = ffi.new("DhruvRashiInfo *")
    check(
        lib.dhruv_amsha_rashi_info(sidereal_lon_deg, amsha_number, variation, out),
        "amsha_rashi_info",
    )
    return RashiInfo(
        rashi_index=out.rashi_index,
        degrees_in_rashi=out.degrees_in_rashi,
        dms=Dms(
            degrees=out.dms.degrees,
            minutes=out.dms.minutes,
            seconds=out.dms.seconds,
        ),
    )


# ---------------------------------------------------------------------------
# Orchestration: amsha chart for date
# ---------------------------------------------------------------------------


def amsha_chart_for_date(
    engine,
    lsk,
    eop,
    jd_utc,
    location,
    amsha_code,
    variation=0,
    ayanamsha_system=0,
    use_nutation=1,
    scope=None,
    bhava_config=None,
    riseset_config=None,
):
    """Compute a single amsha (divisional) chart for a date and location.

    Args:
        engine: Engine instance.
        lsk: LSK handle.
        eop: EOP handle.
        jd_utc: UTC time tuple (year, month, day[, hour, min, sec]).
        location: (lat, lon[, alt]) tuple.
        amsha_code: D-number (e.g. 9 for Navamsha).
        variation: Amsha-specific variation code (0=default for that amsha).
        ayanamsha_system: Ayanamsha system code.
        use_nutation: 1=yes, 0=no.
        scope: Optional dict with include_bhava_cusps, include_arudha_padas,
               include_upagrahas, include_sphutas, include_special_lagnas (u8).
        bhava_config: Optional bhava config dict.
        riseset_config: Optional riseset config dict.

    Returns:
        AmshaChart dataclass.
    """
    utc = _make_utc(jd_utc)
    loc = _make_location(location)
    bhava_cfg = _make_bhava_config(bhava_config)
    rs_cfg = _make_riseset_config(riseset_config)

    scope_c = ffi.new("DhruvAmshaChartScope *")
    if scope is not None:
        scope_c.include_bhava_cusps = scope.get("include_bhava_cusps", 0)
        scope_c.include_arudha_padas = scope.get("include_arudha_padas", 0)
        scope_c.include_upagrahas = scope.get("include_upagrahas", 0)
        scope_c.include_sphutas = scope.get("include_sphutas", 0)
        scope_c.include_special_lagnas = scope.get("include_special_lagnas", 0)
        scope_c.include_outer_planets = scope.get("include_outer_planets", 1)
    else:
        scope_c.include_outer_planets = 1

    out = ffi.new("DhruvAmshaChart *")
    check(
        lib.dhruv_amsha_chart_for_date(
            engine._ptr,
            eop,
            utc,
            loc,
            bhava_cfg,
            rs_cfg,
            ayanamsha_system,
            use_nutation,
            amsha_code,
            variation,
            scope_c,
            out,
        ),
        "amsha_chart_for_date",
    )

    grahas = _extract_amsha_family(out.grahas, AMSHA_POINT_FAMILY_GRAHA, 9)
    outer_planets = None
    if out.outer_planets_valid:
        outer_planets = _extract_amsha_family(
            out.outer_planets, AMSHA_POINT_FAMILY_OUTER_PLANET, 3
        )
    lagna = _extract_amsha_entry(out.lagna, AMSHA_POINT_FAMILY_LAGNA, 0)

    bhava_cusps = None
    if out.bhava_cusps_valid:
        bhava_cusps = _extract_amsha_family(
            out.bhava_cusps, AMSHA_POINT_FAMILY_BHAVA_CUSP, 12
        )

    rashi_bhava_cusps = None
    if out.rashi_bhava_cusps_valid:
        rashi_bhava_cusps = _extract_amsha_family(
            out.rashi_bhava_cusps, AMSHA_POINT_FAMILY_RASHI_BHAVA_CUSP, 12
        )

    arudha_padas = None
    if out.arudha_padas_valid:
        arudha_padas = _extract_amsha_family(
            out.arudha_padas, AMSHA_POINT_FAMILY_ARUDHA_PADA, 12
        )

    rashi_bhava_arudha_padas = None
    if out.rashi_bhava_arudha_padas_valid:
        rashi_bhava_arudha_padas = _extract_amsha_family(
            out.rashi_bhava_arudha_padas, AMSHA_POINT_FAMILY_RASHI_BHAVA_ARUDHA_PADA, 12
        )

    upagrahas = None
    if out.upagrahas_valid:
        upagrahas = _extract_amsha_family(out.upagrahas, AMSHA_POINT_FAMILY_UPAGRAHA, 11)

    sphutas = None
    if out.sphutas_valid:
        sphutas = _extract_amsha_family(out.sphutas, AMSHA_POINT_FAMILY_SPHUTA, 16)

    special_lagnas = None
    if out.special_lagnas_valid:
        special_lagnas = _extract_amsha_family(
            out.special_lagnas, AMSHA_POINT_FAMILY_SPECIAL_LAGNA, 8
        )

    return AmshaChart(
        amsha_code=out.amsha_code,
        variation_code=out.variation_code,
        grahas=grahas,
        lagna=lagna,
        outer_planets=outer_planets,
        bhava_cusps=bhava_cusps,
        rashi_bhava_cusps=rashi_bhava_cusps,
        arudha_padas=arudha_padas,
        rashi_bhava_arudha_padas=rashi_bhava_arudha_padas,
        upagrahas=upagrahas,
        sphutas=sphutas,
        special_lagnas=special_lagnas,
    )


def amsha_variations(amsha_code):
    """List supported variations for a single amsha."""
    out = ffi.new("DhruvAmshaVariationList *")
    check(lib.dhruv_amsha_variations(amsha_code, out), "amsha_variations")
    return _extract_amsha_variation_catalog(out[0])


def amsha_variations_many(amsha_codes):
    """List supported variations for multiple amshas."""
    count = len(amsha_codes)
    c_codes = ffi.new("uint16_t[]", count)
    for i, code in enumerate(amsha_codes):
        c_codes[i] = code
    out = ffi.new("DhruvAmshaVariationCatalogs *")
    check(
        lib.dhruv_amsha_variations_many(c_codes, count, out),
        "amsha_variations_many",
    )
    return [_extract_amsha_variation_catalog(out.lists[i]) for i in range(out.count)]


# ---------------------------------------------------------------------------
# Range operations: amsha series and amsha lagna events
# ---------------------------------------------------------------------------


def _utc_from_c(u) -> UtcTime:
    return UtcTime(
        year=u.year,
        month=u.month,
        day=u.day,
        hour=u.hour,
        minute=u.minute,
        second=u.second,
    )


def _make_amsha_requests(amsha_codes, variation_codes):
    """Build (c_amsha_codes, c_variation_codes, count) request arrays."""
    count = len(amsha_codes)
    c_codes = ffi.new("uint16_t[]", max(count, 1))
    for i, code in enumerate(amsha_codes):
        c_codes[i] = code
    c_variations = ffi.NULL
    if variation_codes is not None:
        if len(variation_codes) != count:
            raise ValueError(
                "variation_codes must have the same length as amsha_codes"
            )
        c_variations = ffi.new("uint8_t[]", max(count, 1))
        for i, vc in enumerate(variation_codes):
            c_variations[i] = vc
    return c_codes, c_variations, count


def amsha_series(
    engine,
    eop,
    from_utc,
    to_utc,
    step_minutes,
    location,
    amsha_codes,
    variation_codes=None,
    include_grahas=True,
    sankranti_config=None,
):
    """Sample slim varga charts at a fixed cadence over [from_utc, to_utc].

    Grid semantics match ``graha_positions_series``: one point per
    *step_minutes* starting at *from_utc*, endpoints inclusive when they
    fall on the grid. Each point carries one chart per request, in request
    order (duplicate requests repeated). The varga lagna is always computed;
    graha entries are added when *include_grahas* is true.

    Rejects ``step_minutes == 0``, reversed ranges, empty or invalid request
    lists, and grids whose points x unique requests exceed
    ``MAX_AMSHA_SERIES_CELLS`` (100,000).

    Args:
        engine: Engine instance.
        eop: EOP handle.
        from_utc: Range start as (year, month, day[, hour, min, sec]) tuple.
        to_utc: Range end tuple (must be after from_utc).
        step_minutes: Sampling cadence in minutes (>= 1).
        location: (lat_deg, lon_deg[, alt_m]) tuple.
        amsha_codes: List of D-numbers (e.g. [1, 9, 12]).
        variation_codes: Optional list of amsha-specific variation codes,
            same length as amsha_codes. None = all default.
        include_grahas: When true, each chart carries the 9 graha entries in
            addition to the varga lagna.
        sankranti_config: Optional ``DhruvSankrantiConfig`` pointer.
            Library default when ``None``.

    Returns:
        List of ``AmshaSeriesPoint`` dataclasses.
    """
    from_c = _make_utc(from_utc)
    to_c = _make_utc(to_utc)
    loc = _make_location(location)
    cfg = sankranti_config if sankranti_config is not None else ffi.NULL
    c_codes, c_variations, count = _make_amsha_requests(amsha_codes, variation_codes)

    handle = ffi.new("DhruvAmshaSeriesHandle *")
    check(
        lib.dhruv_amsha_series(
            engine._ptr,
            eop,
            from_c,
            to_c,
            step_minutes,
            loc,
            cfg,
            c_codes,
            c_variations,
            count,
            1 if include_grahas else 0,
            handle,
        ),
        "amsha_series",
    )
    try:
        h = handle[0]
        point_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_amsha_series_point_count(h, point_count),
            "amsha_series_point_count",
        )
        chart_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_amsha_series_chart_count(h, chart_count),
            "amsha_series_chart_count",
        )

        points = []
        utc_c = ffi.new("DhruvUtcTime *")
        jd_c = ffi.new("double *")
        chart_c = ffi.new("DhruvAmshaSeriesChart *")
        for i in range(point_count[0]):
            check(
                lib.dhruv_amsha_series_point_at(h, i, utc_c, jd_c),
                "amsha_series_point_at",
            )
            charts = []
            for j in range(chart_count[0]):
                check(
                    lib.dhruv_amsha_series_chart_at(h, i, j, chart_c),
                    "amsha_series_chart_at",
                )
                grahas = None
                if chart_c.grahas_valid:
                    grahas = [_extract_amsha_entry(chart_c.grahas[k]) for k in range(9)]
                charts.append(
                    AmshaSeriesChart(
                        amsha_code=chart_c.amsha_code,
                        variation_code=chart_c.variation_code,
                        lagna=_extract_amsha_entry(chart_c.lagna),
                        grahas=grahas,
                    )
                )
            points.append(
                AmshaSeriesPoint(
                    utc=(
                        utc_c.year,
                        utc_c.month,
                        utc_c.day,
                        utc_c.hour,
                        utc_c.minute,
                        utc_c.second,
                    ),
                    jd_utc=jd_c[0],
                    charts=charts,
                )
            )
        return points
    finally:
        lib.dhruv_amsha_series_free(handle[0])


def amsha_lagna_events(
    engine,
    eop,
    from_utc,
    to_utc,
    location,
    amsha_codes,
    variation_codes=None,
    max_segments=0,
    sankranti_config=None,
):
    """Stream exact varga-lagna rashi segments overlapping [from_utc, to_utc].

    Returns one entry per unique (amsha, variation) request (duplicates
    collapsed), in request order. Segments carry exact varga-lagna transition
    boundaries (no sampling grid) and chain exactly within an entry
    (``segment.end == next_segment.start``).

    Args:
        engine: Engine instance.
        eop: EOP handle.
        from_utc: Range start as (year, month, day[, hour, min, sec]) tuple.
        to_utc: Range end tuple (must be after from_utc).
        location: (lat_deg, lon_deg[, alt_m]) tuple.
        amsha_codes: List of D-numbers (e.g. [1, 9]).
        variation_codes: Optional list of amsha-specific variation codes,
            same length as amsha_codes. None = all default.
        max_segments: Cap on total segments across all amshas; ``0`` selects
            the hard ceiling ``MAX_AMSHA_LAGNA_SEGMENTS`` (50,000).
        sankranti_config: Optional ``DhruvSankrantiConfig`` pointer.
            Library default when ``None``.

    Returns:
        ``AmshaLagnaEventsResult``. When ``truncated`` is True, resume by
        calling again with ``from_utc=result.next_from`` and deduplicating
        on segment starts.
    """
    from_c = _make_utc(from_utc)
    to_c = _make_utc(to_utc)
    loc = _make_location(location)
    cfg = sankranti_config if sankranti_config is not None else ffi.NULL
    c_codes, c_variations, count = _make_amsha_requests(amsha_codes, variation_codes)

    handle = ffi.new("DhruvAmshaLagnaEventsHandle *")
    check(
        lib.dhruv_amsha_lagna_events(
            engine._ptr,
            eop,
            from_c,
            to_c,
            loc,
            cfg,
            c_codes,
            c_variations,
            count,
            max_segments,
            handle,
        ),
        "amsha_lagna_events",
    )
    try:
        h = handle[0]
        entry_count = ffi.new("uint32_t *")
        check(
            lib.dhruv_amsha_lagna_events_entry_count(h, entry_count),
            "amsha_lagna_events_entry_count",
        )

        entries = []
        amsha_code_c = ffi.new("uint16_t *")
        variation_code_c = ffi.new("uint8_t *")
        segment_count = ffi.new("uint32_t *")
        segment_c = ffi.new("DhruvAmshaLagnaSegment *")
        for i in range(entry_count[0]):
            check(
                lib.dhruv_amsha_lagna_events_entry_info(
                    h, i, amsha_code_c, variation_code_c
                ),
                "amsha_lagna_events_entry_info",
            )
            check(
                lib.dhruv_amsha_lagna_events_segment_count(h, i, segment_count),
                "amsha_lagna_events_segment_count",
            )
            segments = []
            for j in range(segment_count[0]):
                check(
                    lib.dhruv_amsha_lagna_events_segment_at(h, i, j, segment_c),
                    "amsha_lagna_events_segment_at",
                )
                segments.append(
                    AmshaLagnaSegment(
                        rashi_index=segment_c.rashi_index,
                        start=_utc_from_c(segment_c.start),
                        end=_utc_from_c(segment_c.end),
                    )
                )
            entries.append(
                AmshaLagnaEntry(
                    amsha_code=amsha_code_c[0],
                    variation_code=variation_code_c[0],
                    segments=segments,
                )
            )

        truncated = ffi.new("uint8_t *")
        next_valid = ffi.new("uint8_t *")
        next_from_c = ffi.new("DhruvUtcTime *")
        check(
            lib.dhruv_amsha_lagna_events_meta(h, truncated, next_valid, next_from_c),
            "amsha_lagna_events_meta",
        )
        next_from = _utc_from_c(next_from_c[0]) if next_valid[0] else None

        return AmshaLagnaEventsResult(
            entries=entries,
            truncated=bool(truncated[0]),
            next_from=next_from,
        )
    finally:
        lib.dhruv_amsha_lagna_events_free(handle[0])
