"""
CFFI C-declaration string for ctara-dhruv-core.

Content derived from ``crates/dhruv_ffi_c/include/dhruv.h`` -- the canonical C
header shared by Python, Go, and Node bindings.  Do NOT edit the embedded
header content manually; update dhruv.h and re-generate with::

    python3 scripts/gen_cdef.py
"""

from __future__ import annotations

import re as _re

# ---------------------------------------------------------------------------
# Raw header content -- verbatim copy of dhruv.h
# ---------------------------------------------------------------------------
_RAW_HEADER: str = r"""
/*
 * dhruv.h -- Canonical C header for ctara-dhruv-core FFI
 *
 * SPDX-License-Identifier: MIT
 *
 * This file mirrors every #[repr(C)] struct and #[unsafe(no_mangle)]
 * function exported by dhruv_ffi_c.  Keep it in sync with lib.rs and
 * bindings/python-open/src/ctara_dhruv/_cdef.py.
 */

#ifndef DHRUV_H
#define DHRUV_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ===================================================================
 * Constants
 * =================================================================== */

/* API version */
#define DHRUV_API_VERSION       87
#define DHRUV_PATH_CAPACITY     512
#define DHRUV_MAX_SPK_PATHS     8
#define DHRUV_MAX_AMSHA_VARIATIONS 16
#define DHRUV_MAX_OSCULATING_APOGEE_REQUESTS 32
#define DHRUV_MAX_CONJUNCTION_TARGETS 16
#define DHRUV_AMSHA_VARIATION_NAME_CAPACITY 48
#define DHRUV_AMSHA_VARIATION_LABEL_CAPACITY 64
#define DHRUV_AMSHA_VARIATION_DESCRIPTION_CAPACITY 160
#define DHRUV_GOCHAR_NAME_CAPACITY 128
#define DHRUV_GOCHAR_TRANSIT_RAHU 10007
#define DHRUV_GOCHAR_TRANSIT_KETU 10008

/* DhruvStatus (repr(i32)) */
typedef int32_t DhruvStatus;

/* Status codes */
#define DHRUV_STATUS_OK                  0
#define DHRUV_STATUS_INVALID_CONFIG      1
#define DHRUV_STATUS_INVALID_QUERY       2
#define DHRUV_STATUS_KERNEL_LOAD         3
#define DHRUV_STATUS_TIME_CONVERSION     4
#define DHRUV_STATUS_UNSUPPORTED_QUERY   5
#define DHRUV_STATUS_EPOCH_OUT_OF_RANGE  6
#define DHRUV_STATUS_NULL_POINTER        7
#define DHRUV_STATUS_EOP_LOAD            8
#define DHRUV_STATUS_EOP_OUT_OF_RANGE    9
#define DHRUV_STATUS_INVALID_LOCATION   10
#define DHRUV_STATUS_NO_CONVERGENCE     11
#define DHRUV_STATUS_INVALID_SEARCH_CONFIG 12
#define DHRUV_STATUS_INVALID_INPUT      13
#define DHRUV_STATUS_INTERNAL          255

/* DhruvReferencePlane (repr(i32)) */
#define DHRUV_REFERENCE_PLANE_ECLIPTIC   0
#define DHRUV_REFERENCE_PLANE_INVARIABLE 1

/* Precession model selector */
#define DHRUV_PRECESSION_MODEL_NEWCOMB1895 0
#define DHRUV_PRECESSION_MODEL_LIESKE1977  1
#define DHRUV_PRECESSION_MODEL_IAU2006     2
#define DHRUV_PRECESSION_MODEL_VONDRAK2011 3

/* Graha longitude selector */
#define DHRUV_GRAHA_LONGITUDE_KIND_SIDEREAL 0
#define DHRUV_GRAHA_LONGITUDE_KIND_TROPICAL 1

/* Query time selector */
#define DHRUV_QUERY_TIME_JD_TDB 0
#define DHRUV_QUERY_TIME_UTC    1

/* Search time selector */
#define DHRUV_SEARCH_TIME_JD_TDB 0
#define DHRUV_SEARCH_TIME_UTC    1

/* Query output selector */
#define DHRUV_QUERY_OUTPUT_CARTESIAN 0
#define DHRUV_QUERY_OUTPUT_SPHERICAL 1
#define DHRUV_QUERY_OUTPUT_BOTH      2

/* Time policy */
#define DHRUV_TIME_POLICY_STRICT_LSK      0
#define DHRUV_TIME_POLICY_HYBRID_DELTA_T  1

/* Delta-T model */
#define DHRUV_DELTA_T_MODEL_LEGACY_ESPENAK_MEEUS_2006       0
#define DHRUV_DELTA_T_MODEL_SMH2016_WITH_PRE720_QUADRATIC   1

/* Future Delta-T transition */
#define DHRUV_FUTURE_DELTA_T_TRANSITION_LEGACY_TT_UTC_BLEND          0
#define DHRUV_FUTURE_DELTA_T_TRANSITION_BRIDGE_FROM_MODERN_ENDPOINT  1

/* SMH future-family selector */
#define DHRUV_SMH_FUTURE_FAMILY_ADDENDUM_2020_PIECEWISE 0
#define DHRUV_SMH_FUTURE_FAMILY_CONSTANT_C_MINUS20      1
#define DHRUV_SMH_FUTURE_FAMILY_CONSTANT_C_MINUS17P52   2
#define DHRUV_SMH_FUTURE_FAMILY_CONSTANT_C_MINUS15P32   3
#define DHRUV_SMH_FUTURE_FAMILY_STEPHENSON_1997         4
#define DHRUV_SMH_FUTURE_FAMILY_STEPHENSON_2016         5

/* TT-UTC diagnostic source */
#define DHRUV_TT_UTC_SOURCE_LSK_DELTA_AT  0
#define DHRUV_TT_UTC_SOURCE_DELTA_T_MODEL 1

/* Time warning kinds */
#define DHRUV_TIME_WARNING_LSK_FUTURE_FROZEN      0
#define DHRUV_TIME_WARNING_LSK_PRE_RANGE_FALLBACK 1
#define DHRUV_TIME_WARNING_EOP_FUTURE_FROZEN      2
#define DHRUV_TIME_WARNING_EOP_PRE_RANGE_FALLBACK 3
#define DHRUV_TIME_WARNING_DELTA_T_MODEL_USED     4

/* Delta-T segment codes */
#define DHRUV_DELTA_T_SEGMENT_PRE_MINUS720_QUADRATIC  0
#define DHRUV_DELTA_T_SEGMENT_SMH2016_RECONSTRUCTION  1
#define DHRUV_DELTA_T_SEGMENT_SMH_ASYMPTOTIC_FUTURE   2
#define DHRUV_DELTA_T_SEGMENT_BEFORE_MINUS500         3
#define DHRUV_DELTA_T_SEGMENT_MINUS500_TO_500         4
#define DHRUV_DELTA_T_SEGMENT_YEAR500_TO1600          5
#define DHRUV_DELTA_T_SEGMENT_YEAR1600_TO1700         6
#define DHRUV_DELTA_T_SEGMENT_YEAR1700_TO1800         7
#define DHRUV_DELTA_T_SEGMENT_YEAR1800_TO1860         8
#define DHRUV_DELTA_T_SEGMENT_YEAR1860_TO1900         9
#define DHRUV_DELTA_T_SEGMENT_YEAR1900_TO1920         10
#define DHRUV_DELTA_T_SEGMENT_YEAR1920_TO1941         11
#define DHRUV_DELTA_T_SEGMENT_YEAR1941_TO1961         12
#define DHRUV_DELTA_T_SEGMENT_YEAR1961_TO1986         13
#define DHRUV_DELTA_T_SEGMENT_YEAR1986_TO2005         14
#define DHRUV_DELTA_T_SEGMENT_YEAR2005_TO2050         15
#define DHRUV_DELTA_T_SEGMENT_YEAR2050_TO2150         16
#define DHRUV_DELTA_T_SEGMENT_AFTER2150               17

#define DHRUV_DASHA_TIME_NONE   -1
#define DHRUV_DASHA_TIME_JD_UTC 0
#define DHRUV_DASHA_TIME_UTC    1

#define DHRUV_MAX_TIME_WARNINGS 8

/* Sun limb */
#define DHRUV_SUN_LIMB_UPPER     0
#define DHRUV_SUN_LIMB_CENTER    1
#define DHRUV_SUN_LIMB_LOWER     2

/* Rise/set result type */
#define DHRUV_RISESET_EVENT       0
#define DHRUV_RISESET_NEVER_RISES 1
#define DHRUV_RISESET_NEVER_SETS  2

/* Rise/set event codes */
#define DHRUV_EVENT_SUNRISE             0
#define DHRUV_EVENT_SUNSET              1
#define DHRUV_EVENT_CIVIL_DAWN          2
#define DHRUV_EVENT_CIVIL_DUSK          3
#define DHRUV_EVENT_NAUTICAL_DAWN       4
#define DHRUV_EVENT_NAUTICAL_DUSK       5
#define DHRUV_EVENT_ASTRONOMICAL_DAWN   6
#define DHRUV_EVENT_ASTRONOMICAL_DUSK   7

/* Ayanamsha mode */
#define DHRUV_AYANAMSHA_MODE_MEAN    0
#define DHRUV_AYANAMSHA_MODE_TRUE    1
#define DHRUV_AYANAMSHA_MODE_UNIFIED 2

/* Ayanamsha time */
#define DHRUV_AYANAMSHA_TIME_JD_TDB 0
#define DHRUV_AYANAMSHA_TIME_UTC    1

/* Bhava system codes */
#define DHRUV_BHAVA_EQUAL           0
#define DHRUV_BHAVA_SURYA_SIDDHANTA 1
#define DHRUV_BHAVA_SRIPATI         2
#define DHRUV_BHAVA_KP              3
#define DHRUV_BHAVA_KOCH            4
#define DHRUV_BHAVA_REGIOMONTANUS   5
#define DHRUV_BHAVA_CAMPANUS        6
#define DHRUV_BHAVA_AXIAL_ROTATION  7
#define DHRUV_BHAVA_TOPOCENTRIC     8
#define DHRUV_BHAVA_ALCABITUS       9

/* Bhava reference mode */
#define DHRUV_BHAVA_REF_START   0
#define DHRUV_BHAVA_REF_MIDDLE  1

/* Bhava starting point */
#define DHRUV_BHAVA_START_LAGNA  -1
#define DHRUV_BHAVA_START_CUSTOM -2

/* Lunar node codes */
#define DHRUV_NODE_RAHU  0
#define DHRUV_NODE_KETU  1

/* Lunar node mode */
#define DHRUV_NODE_MODE_MEAN 0
#define DHRUV_NODE_MODE_TRUE 1

/* Lunar node backend */
#define DHRUV_NODE_BACKEND_ANALYTIC 0
#define DHRUV_NODE_BACKEND_ENGINE   1

/* Lunar node time */
#define DHRUV_NODE_TIME_JD_TDB 0
#define DHRUV_NODE_TIME_UTC    1

/* Conjunction query mode */
#define DHRUV_CONJUNCTION_QUERY_MODE_NEXT  0
#define DHRUV_CONJUNCTION_QUERY_MODE_PREV  1
#define DHRUV_CONJUNCTION_QUERY_MODE_RANGE 2

/* Sentinel */
#define DHRUV_JD_ABSENT (-1.0)

/* Eclipse type constants */
#define DHRUV_CHANDRA_GRAHAN_PENUMBRAL 0
#define DHRUV_CHANDRA_GRAHAN_PARTIAL   1
#define DHRUV_CHANDRA_GRAHAN_TOTAL     2

#define DHRUV_SURYA_GRAHAN_PARTIAL  0
#define DHRUV_SURYA_GRAHAN_ANNULAR  1
#define DHRUV_SURYA_GRAHAN_TOTAL    2
#define DHRUV_SURYA_GRAHAN_HYBRID   3

/* Whether and how the central shadow reaches Earth */
#define DHRUV_SURYA_CENTRALITY_NONE    0
#define DHRUV_SURYA_CENTRALITY_PARTIAL 1
#define DHRUV_SURYA_CENTRALITY_FULL    2

/* Ring-set selectors for isoline/corridor geometry */
#define DHRUV_SURYA_RING_SET_VISIBILITY 0
#define DHRUV_SURYA_RING_SET_DURATION   1
#define DHRUV_SURYA_RING_SET_MAGNITUDE  2
#define DHRUV_SURYA_RING_SET_CORRIDOR   3

/* Ring pole containment */
#define DHRUV_RING_POLE_NONE  0
#define DHRUV_RING_POLE_NORTH 1
#define DHRUV_RING_POLE_SOUTH 2

/* Contact selectors for contact-moment footprints */
#define DHRUV_SURYA_CONTACT_C1       0
#define DHRUV_SURYA_CONTACT_C2       1
#define DHRUV_SURYA_CONTACT_GREATEST 2
#define DHRUV_SURYA_CONTACT_C3       3
#define DHRUV_SURYA_CONTACT_C4       4

/* Maximum isoline levels per family in DhruvGrahanConfig */
#define DHRUV_GRAHAN_MAX_ISOLINE_LEVELS 16

/* Eclipse query */
#define DHRUV_GRAHAN_KIND_CHANDRA 0
#define DHRUV_GRAHAN_KIND_SURYA   1

#define DHRUV_GRAHAN_QUERY_MODE_NEXT  0
#define DHRUV_GRAHAN_QUERY_MODE_PREV  1
#define DHRUV_GRAHAN_QUERY_MODE_RANGE 2

/* Station / max-speed */
#define DHRUV_STATION_RETROGRADE 0
#define DHRUV_STATION_DIRECT     1

#define DHRUV_MAX_SPEED_DIRECT     0
#define DHRUV_MAX_SPEED_RETROGRADE 1

/* Motion query */
#define DHRUV_MOTION_KIND_STATIONARY 0
#define DHRUV_MOTION_KIND_MAX_SPEED  1

#define DHRUV_MOTION_QUERY_MODE_NEXT  0
#define DHRUV_MOTION_QUERY_MODE_PREV  1
#define DHRUV_MOTION_QUERY_MODE_RANGE 2

/* Lunar phase */
#define DHRUV_LUNAR_PHASE_NEW_MOON  0
#define DHRUV_LUNAR_PHASE_FULL_MOON 1

#define DHRUV_LUNAR_PHASE_KIND_AMAVASYA 0
#define DHRUV_LUNAR_PHASE_KIND_PURNIMA  1

#define DHRUV_LUNAR_PHASE_QUERY_MODE_NEXT  0
#define DHRUV_LUNAR_PHASE_QUERY_MODE_PREV  1
#define DHRUV_LUNAR_PHASE_QUERY_MODE_RANGE 2

/* Ayana */
#define DHRUV_AYANA_UTTARAYANA   0
#define DHRUV_AYANA_DAKSHINAYANA 1

/* Sankranti */
#define DHRUV_SANKRANTI_TARGET_ANY      0
#define DHRUV_SANKRANTI_TARGET_SPECIFIC 1

/* Gochar events */
#define DHRUV_GOCHAR_NATAL_TARGET_GRAHA         0
#define DHRUV_GOCHAR_NATAL_TARGET_BINDU         1
#define DHRUV_GOCHAR_NATAL_TARGET_SPHUTA        2
#define DHRUV_GOCHAR_NATAL_TARGET_SPECIAL_LAGNA 3
#define DHRUV_GOCHAR_NATAL_TARGET_ARUDHA_PADA   4
#define DHRUV_GOCHAR_NATAL_TARGET_CUSTOM        5

#define DHRUV_TAJAKA_RETURN_BASIS_TROPICAL_SOLAR 0
#define DHRUV_TAJAKA_RETURN_BASIS_SIDEREAL_SOLAR 1

#define DHRUV_TRANSIT_ASPECT_KIND_CONJUNCTION 0
#define DHRUV_TRANSIT_ASPECT_KIND_OPPOSITION  1
#define DHRUV_TRANSIT_ASPECT_KIND_SPECIAL     2

#define DHRUV_TRANSIT_ASPECT_OWNER_GOCHAR_BODY  0
#define DHRUV_TRANSIT_ASPECT_OWNER_NATAL_TARGET 1

#define DHRUV_SANKRANTI_QUERY_MODE_NEXT  0
#define DHRUV_SANKRANTI_QUERY_MODE_PREV  1
#define DHRUV_SANKRANTI_QUERY_MODE_RANGE 2

/* Panchang time */
#define DHRUV_PANCHANG_TIME_JD_TDB 0
#define DHRUV_PANCHANG_TIME_UTC    1

/* Panchang include masks */
#define DHRUV_PANCHANG_INCLUDE_TITHI      (1U << 0)
#define DHRUV_PANCHANG_INCLUDE_KARANA     (1U << 1)
#define DHRUV_PANCHANG_INCLUDE_YOGA       (1U << 2)
#define DHRUV_PANCHANG_INCLUDE_VAAR       (1U << 3)
#define DHRUV_PANCHANG_INCLUDE_HORA       (1U << 4)
#define DHRUV_PANCHANG_INCLUDE_GHATIKA    (1U << 5)
#define DHRUV_PANCHANG_INCLUDE_NAKSHATRA  (1U << 6)
#define DHRUV_PANCHANG_INCLUDE_MASA       (1U << 7)
#define DHRUV_PANCHANG_INCLUDE_AYANA      (1U << 8)
#define DHRUV_PANCHANG_INCLUDE_VARSHA     (1U << 9)
#define DHRUV_PANCHANG_INCLUDE_ALL_CORE     \
    (DHRUV_PANCHANG_INCLUDE_TITHI     |     \
     DHRUV_PANCHANG_INCLUDE_KARANA    |     \
     DHRUV_PANCHANG_INCLUDE_YOGA      |     \
     DHRUV_PANCHANG_INCLUDE_VAAR      |     \
     DHRUV_PANCHANG_INCLUDE_HORA      |     \
     DHRUV_PANCHANG_INCLUDE_GHATIKA   |     \
     DHRUV_PANCHANG_INCLUDE_NAKSHATRA)
#define DHRUV_PANCHANG_INCLUDE_ALL_CALENDAR \
    (DHRUV_PANCHANG_INCLUDE_MASA  |         \
     DHRUV_PANCHANG_INCLUDE_AYANA |         \
     DHRUV_PANCHANG_INCLUDE_VARSHA)
#define DHRUV_PANCHANG_INCLUDE_ALL          \
    (DHRUV_PANCHANG_INCLUDE_ALL_CORE |      \
     DHRUV_PANCHANG_INCLUDE_ALL_CALENDAR)
#define DHRUV_PANCHANG_INCLUDE_LOCATION_INDEPENDENT \
    (DHRUV_PANCHANG_INCLUDE_TITHI     |             \
     DHRUV_PANCHANG_INCLUDE_KARANA    |             \
     DHRUV_PANCHANG_INCLUDE_YOGA      |             \
     DHRUV_PANCHANG_INCLUDE_NAKSHATRA |             \
     DHRUV_PANCHANG_INCLUDE_MASA      |             \
     DHRUV_PANCHANG_INCLUDE_AYANA     |             \
     DHRUV_PANCHANG_INCLUDE_VARSHA)
#define DHRUV_PANCHANG_INCLUDE_LOCATION_DEPENDENT   \
    (DHRUV_PANCHANG_INCLUDE_VAAR |                  \
     DHRUV_PANCHANG_INCLUDE_HORA |                  \
     DHRUV_PANCHANG_INCLUDE_GHATIKA)

/* Count constants */
#define DHRUV_GRAHA_COUNT              9
#define DHRUV_SAPTA_GRAHA_COUNT        7
#define DHRUV_SPHUTA_COUNT            16
#define DHRUV_SPECIAL_LAGNA_COUNT      8
#define DHRUV_ARUDHA_PADA_COUNT       12
#define DHRUV_UPAGRAHA_COUNT          11
#define DHRUV_ASHTAKAVARGA_GRAHA_COUNT 7
#define DHRUV_MAX_AMSHA_REQUESTS      40
#define DHRUV_MAX_DASHA_SYSTEMS       23
#define DHRUV_MAX_CHARAKARAKA_ENTRIES 8
#define DHRUV_MAX_AMSHA_SERIES_CELLS   100000
#define DHRUV_MAX_PANCHANG_EVENTS       50000
#define DHRUV_MAX_AMSHA_LAGNA_SEGMENTS  50000
#define DHRUV_MAX_CHARAKARAKA_EVENTS    50000

/* Charakaraka schemes */
#define DHRUV_CHARAKARAKA_SCHEME_EIGHT             0
#define DHRUV_CHARAKARAKA_SCHEME_SEVEN_NO_PITRI    1
#define DHRUV_CHARAKARAKA_SCHEME_SEVEN_PK_MERGED_MK 2
#define DHRUV_CHARAKARAKA_SCHEME_MIXED_PARASHARA   3

/* Charakaraka role codes */
#define DHRUV_CHARAKARAKA_ROLE_ATMA         0
#define DHRUV_CHARAKARAKA_ROLE_AMATYA       1
#define DHRUV_CHARAKARAKA_ROLE_BHRATRI      2
#define DHRUV_CHARAKARAKA_ROLE_MATRI        3
#define DHRUV_CHARAKARAKA_ROLE_PITRI        4
#define DHRUV_CHARAKARAKA_ROLE_PUTRA        5
#define DHRUV_CHARAKARAKA_ROLE_GNATI        6
#define DHRUV_CHARAKARAKA_ROLE_DARA         7
#define DHRUV_CHARAKARAKA_ROLE_MATRI_PUTRA  8

/* Charakaraka ranking-change event triggers */
#define DHRUV_CHARAKARAKA_TRIGGER_DEGREE_CROSSING    0
#define DHRUV_CHARAKARAKA_TRIGGER_RASHI_INGRESS      1
#define DHRUV_CHARAKARAKA_TRIGGER_SCHEME_MODE_CHANGE 2

/* Tara output selectors */
#define DHRUV_TARA_OUTPUT_EQUATORIAL 0
#define DHRUV_TARA_OUTPUT_ECLIPTIC   1
#define DHRUV_TARA_OUTPUT_SIDEREAL   2

/* ===================================================================
 * Opaque handles
 * =================================================================== */

typedef struct DhruvEngineHandle    DhruvEngineHandle;
typedef struct DhruvLskHandle       DhruvLskHandle;
typedef struct DhruvEopHandle       DhruvEopHandle;
typedef struct DhruvConfigHandle    DhruvConfigHandle;
typedef struct DhruvTaraCatalogHandle DhruvTaraCatalogHandle;

/* DhruvDashaHierarchyHandle is void* */
typedef void *DhruvDashaHierarchyHandle;
/* DhruvDashaPeriodListHandle is void* */
typedef void *DhruvDashaPeriodListHandle;
/* DhruvGocharEventsHandle is void* */
typedef void *DhruvGocharEventsHandle;
/* DhruvSuryaGrahanGeometryHandle is void* */
typedef void *DhruvSuryaGrahanGeometryHandle;

/* ===================================================================
 * Structs
 * =================================================================== */

typedef struct {
    uint32_t spk_path_count;
    uint8_t  spk_paths_utf8[8][512];
    uint8_t  lsk_path_utf8[512];
    uint64_t cache_capacity;
    uint8_t  strict_validation;
} DhruvEngineConfig;

typedef struct {
    uint32_t spk_path_count;
    uint8_t  spk_paths_utf8[8][512];
} DhruvSpkSetConfig;

typedef struct {
    uint64_t generation;
    uint32_t active_count;
    uint32_t loaded_count;
    uint32_t reused_count;
} DhruvSpkReplaceReport;

typedef struct {
    uint8_t  path_utf8[512];
    uint32_t segment_count;
    uint64_t generation;
} DhruvLoadedSpkInfo;

typedef struct {
    uint32_t count;
    uint64_t generation;
    DhruvLoadedSpkInfo entries[8];
} DhruvLoadedSpkList;

typedef struct {
    int32_t target;
    int32_t observer;
    int32_t frame;
    double  epoch_tdb_jd;
} DhruvQuery;

typedef struct {
    double position_km[3];
    double velocity_km_s[3];
} DhruvStateVector;

typedef struct {
    double lon_deg;
    double lat_deg;
    double distance_km;
} DhruvSphericalCoords;

typedef struct {
    double lon_deg;
    double lat_deg;
    double distance_km;
    double lon_speed;
    double lat_speed;
    double distance_speed;
} DhruvSphericalState;

typedef struct {
    DhruvStateVector    state_vector;
    DhruvSphericalState spherical_state;
} DhruvQueryResult;

typedef struct {
    uint8_t warn_on_fallback;
    int32_t delta_t_model;
    uint8_t freeze_future_dut1;
    double  pre_range_dut1;
    int32_t future_delta_t_transition;
    double  future_transition_years;
    int32_t smh_future_family;
} DhruvTimeConversionOptions;

typedef struct {
    int32_t                   mode;
    DhruvTimeConversionOptions options;
} DhruvTimePolicy;

typedef struct {
    int32_t  year;
    uint32_t month;
    uint32_t day;
    uint32_t hour;
    uint32_t minute;
    double   second;
} DhruvUtcTime;

typedef struct {
    int32_t kind;
    double  utc_seconds;
    double  first_entry_utc_seconds;
    double  last_entry_utc_seconds;
    double  used_delta_at_seconds;
    double  mjd;
    double  first_entry_mjd;
    double  last_entry_mjd;
    double  used_dut1_seconds;
    int32_t delta_t_model;
    int32_t delta_t_segment;
} DhruvTimeWarning;

typedef struct {
    int32_t          source;
    double           tt_minus_utc_s;
    uint32_t         warning_count;
    DhruvTimeWarning warnings[DHRUV_MAX_TIME_WARNINGS];
} DhruvTimeDiagnostics;

typedef struct {
    DhruvUtcTime    utc;
    DhruvTimePolicy policy;
} DhruvUtcToTdbRequest;

typedef struct {
    double               jd_tdb;
    DhruvTimeDiagnostics diagnostics;
} DhruvUtcToTdbResult;

typedef struct {
    int32_t      target;
    int32_t      observer;
    int32_t      frame;
    int32_t      time_kind;
    double       epoch_tdb_jd;
    DhruvUtcTime utc;
    int32_t      output_mode;
} DhruvQueryRequest;

typedef struct {
    double latitude_deg;
    double longitude_deg;
    double altitude_m;
} DhruvGeoLocation;

typedef struct {
    uint8_t use_refraction;
    int32_t sun_limb;
    uint8_t altitude_correction;
} DhruvRiseSetConfig;

typedef struct {
    int32_t result_type;
    int32_t event_code;
    double  jd_tdb;
} DhruvRiseSetResult;

typedef struct {
    int32_t system_code;
    int32_t mode;
    int32_t time_kind;
    double  jd_tdb;
    DhruvUtcTime utc;
    uint8_t use_nutation;
    double  delta_psi_arcsec;
} DhruvAyanamshaComputeRequest;

typedef struct {
    uint16_t degrees;
    uint8_t  minutes;
    double   seconds;
} DhruvDms;

typedef struct {
    uint8_t  rashi_index;
    DhruvDms dms;
    double   degrees_in_rashi;
} DhruvRashiInfo;

typedef struct {
    uint8_t nakshatra_index;
    uint8_t pada;
    double  degrees_in_nakshatra;
    double  degrees_in_pada;
} DhruvNakshatraInfo;

typedef struct {
    uint8_t nakshatra_index;
    uint8_t pada;
    double  degrees_in_nakshatra;
} DhruvNakshatra28Info;

/* --- Bhava --- */

typedef struct {
    int32_t system;
    int32_t starting_point;
    double  custom_start_deg;
    int32_t reference_mode;
    int32_t output_mode;
    int32_t ayanamsha_system;
    uint8_t use_nutation;
    int32_t reference_plane;
    uint8_t use_rashi_bhava_for_bala_avastha;
    uint8_t include_node_aspects_for_drik_bala;
    uint8_t include_special_bhavabala_rules;
    uint8_t divide_guru_buddh_drishti_by_4_for_drik_bala;
    int32_t chandra_benefic_rule;
    int32_t sayanadi_ghatika_rounding;
    uint8_t include_rashi_bhava_results;
} DhruvBhavaConfig;

typedef struct {
    uint8_t number;
    double  cusp_deg;
    double  start_deg;
    double  end_deg;
} DhruvBhava;

typedef struct {
    DhruvBhava bhavas[12];
    double lagna_deg;
    double mc_deg;
    uint8_t rashi_bhava_valid;
    DhruvBhava rashi_bhava_bhavas[12];
    double rashi_bhava_lagna_deg;
    double rashi_bhava_mc_deg;
} DhruvBhavaResult;

/* --- Lunar node --- */

typedef struct {
    int32_t      node_code;
    int32_t      mode_code;
    int32_t      backend;
    int32_t      time_kind;
    double       jd_tdb;
    DhruvUtcTime utc;
} DhruvLunarNodeRequest;

/* --- Sankranti config (shared by search requests below) --- */

typedef struct {
    int32_t  ayanamsha_system;
    uint8_t  use_nutation;
    int32_t  reference_plane;
    double   step_size_days;
    uint32_t max_iterations;
    double   convergence_days;
    /* 0 = mean node, any other value = true node (v84). */
    int32_t  node_mode;
} DhruvSankrantiConfig;

/* --- Conjunction --- */

typedef struct {
    double   target_separation_deg;
    double   step_size_days;
    uint32_t max_iterations;
    double   convergence_days;
    /* 0 = mean node, any other value = true node (v84). */
    int32_t  node_mode;
} DhruvConjunctionConfig;

typedef struct {
    int32_t body1_code;   /* NAIF code, or 10007 (Rahu) / 10008 (Ketu) */
    int32_t body2_code;   /* NAIF code, or 10007 (Rahu) / 10008 (Ketu) */
    int32_t query_mode;
    int32_t time_kind;
    double  at_jd_tdb;
    double  start_jd_tdb;
    double  end_jd_tdb;
    DhruvUtcTime at_utc;
    DhruvUtcTime start_utc;
    DhruvUtcTime end_utc;
    DhruvConjunctionConfig config;
    /* v84 multi-angle sweep: 0 = single angle from config. */
    uint32_t target_separation_count;
    double   target_separations_deg[DHRUV_MAX_CONJUNCTION_TARGETS];
    /* v84 sidereal echo: read sidereal_config when non-zero. */
    uint8_t  has_sidereal_config;
    DhruvSankrantiConfig sidereal_config;
} DhruvConjunctionSearchRequest;

typedef struct {
    double  jd_tdb;
    DhruvUtcTime utc;
    double  actual_separation_deg;
    double  body1_longitude_deg;
    double  body2_longitude_deg;
    double  body1_latitude_deg;
    double  body2_latitude_deg;
    int32_t body1_code;
    int32_t body2_code;
    /* v84: matched target angle plus optional sidereal echoes. */
    double  target_separation_deg;
    uint8_t has_sidereal;
    double  body1_sidereal_longitude_deg; /* 0.0 when has_sidereal == 0 */
    double  body2_sidereal_longitude_deg; /* 0.0 when has_sidereal == 0 */
    int32_t body1_rashi_index;            /* -1 when has_sidereal == 0 */
    int32_t body2_rashi_index;            /* -1 when has_sidereal == 0 */
} DhruvConjunctionEvent;

/* --- Grahan (eclipse) --- */

typedef struct {
    uint8_t include_penumbral;
    uint8_t include_peak_details;
    uint8_t include_path;
    uint8_t include_local_grid;
    uint8_t include_isolines;
    uint8_t include_central_corridor;
    uint8_t include_contact_footprints;
    uint8_t include_umbra_footprints;
    uint32_t path_step_minutes;
    uint32_t boundary_step_deg;
    /* Local-grid spacing in degrees; values outside [0.5, 10] are clamped. */
    double local_grid_step_deg;
    /* Visible-duration isoline levels as fractions of the C1-C4 span. */
    double duration_isoline_fractions[DHRUV_GRAHAN_MAX_ISOLINE_LEVELS];
    uint32_t duration_isoline_fraction_count;
    /* Local maximum-magnitude isoline levels. */
    double magnitude_isoline_levels[DHRUV_GRAHAN_MAX_ISOLINE_LEVELS];
    uint32_t magnitude_isoline_level_count;
    /* Instantaneous iso-magnitude contour levels for footprints. */
    double instantaneous_magnitude_levels[DHRUV_GRAHAN_MAX_ISOLINE_LEVELS];
    uint32_t instantaneous_magnitude_level_count;
} DhruvGrahanConfig;

typedef struct {
    int32_t grahan_kind;
    int32_t query_mode;
    int32_t time_kind;
    double  at_jd_tdb;
    double  start_jd_tdb;
    double  end_jd_tdb;
    DhruvUtcTime at_utc;
    DhruvUtcTime start_utc;
    DhruvUtcTime end_utc;
    DhruvGrahanConfig config;
    uint8_t location_valid;
    DhruvGeoLocation location;
} DhruvGrahanSearchRequest;

typedef struct {
    int32_t grahan_type;
    double  magnitude;
    double  penumbral_magnitude;
    double  greatest_grahan_jd;
    DhruvUtcTime greatest_grahan_utc;
    double  p1_jd;
    DhruvUtcTime p1_utc;
    double  u1_jd;
    DhruvUtcTime u1_utc;
    double  u2_jd;
    DhruvUtcTime u2_utc;
    double  u3_jd;
    DhruvUtcTime u3_utc;
    double  u4_jd;
    DhruvUtcTime u4_utc;
    double  p4_jd;
    DhruvUtcTime p4_utc;
    double  moon_ecliptic_lat_deg;
    double  angular_separation_deg;
    /* Moon's apparent geocentric RA/declination at greatest grahan, degrees
       (equinox of date, nutation applied). */
    double  moon_right_ascension_deg;
    double  moon_declination_deg;
} DhruvChandraGrahanResult;

typedef struct {
    double latitude_deg;
    double longitude_deg;
} DhruvEclipseGeoPoint;

typedef struct {
    double jd_tdb;
    DhruvUtcTime utc;
    DhruvEclipseGeoPoint center;
    uint8_t northern_limit_valid;
    DhruvEclipseGeoPoint northern_limit;
    uint8_t southern_limit_valid;
    DhruvEclipseGeoPoint southern_limit;
    double width_km;
    double central_duration_seconds;
    double sun_altitude_deg;
    double sun_azimuth_deg;
    int32_t grahan_type;
} DhruvSuryaGrahanPathPoint;

typedef struct {
    double jd_tdb;
    DhruvUtcTime utc;
    uint32_t boundary_count;
    /* Pole containment of the shadow region (DHRUV_RING_POLE_*). */
    int32_t contains_pole;
    /* Number of instantaneous iso-magnitude rings at this timestamp. */
    uint32_t magnitude_ring_count;
} DhruvSuryaGrahanFootprint;

/* One instantaneous iso-magnitude contour ring. */
typedef struct {
    double level;
    int32_t contains_pole;   /* DHRUV_RING_POLE_* */
    uint32_t point_count;    /* final point repeats the first */
} DhruvSuryaMagnitudeRing;

/* One contact-moment penumbral footprint. boundary_count may be zero at
   exact C1/C4 tangency; fall back to the nearest sampled footprint. */
typedef struct {
    int32_t contact;   /* DHRUV_SURYA_CONTACT_* */
    double jd_tdb;
    DhruvUtcTime utc;
    uint32_t boundary_count;
    int32_t contains_pole;   /* DHRUV_RING_POLE_* */
    uint32_t magnitude_ring_count;
} DhruvSuryaContactFootprint;

/* One instantaneous umbral/antumbral shadow outline. */
typedef struct {
    double jd_tdb;
    DhruvUtcTime utc;
    int32_t grahan_type;     /* DHRUV_SURYA_GRAHAN_TOTAL or _ANNULAR */
    uint32_t boundary_count;
    int32_t contains_pole;   /* DHRUV_RING_POLE_* */
} DhruvSuryaUmbraFootprint;

/* One sample of the per-event local-circumstance grid. */
typedef struct {
    double latitude_deg;
    double longitude_deg;
    double magnitude;
    double obscuration;
    double maximum_jd;
    DhruvUtcTime maximum_utc;
    double first_contact_jd;
    DhruvUtcTime first_contact_utc;
    double last_contact_jd;
    DhruvUtcTime last_contact_utc;
    double visible_duration_seconds;
} DhruvSuryaLocalGridSample;

/* Metadata for one level of a ring set (isoline level or corridor segment). */
typedef struct {
    /* Duration fraction or magnitude level; 0 for visibility/corridor sets. */
    double level_value;
    /* Corridor segment type (DHRUV_SURYA_GRAHAN_*), or -1 for isoline sets. */
    int32_t grahan_type;
    uint32_t ring_count;
} DhruvSuryaRingSetLevel;

/* Metadata for one closed ring of a ring set. */
typedef struct {
    /* Pole containment (DHRUV_RING_POLE_*). */
    int32_t contains_pole;
    /* Number of boundary points; the final point repeats the first. */
    uint32_t point_count;
} DhruvSuryaIsolineRing;

typedef struct {
    int32_t grahan_type;
    double  magnitude;
    double  obscuration;
    double  apparent_diameter_ratio;
    double  gamma;
    double  greatest_grahan_jd;
    DhruvUtcTime greatest_grahan_utc;
    double  c1_jd;
    DhruvUtcTime c1_utc;
    double  c2_jd;
    DhruvUtcTime c2_utc;
    double  c3_jd;
    DhruvUtcTime c3_utc;
    double  c4_jd;
    DhruvUtcTime c4_utc;
    double  moon_ecliptic_lat_deg;
    double  angular_separation_deg;
    /* Sun's apparent geocentric RA/declination at greatest grahan, degrees
       (equinox of date, nutation applied). */
    double  sun_right_ascension_deg;
    double  sun_declination_deg;
    uint8_t greatest_location_valid;
    double  greatest_latitude_deg;
    double  greatest_longitude_deg;
    double  bessel_x;
    double  bessel_y;
    double  bessel_d_deg;
    double  bessel_mu_deg;
    double  bessel_l1;
    double  bessel_l2;
    double  bessel_tan_f1;
    double  bessel_tan_f2;
    uint32_t path_count;
    uint32_t footprint_count;
    /* Whether and how the central shadow reaches Earth
       (DHRUV_SURYA_CENTRALITY_*). */
    int32_t centrality;
    uint32_t local_grid_count;
    uint8_t isolines_valid;
    uint8_t central_corridor_valid;
    uint32_t contact_footprint_count;
    uint32_t umbra_footprint_count;
    DhruvSuryaGrahanGeometryHandle geometry_handle;
    uint8_t local_valid;
    uint8_t local_visible;
    int32_t local_grahan_type;
    double  local_maximum_jd;
    DhruvUtcTime local_maximum_utc;
    double  local_c1_jd;
    DhruvUtcTime local_c1_utc;
    double  local_c2_jd;
    DhruvUtcTime local_c2_utc;
    double  local_c3_jd;
    DhruvUtcTime local_c3_utc;
    double  local_c4_jd;
    DhruvUtcTime local_c4_utc;
    double  local_magnitude;
    double  local_obscuration;
    double  local_sun_altitude_deg;
    double  local_sun_azimuth_deg;
    double  local_central_duration_seconds;
} DhruvSuryaGrahanResult;

/* --- Stationary / max-speed --- */

typedef struct {
    double   step_size_days;
    uint32_t max_iterations;
    double   convergence_days;
    double   numerical_step_days;
    /* 0 = mean node, any other value = true node (v84). Stationary
     * search of Rahu/Ketu requires the true node. */
    int32_t  node_mode;
} DhruvStationaryConfig;

typedef struct {
    int32_t body_code;    /* NAIF code, or 10007 (Rahu) / 10008 (Ketu) */
    int32_t motion_kind;
    int32_t query_mode;
    int32_t time_kind;
    double  at_jd_tdb;
    double  start_jd_tdb;
    double  end_jd_tdb;
    DhruvUtcTime at_utc;
    DhruvUtcTime start_utc;
    DhruvUtcTime end_utc;
    DhruvStationaryConfig config;
    /* v84 sidereal echo: read sidereal_config when non-zero. */
    uint8_t has_sidereal_config;
    DhruvSankrantiConfig sidereal_config;
} DhruvMotionSearchRequest;

typedef struct {
    double  jd_tdb;
    DhruvUtcTime utc;
    int32_t body_code;
    double  longitude_deg;
    double  latitude_deg;
    int32_t station_type;
    /* v84 sidereal echoes. */
    uint8_t has_sidereal;
    double  sidereal_longitude_deg; /* 0.0 when has_sidereal == 0 */
    int32_t rashi_index;            /* -1 when has_sidereal == 0 */
} DhruvStationaryEvent;

typedef struct {
    double  jd_tdb;
    DhruvUtcTime utc;
    int32_t body_code;
    double  longitude_deg;
    double  latitude_deg;
    double  speed_deg_per_day;
    int32_t speed_type;
    /* v84 sidereal echoes. */
    uint8_t has_sidereal;
    double  sidereal_longitude_deg; /* 0.0 when has_sidereal == 0 */
    int32_t rashi_index;            /* -1 when has_sidereal == 0 */
} DhruvMaxSpeedEvent;

/* --- Sankranti / Lunar phase --- */

/* DhruvSankrantiConfig is defined above the conjunction section. */

typedef struct {
    DhruvUtcTime utc;
    int32_t rashi_index;
    /* Legacy aliases for the tracked body's longitudes (the Sun for
     * classical sankranti requests); identical to the v84 fields below. */
    double  sun_sidereal_longitude_deg;
    double  sun_tropical_longitude_deg;
    /* v84 any-body ingress fields. */
    int32_t body_code;              /* NAIF code, or 10007/10008 */
    double  sidereal_longitude_deg;
    double  tropical_longitude_deg;
    uint8_t is_retrograde;          /* 1 = boundary crossed in retrograde */
} DhruvSankrantiEvent;

typedef struct {
    int32_t target_kind;
    int32_t query_mode;
    int32_t rashi_index;
    int32_t time_kind;
    double  at_jd_tdb;
    double  start_jd_tdb;
    double  end_jd_tdb;
    DhruvUtcTime at_utc;
    DhruvUtcTime start_utc;
    DhruvUtcTime end_utc;
    DhruvSankrantiConfig config;
    /* v84: 0 = Sun (classical sankranti), otherwise a NAIF code or
     * 10007 (Rahu) / 10008 (Ketu). */
    int32_t body_code;
} DhruvSankrantiSearchRequest;

typedef struct {
    int32_t phase_kind;
    int32_t query_mode;
    int32_t time_kind;
    double  at_jd_tdb;
    double  start_jd_tdb;
    double  end_jd_tdb;
    DhruvUtcTime at_utc;
    DhruvUtcTime start_utc;
    DhruvUtcTime end_utc;
} DhruvLunarPhaseSearchRequest;

typedef struct {
    DhruvUtcTime utc;
    int32_t phase;
    double  moon_longitude_deg;
    double  sun_longitude_deg;
} DhruvLunarPhaseEvent;

/* --- Panchang types --- */

typedef struct {
    int32_t      tithi_index;
    int32_t      paksha;
    int32_t      tithi_in_paksha;
    double       degrees_in_tithi;
} DhruvTithiPosition;

typedef struct {
    int32_t karana_index;
    double  degrees_in_karana;
} DhruvKaranaPosition;

typedef struct {
    int32_t yoga_index;
    double  degrees_in_yoga;
} DhruvYogaPosition;

typedef struct {
    int32_t samvatsara_index;
    int32_t cycle_position;
} DhruvSamvatsaraResult;

typedef struct {
    int32_t      tithi_index;
    int32_t      paksha;
    int32_t      tithi_in_paksha;
    DhruvUtcTime start;
    DhruvUtcTime end;
} DhruvTithiInfo;

typedef struct {
    int32_t      karana_index;
    int32_t      karana_name_index;
    DhruvUtcTime start;
    DhruvUtcTime end;
} DhruvKaranaInfo;

typedef struct {
    int32_t      yoga_index;
    DhruvUtcTime start;
    DhruvUtcTime end;
} DhruvYogaInfo;

typedef struct {
    int32_t      vaar_index;
    DhruvUtcTime start;
    DhruvUtcTime end;
} DhruvVaarInfo;

typedef struct {
    int32_t      hora_index;
    int32_t      hora_position;
    DhruvUtcTime start;
    DhruvUtcTime end;
} DhruvHoraInfo;

typedef struct {
    int32_t      value;
    DhruvUtcTime start;
    DhruvUtcTime end;
} DhruvGhatikaInfo;

typedef struct {
    int32_t      nakshatra_index;
    int32_t      pada;
    DhruvUtcTime start;
    DhruvUtcTime end;
} DhruvPanchangNakshatraInfo;

typedef struct {
    int32_t      masa_index;
    uint8_t      adhika;
    DhruvUtcTime start;
    DhruvUtcTime end;
} DhruvMasaInfo;

typedef struct {
    int32_t      ayana;
    DhruvUtcTime start;
    DhruvUtcTime end;
} DhruvAyanaInfo;

typedef struct {
    int32_t      samvatsara_index;
    int32_t      order;
    DhruvUtcTime start;
    DhruvUtcTime end;
} DhruvVarshaInfo;

typedef struct {
    int32_t          time_kind;
    double           jd_tdb;
    DhruvUtcTime     utc;
    uint32_t         include_mask;
    /* 1 when location is set; 0 to compute without an observer location.
     * Required only for location-dependent elements (vaar, hora, ghatika). */
    uint8_t          has_location;
    DhruvGeoLocation location;
    DhruvRiseSetConfig  riseset_config;
    DhruvSankrantiConfig sankranti_config;
    /* Caller-cached calendar elements from a previous result. A known value
     * is reused only when the requested moment falls inside its [start, end)
     * window and its element is selected in include_mask; stale or invalid
     * values are ignored and the element is recomputed. Each known_* field
     * is read only when its known_*_valid flag is non-zero. */
    uint8_t         known_masa_valid;
    DhruvMasaInfo   known_masa;
    uint8_t         known_ayana_valid;
    DhruvAyanaInfo  known_ayana;
    uint8_t         known_varsha_valid;
    DhruvVarshaInfo known_varsha;
} DhruvPanchangComputeRequest;

typedef struct {
    uint8_t                  tithi_valid;
    DhruvTithiInfo           tithi;
    uint8_t                  karana_valid;
    DhruvKaranaInfo          karana;
    uint8_t                  yoga_valid;
    DhruvYogaInfo            yoga;
    uint8_t                  vaar_valid;
    DhruvVaarInfo            vaar;
    uint8_t                  hora_valid;
    DhruvHoraInfo            hora;
    uint8_t                  ghatika_valid;
    DhruvGhatikaInfo         ghatika;
    uint8_t                  nakshatra_valid;
    DhruvPanchangNakshatraInfo nakshatra;
    uint8_t                  masa_valid;
    DhruvMasaInfo            masa;
    uint8_t                  ayana_valid;
    DhruvAyanaInfo           ayana;
    uint8_t                  varsha_valid;
    DhruvVarshaInfo          varsha;
} DhruvPanchangOperationResult;

typedef struct {
    int32_t     kind;
    uint8_t     index;
    const char *name_utf8;
    double      longitude_deg;
} DhruvGocharNatalTarget;

typedef struct {
    int32_t  tajaka_return_basis;
    uint32_t yearly_count;
    uint32_t monthly_count;
    double   transit_window_days;
    uint8_t  include_return_charts;
    double   solar_step_size_days;
    double   lunar_step_size_days;
    double   solar_convergence_days;
    double   lunar_convergence_days;
    uint32_t max_iterations;
} DhruvGocharEventsConfig;

typedef struct {
    double        natal_tropical_solar_longitude_deg;
    double        natal_sidereal_solar_longitude_deg;
    double        natal_elongation_deg;
    DhruvMasaInfo natal_masa;
} DhruvGocharReference;

typedef struct {
    DhruvUtcTime         birth_utc;
    DhruvUtcTime         at_utc;
    DhruvGocharReference reference;
} DhruvGocharEventsSummary;

typedef struct {
    DhruvUtcTime utc;
    double       jd_tdb;
    int32_t      basis;
    double       target_solar_longitude_deg;
    double       event_solar_longitude_deg;
    uint8_t      has_chart;
} DhruvTajakaReturnEventRow;

typedef struct {
    DhruvUtcTime  utc;
    double        jd_tdb;
    double        target_elongation_deg;
    double        event_elongation_deg;
    DhruvMasaInfo masa;
    uint8_t       has_chart;
} DhruvTithiPraveshaEventRow;

typedef struct {
    /* Physical body Body::code(), or DHRUV_GOCHAR_TRANSIT_RAHU/KETU. */
    uint32_t transit_body_code;
    int32_t  target_kind;
    uint8_t  target_index;
    char     target_name[DHRUV_GOCHAR_NAME_CAPACITY];
    int32_t  aspect_kind;
    int32_t  aspect_owner;
    double   aspect_angle_deg;
    DhruvUtcTime utc;
    double   jd_tdb;
    double   transit_longitude_deg;
    double   target_longitude_deg;
    double   actual_separation_deg;
} DhruvTransitToNatalAspectEventRow;

/* --- UTC event variants --- */

typedef struct {
    DhruvUtcTime utc;
    double  actual_separation_deg;
    double  body1_longitude_deg;
    double  body2_longitude_deg;
    double  body1_latitude_deg;
    double  body2_latitude_deg;
    int32_t body1_code;
    int32_t body2_code;
    /* v84: matched target angle plus optional sidereal echoes. */
    double  target_separation_deg;
    uint8_t has_sidereal;
    double  body1_sidereal_longitude_deg; /* 0.0 when has_sidereal == 0 */
    double  body2_sidereal_longitude_deg; /* 0.0 when has_sidereal == 0 */
    int32_t body1_rashi_index;            /* -1 when has_sidereal == 0 */
    int32_t body2_rashi_index;            /* -1 when has_sidereal == 0 */
} DhruvConjunctionEventUtc;

typedef struct {
    DhruvUtcTime utc;
    int32_t body_code;
    double  longitude_deg;
    double  latitude_deg;
    int32_t station_type;
    /* v84 sidereal echoes. */
    uint8_t has_sidereal;
    double  sidereal_longitude_deg; /* 0.0 when has_sidereal == 0 */
    int32_t rashi_index;            /* -1 when has_sidereal == 0 */
} DhruvStationaryEventUtc;

typedef struct {
    DhruvUtcTime utc;
    int32_t body_code;
    double  longitude_deg;
    double  latitude_deg;
    double  speed_deg_per_day;
    int32_t speed_type;
    /* v84 sidereal echoes. */
    uint8_t has_sidereal;
    double  sidereal_longitude_deg; /* 0.0 when has_sidereal == 0 */
    int32_t rashi_index;            /* -1 when has_sidereal == 0 */
} DhruvMaxSpeedEventUtc;

typedef struct {
    int32_t      result_type;
    int32_t      event_code;
    DhruvUtcTime utc;
} DhruvRiseSetResultUtc;

typedef struct {
    int32_t      grahan_type;
    double       magnitude;
    double       penumbral_magnitude;
    DhruvUtcTime greatest_grahan;
    DhruvUtcTime p1;
    DhruvUtcTime u1;
    DhruvUtcTime u2;
    DhruvUtcTime u3;
    DhruvUtcTime u4;
    DhruvUtcTime p4;
    double       moon_ecliptic_lat_deg;
    double       angular_separation_deg;
    uint8_t      u1_valid;
    uint8_t      u2_valid;
    uint8_t      u3_valid;
    uint8_t      u4_valid;
} DhruvChandraGrahanResultUtc;

typedef struct {
    int32_t      grahan_type;
    double       magnitude;
    DhruvUtcTime greatest_grahan;
    DhruvUtcTime c1;
    DhruvUtcTime c2;
    DhruvUtcTime c3;
    DhruvUtcTime c4;
    double       moon_ecliptic_lat_deg;
    double       angular_separation_deg;
    uint8_t      c1_valid;
    uint8_t      c2_valid;
    uint8_t      c3_valid;
    uint8_t      c4_valid;
} DhruvSuryaGrahanResultUtc;

/* --- Sphuta --- */

typedef struct {
    double sun;
    double moon;
    double mars;
    double jupiter;
    double venus;
    double rahu;
    double lagna;
    double eighth_lord;
    double gulika;
} DhruvSphutalInputs;

typedef struct {
    double longitudes[16];
} DhruvSphutalResult;

/* --- Special Lagnas --- */

typedef struct {
    double bhava_lagna;
    double hora_lagna;
    double ghati_lagna;
    double vighati_lagna;
    double varnada_lagna;
    double sree_lagna;
    double pranapada_lagna;
    double indu_lagna;
} DhruvSpecialLagnas;

/* --- Arudha --- */

typedef struct {
    uint8_t bhava_number;
    double  longitude_deg;
    uint8_t rashi_index;
} DhruvArudhaResult;

/* --- Upagrahas --- */

enum {
    DHRUV_UPAGRAHA_POINT_START = 0,
    DHRUV_UPAGRAHA_POINT_MIDDLE = 1,
    DHRUV_UPAGRAHA_POINT_END = 2
};

enum {
    DHRUV_GULIKA_MAANDI_PLANET_RAHU = 0,
    DHRUV_GULIKA_MAANDI_PLANET_SATURN = 1
};

typedef struct {
    int32_t gulika_point;
    int32_t maandi_point;
    int32_t other_point;
    int32_t gulika_planet;
    int32_t maandi_planet;
} DhruvTimeUpagrahaConfig;

typedef struct {
    double gulika;
    double maandi;
    double kaala;
    double mrityu;
    double artha_prahara;
    double yama_ghantaka;
    double dhooma;
    double vyatipata;
    double parivesha;
    double indra_chapa;
    double upaketu;
} DhruvAllUpagrahas;

/* --- Ashtakavarga --- */

typedef struct {
    uint8_t graha_index;
    uint8_t points[12];
    uint8_t contributors[12][8];
} DhruvBhinnaAshtakavarga;

typedef struct {
    uint8_t total_points[12];
    uint8_t after_trikona[12];
    uint8_t after_ekadhipatya[12];
} DhruvSarvaAshtakavarga;

typedef struct {
    DhruvBhinnaAshtakavarga bavs[7];
    DhruvSarvaAshtakavarga  sav;
} DhruvAshtakavargaResult;

/* --- Drishti --- */

typedef struct {
    double angular_distance;
    double base_virupa;
    double special_virupa;
    double total_virupa;
} DhruvDrishtiEntry;

typedef struct {
    DhruvDrishtiEntry entries[9][9];
} DhruvGrahaDrishtiMatrix;

typedef struct {
    uint8_t include_bhava;
    uint8_t include_lagna;
    uint8_t include_bindus;
} DhruvDrishtiConfig;

typedef struct {
    DhruvDrishtiEntry graha_to_graha[9][9];
    DhruvDrishtiEntry graha_to_bhava[9][12];
    DhruvDrishtiEntry graha_to_rashi_bhava[9][12];
    DhruvDrishtiEntry graha_to_lagna[9];
    DhruvDrishtiEntry graha_to_bindus[9][19];
} DhruvDrishtiResult;

/* --- Graha positions --- */

typedef struct {
    uint8_t include_basic_states;
    uint8_t include_sensitive_point_distances;
} DhruvBasicStatesConfig;

typedef struct {
    uint8_t exalted;
    uint8_t debilitated;
    uint8_t combust;
    uint8_t retrograde;
    uint8_t moolatrikone;
    uint8_t marankarak_sthana;
    uint8_t mrityubhaga;
    uint8_t pushkaramsha;
    uint8_t pushkarbhaga;
} DhruvBasicStates;

typedef struct {
    double mrityubhaga;
    double pushkarbhaga;
} DhruvSensitivePointDistances;

typedef struct {
    uint8_t include_nakshatra;
    uint8_t include_lagna;
    uint8_t include_outer_planets;
    uint8_t include_bhava;
    DhruvBasicStatesConfig basic_states_config;
    /* Compute geocentric equatorial coordinates per entry plus Greenwich
       sidereal time on the result (1 = yes). Equinox of date; nutation
       applied per the request's use_nutation flag. */
    uint8_t include_equatorial;
} DhruvGrahaPositionsConfig;

typedef struct {
    double  sidereal_longitude;
    uint8_t rashi_index;
    uint8_t nakshatra_index;
    uint8_t pada;
    uint8_t bhava_number;
    uint8_t rashi_bhava_number;
    uint8_t basic_states_valid;
    DhruvBasicStates basic_states;
    uint8_t sensitive_point_distances_valid;
    DhruvSensitivePointDistances sensitive_point_distances;
    uint8_t equatorial_valid;
    /* Geocentric right ascension in degrees [0, 360), equinox of date. */
    double  right_ascension_deg;
    /* Geocentric declination in degrees [-90, +90], equinox of date. */
    double  declination_deg;
    /* Geocentric ecliptic latitude in degrees (0 for point-like entries). */
    double  ecliptic_latitude_deg;
} DhruvGrahaEntry;

typedef struct {
    DhruvGrahaEntry grahas[9];
    DhruvGrahaEntry lagna;
    DhruvGrahaEntry outer_planets[3];
    uint8_t earth_orientation_valid;
    /* Greenwich Mean Sidereal Time in degrees [0, 360). */
    double  gmst_deg;
    /* Greenwich Apparent Sidereal Time in degrees [0, 360). */
    double  gast_deg;
} DhruvGrahaPositions;

/* --- Core Bindus --- */

typedef struct {
    uint8_t include_nakshatra;
    uint8_t include_bhava;
    DhruvTimeUpagrahaConfig upagraha_config;
} DhruvBindusConfig;

typedef struct {
    DhruvGrahaEntry arudha_padas[12];
    uint8_t rashi_bhava_arudha_padas_valid;
    DhruvGrahaEntry rashi_bhava_arudha_padas[12];
    DhruvGrahaEntry bhrigu_bindu;
    DhruvGrahaEntry pranapada_lagna;
    DhruvGrahaEntry gulika;
    DhruvGrahaEntry maandi;
    DhruvGrahaEntry hora_lagna;
    DhruvGrahaEntry ghati_lagna;
    DhruvGrahaEntry sree_lagna;
} DhruvBindusResult;

/* --- Graha longitudes --- */

typedef struct {
    double longitudes[9];
    uint8_t outer_planets_valid;
    double outer_planets[3];
} DhruvGrahaLongitudes;

typedef struct {
    int32_t kind;
    int32_t ayanamsha_system;
    uint8_t use_nutation;
    int32_t precession_model;
    int32_t reference_plane;
    /* Lunar-node model for Rahu/Ketu (DHRUV_NODE_MODE_MEAN = mean node,
       any other value = true node). */
    int32_t node_mode;
} DhruvGrahaLongitudesConfig;

typedef struct {
    uint8_t graha_index;
    double sidereal_longitude;
    double ayanamsha_deg;
    double reference_plane_longitude;
} DhruvMovingOsculatingApogeeEntry;

typedef struct {
    uint8_t count;
    DhruvMovingOsculatingApogeeEntry entries[DHRUV_MAX_OSCULATING_APOGEE_REQUESTS];
} DhruvMovingOsculatingApogees;

/* --- Amsha (divisional chart) --- */

/* Amsha point families. Each code names one DhruvAmshaChart array; a point's
 * identity is (family, index into that array). Resolve it with
 * dhruv_amsha_point_name() / dhruv_amsha_point_key(). Names are static and
 * never vary by amsha, variation or date, which is why they are queried rather
 * than repeated inside every DhruvAmshaEntry. */
#define DHRUV_AMSHA_POINT_FAMILY_LAGNA                    0  /*  1 point,  lagna */
#define DHRUV_AMSHA_POINT_FAMILY_GRAHA                    1  /*  9 points, grahas */
#define DHRUV_AMSHA_POINT_FAMILY_OUTER_PLANET             2  /*  3 points, outer_planets */
#define DHRUV_AMSHA_POINT_FAMILY_BHAVA_CUSP               3  /* 12 points, bhava_cusps */
#define DHRUV_AMSHA_POINT_FAMILY_RASHI_BHAVA_CUSP         4  /* 12 points, rashi_bhava_cusps */
#define DHRUV_AMSHA_POINT_FAMILY_ARUDHA_PADA              5  /* 12 points, arudha_padas */
#define DHRUV_AMSHA_POINT_FAMILY_RASHI_BHAVA_ARUDHA_PADA  6  /* 12 points, rashi_bhava_arudha_padas */
#define DHRUV_AMSHA_POINT_FAMILY_UPAGRAHA                 7  /* 11 points, upagrahas */
#define DHRUV_AMSHA_POINT_FAMILY_SPHUTA                   8  /* 16 points, sphutas */
#define DHRUV_AMSHA_POINT_FAMILY_SPECIAL_LAGNA            9  /*  8 points, special_lagnas */
#define DHRUV_AMSHA_POINT_FAMILY_COUNT                   10

typedef struct {
    double   sidereal_longitude;
    uint8_t  rashi_index;
    uint16_t dms_degrees;
    uint8_t  dms_minutes;
    uint8_t  nakshatra_index;      /* 0-26 */
    uint8_t  pada;                 /* 1-4 */
    uint8_t  rashi_bhava_number;   /* 1-12, whole-sign from the varga lagna */
    double   dms_seconds;
    double   degrees_in_rashi;
} DhruvAmshaEntry;

typedef struct {
    uint8_t include_bhava_cusps;
    uint8_t include_arudha_padas;
    uint8_t include_upagrahas;
    uint8_t include_sphutas;
    uint8_t include_special_lagnas;
    uint8_t include_outer_planets;
} DhruvAmshaChartScope;

typedef struct {
    uint8_t  count;
    uint16_t codes[40];
    uint8_t  variations[40];
} DhruvAmshaSelectionConfig;

typedef struct {
    uint16_t amsha_code;
    uint8_t  variation_code;
    char     name[DHRUV_AMSHA_VARIATION_NAME_CAPACITY];
    char     label[DHRUV_AMSHA_VARIATION_LABEL_CAPACITY];
    uint8_t  is_default;
    char     description[DHRUV_AMSHA_VARIATION_DESCRIPTION_CAPACITY];
} DhruvAmshaVariationInfo;

typedef struct {
    uint16_t                amsha_code;
    uint8_t                 default_variation_code;
    uint8_t                 count;
    DhruvAmshaVariationInfo variations[DHRUV_MAX_AMSHA_VARIATIONS];
} DhruvAmshaVariationList;

typedef struct {
    uint8_t                 count;
    DhruvAmshaVariationList lists[40];
} DhruvAmshaVariationCatalogs;

typedef struct {
    uint16_t        amsha_code;
    uint8_t         variation_code;
    DhruvAmshaEntry grahas[9];
    uint8_t         outer_planets_valid;
    DhruvAmshaEntry outer_planets[3];
    DhruvAmshaEntry lagna;
    uint8_t         bhava_cusps_valid;
    DhruvAmshaEntry bhava_cusps[12];
    uint8_t         rashi_bhava_cusps_valid;
    DhruvAmshaEntry rashi_bhava_cusps[12];
    uint8_t         arudha_padas_valid;
    DhruvAmshaEntry arudha_padas[12];
    uint8_t         rashi_bhava_arudha_padas_valid;
    DhruvAmshaEntry rashi_bhava_arudha_padas[12];
    uint8_t         upagrahas_valid;
    DhruvAmshaEntry upagrahas[11];
    uint8_t         sphutas_valid;
    DhruvAmshaEntry sphutas[16];
    uint8_t         special_lagnas_valid;
    DhruvAmshaEntry special_lagnas[8];
} DhruvAmshaChart;

/* Slim varga chart within an amsha series point. grahas is populated
   (grahas_valid = 1) when the series was requested with include_grahas. */
typedef struct {
    uint16_t        amsha_code;
    uint8_t         variation_code;
    DhruvAmshaEntry lagna;
    uint8_t         grahas_valid;
    DhruvAmshaEntry grahas[9];
} DhruvAmshaSeriesChart;

/* One varga-lagna rashi segment. The first segment of a sweep starts at the
   requested from_utc; later segments start at the exact transition. end is
   the exact transition time (the last segment's end is the first transition
   at or after to_utc). */
typedef struct {
    uint8_t      rashi_index;   /* 0-based rashi index (0-11) */
    DhruvUtcTime start;
    DhruvUtcTime end;
} DhruvAmshaLagnaSegment;

/* --- Charakaraka --- */

typedef struct {
    uint8_t role_code;
    uint8_t graha_index;
    uint8_t rank;
    double  longitude_deg;
    double  degrees_in_rashi;
    double  effective_degrees_in_rashi;
} DhruvCharakarakaEntry;

typedef struct {
    uint8_t              scheme;
    uint8_t              used_eight_karakas;
    uint8_t              count;
    DhruvCharakarakaEntry entries[8];
} DhruvCharakarakaResult;

/* One chara-karaka ranking change. before/after reuse the per-moment
   result shape; their entry order is the documented ranking order
   (effective degree desc, then raw degrees-in-rashi desc, then graha
   index asc). changed_roles_mask has bit N set when the role with code N
   changed its assigned graha (a role present on only one side counts). */
typedef struct {
    DhruvUtcTime           utc;
    double                 jd_tdb;
    uint8_t                trigger;            /* DHRUV_CHARAKARAKA_TRIGGER_* */
    uint16_t               changed_roles_mask; /* bit N = role code N (0-8) */
    DhruvCharakarakaResult before;
    DhruvCharakarakaResult after;
} DhruvCharakarakaChangeEvent;

/* --- Shadbala & Vimsopaka --- */

typedef struct {
    double uchcha;
    double saptavargaja;
    double ojhayugma;
    double kendradi;
    double drekkana;
    double total;
} DhruvSthanaBalaBreakdown;

typedef struct {
    double nathonnatha;
    double paksha;
    double tribhaga;
    double abda;
    double masa;
    double vara;
    double hora;
    double ayana;
    double yuddha;
    double total;
} DhruvKalaBalaBreakdown;

typedef struct {
    uint8_t                  graha_index;
    DhruvSthanaBalaBreakdown sthana;
    double                   dig;
    DhruvKalaBalaBreakdown   kala;
    double                   cheshta;
    double                   naisargika;
    double                   drik;
    double                   total_shashtiamsas;
    double                   total_rupas;
    double                   required_strength;
    uint8_t                  is_strong;
} DhruvShadbalaEntry;

typedef struct {
    DhruvShadbalaEntry entries[7];
} DhruvShadbalaResult;

typedef struct {
    uint8_t bhava_number;
    double  cusp_sidereal_lon;
    uint8_t rashi_index;
    uint8_t lord_graha_index;
    double  bhavadhipati;
    double  dig;
    double  drishti;
    double  occupation_bonus;
    double  rising_bonus;
    double  total_virupas;
    double  total_rupas;
} DhruvBhavaBalaEntry;

typedef struct {
    DhruvBhavaBalaEntry entries[12];
} DhruvBhavaBalaResult;

typedef struct {
    double   cusp_sidereal_lons[12];
    double   ascendant_sidereal_lon;
    double   meridian_sidereal_lon;
    uint8_t  graha_bhava_numbers[9];
    double   graha_sidereal_lons[9];
    double   house_lord_strengths[12];
    double   aspect_virupas[9][12];
    uint8_t  include_node_aspects;
    uint8_t  include_special_rules;
    int32_t  chandra_benefic_rule;
    uint32_t birth_period;
} DhruvBhavaBalaInputs;

typedef struct {
    uint8_t graha_index;
    double  shadvarga;
    double  saptavarga;
    double  dashavarga;
    double  shodasavarga;
} DhruvVimsopakaEntry;

typedef struct {
    DhruvVimsopakaEntry entries[9];
} DhruvVimsopakaResult;

typedef struct {
    DhruvShadbalaResult     shadbala;
    DhruvVimsopakaResult    vimsopaka;
    DhruvAshtakavargaResult ashtakavarga;
    DhruvBhavaBalaResult    bhavabala;
} DhruvBalaBundleResult;

/* --- Avastha --- */

typedef struct {
    uint8_t avastha;
    uint8_t sub_states[5];
} DhruvSayanadiResult;

typedef struct {
    uint8_t             baladi;
    uint8_t             jagradadi;
    uint8_t             deeptadi;
    uint16_t            deeptadi_mask;
    uint8_t             deeptadi_count;
    uint8_t             deeptadi_states[9];
    uint8_t             lajjitadi;
    uint8_t             lajjitadi_valid;
    uint16_t            lajjitadi_mask;
    uint8_t             lajjitadi_count;
    uint8_t             lajjitadi_states[6];
    DhruvSayanadiResult sayanadi;
} DhruvGrahaAvasthas;

typedef struct {
    DhruvGrahaAvasthas entries[9];
} DhruvAllGrahaAvasthas;

/* --- Dasha --- */

typedef struct {
    uint8_t  entity_type;
    uint8_t  entity_index;
    const char *entity_name;
    double   start_jd;
    double   end_jd;
    DhruvUtcTime start_utc;
    DhruvUtcTime end_utc;
    uint8_t  level;
    uint16_t order;
    uint32_t parent_idx;
} DhruvDashaPeriod;

typedef struct {
    uint8_t          system;
    double           query_jd;
    DhruvUtcTime     query_utc;
    uint8_t          count;
    DhruvDashaPeriod periods[5];
} DhruvDashaSnapshot;

typedef struct {
    int32_t      time_kind;
    double       jd_utc;
    DhruvUtcTime utc;
} DhruvDashaSnapshotTime;

typedef struct {
    uint8_t count;
    uint8_t systems[DHRUV_MAX_DASHA_SYSTEMS];
    uint8_t max_levels[DHRUV_MAX_DASHA_SYSTEMS];
    uint8_t max_level;
    uint8_t level_methods[5];
    uint8_t yogini_scheme;
    uint8_t use_abhijit;
    /* Level-0 whole-cycle repetition count (0 = system default).
       Nakshatra-based and Yogini systems only; wins over min_span_years. */
    uint8_t cycles;
    /* Repeat level-0 whole cycles until coverage from birth reaches this
       many years (0.0 or negative = disabled). */
    double  min_span_years;
    DhruvDashaSnapshotTime snapshot_time;
} DhruvDashaSelectionConfig;

typedef struct {
    uint8_t level_methods[5];
    uint8_t yogini_scheme;
    uint8_t use_abhijit;
    /* Level-0 whole-cycle repetition count (0 = system default).
       Nakshatra-based and Yogini systems only; wins over min_span_years. */
    uint8_t cycles;
    /* Repeat level-0 whole cycles until coverage from birth reaches this
       many years (0.0 or negative = disabled). */
    double  min_span_years;
} DhruvDashaVariationConfig;

typedef struct {
    double graha_sidereal_lons[9];
    double lagna_sidereal_lon;
} DhruvRashiDashaInputs;

typedef struct {
    uint8_t               has_moon_sid_lon;
    double                moon_sid_lon;
    uint8_t               has_rashi_inputs;
    DhruvRashiDashaInputs rashi_inputs;
    uint8_t               has_sunrise_sunset;
    double                sunrise_jd;
    double                sunset_jd;
} DhruvDashaInputs;

typedef struct {
    int32_t              time_kind;
    double               birth_jd;
    DhruvUtcTime         birth_utc;
    uint8_t              has_location;
    DhruvGeoLocation     location;
    DhruvBhavaConfig     bhava_config;
    DhruvRiseSetConfig   riseset_config;
    DhruvSankrantiConfig sankranti_config;
    uint8_t              has_inputs;
    DhruvDashaInputs     inputs;
} DhruvDashaBirthContext;

typedef struct {
    DhruvDashaBirthContext   birth;
    uint8_t                  system;
    uint8_t                  max_level;
    DhruvDashaVariationConfig variation;
} DhruvDashaHierarchyRequest;

typedef struct {
    DhruvDashaBirthContext   birth;
    int32_t                  query_time_kind;
    double                   query_jd;
    DhruvUtcTime             query_utc;
    uint8_t                  system;
    uint8_t                  max_level;
    DhruvDashaVariationConfig variation;
} DhruvDashaSnapshotRequest;

typedef struct {
    DhruvDashaBirthContext   birth;
    uint8_t                  system;
    DhruvDashaVariationConfig variation;
} DhruvDashaLevel0Request;

typedef struct {
    DhruvDashaBirthContext   birth;
    uint8_t                  system;
    uint8_t                  entity_type;
    uint8_t                  entity_index;
    DhruvDashaVariationConfig variation;
} DhruvDashaLevel0EntityRequest;

typedef struct {
    DhruvDashaBirthContext   birth;
    uint8_t                  system;
    DhruvDashaVariationConfig variation;
    DhruvDashaPeriod         parent;
} DhruvDashaChildrenRequest;

typedef struct {
    DhruvDashaBirthContext   birth;
    uint8_t                  system;
    DhruvDashaVariationConfig variation;
    DhruvDashaPeriod         parent;
    uint8_t                  child_entity_type;
    uint8_t                  child_entity_index;
} DhruvDashaChildPeriodRequest;

typedef struct {
    DhruvDashaBirthContext   birth;
    uint8_t                  system;
    DhruvDashaVariationConfig variation;
    uint8_t                  child_level;
} DhruvDashaCompleteLevelRequest;

/* --- Full Kundali --- */

typedef struct {
    uint8_t  include_bhava_cusps;
    uint8_t  include_graha_positions;
    uint8_t  include_bindus;
    uint8_t  include_drishti;
    uint8_t  include_ashtakavarga;
    uint8_t  include_upagrahas;
    uint8_t  include_sphutas;
    uint8_t  include_special_lagnas;
    uint8_t  include_amshas;
    uint8_t  include_shadbala;
    uint8_t  include_bhavabala;
    uint8_t  include_vimsopaka;
    uint8_t  include_avastha;
    uint8_t  include_charakaraka;
    uint8_t  charakaraka_scheme;
    uint32_t node_dignity_policy;
    DhruvTimeUpagrahaConfig  upagraha_config;
    DhruvGrahaPositionsConfig graha_positions_config;
    DhruvBindusConfig         bindus_config;
    DhruvDrishtiConfig        drishti_config;
    DhruvAmshaChartScope      amsha_scope;
    DhruvAmshaSelectionConfig amsha_selection;
    /* DHRUV_PANCHANG_INCLUDE_* bits; 0 omits the panchang section. */
    uint32_t panchang_include_mask;
    uint8_t  include_dasha;
    DhruvDashaSelectionConfig dasha_config;
} DhruvFullKundaliConfig;

typedef struct {
    DhruvUtcTime             birth_utc;
    DhruvUtcTime             at_utc;
    DhruvGeoLocation         location;
    DhruvBhavaConfig         bhava_config;
    DhruvRiseSetConfig       riseset_config;
    DhruvSankrantiConfig     sankranti_config;
    DhruvFullKundaliConfig   kundali_config;
    DhruvGocharEventsConfig  config;
    /* Physical body Body::code(), or DHRUV_GOCHAR_TRANSIT_RAHU/KETU. */
    const uint32_t           *transit_body_codes;
    uint32_t                 transit_body_count;
    const DhruvGocharNatalTarget *natal_targets;
    uint32_t                 natal_target_count;
} DhruvGocharEventsRequest;

typedef struct {
    double                    ayanamsha_deg;
    uint8_t                   bhava_cusps_valid;
    DhruvBhavaResult          bhava_cusps;
    uint8_t                   rashi_bhava_cusps_valid;
    DhruvBhavaResult          rashi_bhava_cusps;
    uint8_t                   bhava_cusp_sensitive_point_distances_valid;
    DhruvSensitivePointDistances bhava_cusp_sensitive_point_distances[12];
    uint8_t                   rashi_bhava_cusp_sensitive_point_distances_valid;
    DhruvSensitivePointDistances rashi_bhava_cusp_sensitive_point_distances[12];
    uint8_t                   graha_positions_valid;
    DhruvGrahaPositions       graha_positions;
    uint8_t                   bindus_valid;
    DhruvBindusResult         bindus;
    uint8_t                   drishti_valid;
    DhruvDrishtiResult        drishti;
    uint8_t                   ashtakavarga_valid;
    DhruvAshtakavargaResult   ashtakavarga;
    uint8_t                   upagrahas_valid;
    DhruvAllUpagrahas         upagrahas;
    uint8_t                   sphutas_valid;
    DhruvSphutalResult        sphutas;
    uint8_t                   special_lagnas_valid;
    DhruvSpecialLagnas        special_lagnas;
    uint8_t                   amshas_valid;
    uint8_t                   amshas_count;
    DhruvAmshaChart           amshas[40];
    uint8_t                   shadbala_valid;
    DhruvShadbalaResult       shadbala;
    uint8_t                   bhavabala_valid;
    DhruvBhavaBalaResult      bhavabala;
    uint8_t                   vimsopaka_valid;
    DhruvVimsopakaResult      vimsopaka;
    uint8_t                   avastha_valid;
    DhruvAllGrahaAvasthas     avastha;
    uint8_t                   charakaraka_valid;
    DhruvCharakarakaResult    charakaraka;
    uint8_t                   panchang_valid;
    DhruvPanchangOperationResult panchang;
    uint8_t                   dasha_count;
    DhruvDashaHierarchyHandle dasha_handles[DHRUV_MAX_DASHA_SYSTEMS];
    uint8_t                   dasha_systems[DHRUV_MAX_DASHA_SYSTEMS];
    uint8_t                   dasha_snapshot_count;
    DhruvDashaSnapshot        dasha_snapshots[DHRUV_MAX_DASHA_SYSTEMS];
} DhruvFullKundaliResult;

/* --- Tara (fixed star) --- */

typedef struct {
    double ra_deg;
    double dec_deg;
    double distance_au;
} DhruvEquatorialPosition;

typedef struct {
    double position_au[3];
    double velocity_au_day[3];
} DhruvEarthState;

typedef struct {
    int32_t accuracy;
    uint8_t apply_parallax;
} DhruvTaraConfig;

typedef struct {
    int32_t            tara_id;
    int32_t            output_kind;
    double             jd_tdb;
    double             ayanamsha_deg;
    DhruvTaraConfig    config;
    uint8_t            earth_state_valid;
    DhruvEarthState    earth_state;
} DhruvTaraComputeRequest;

typedef struct {
    int32_t                 output_kind;
    DhruvEquatorialPosition equatorial;
    DhruvSphericalCoords    ecliptic;
    double                  sidereal_longitude_deg;
} DhruvTaraComputeResult;

/* ===================================================================
 * Functions
 * =================================================================== */

/* --- Versioning --- */
uint32_t dhruv_api_version(void);

/* --- Config --- */
DhruvStatus dhruv_config_load(
    const char *path_utf8,
    int32_t defaults_mode,
    DhruvConfigHandle **out_handle);
DhruvStatus dhruv_config_free(DhruvConfigHandle *handle);
DhruvStatus dhruv_config_clear_active(void);

/* --- Engine lifecycle --- */
DhruvStatus dhruv_engine_new(
    const DhruvEngineConfig *config,
    DhruvEngineHandle **out);
DhruvStatus dhruv_engine_query(
    const DhruvEngineHandle *engine,
    const DhruvQuery *query,
    DhruvStateVector *out);
DhruvStatus dhruv_engine_query_request(
    const DhruvEngineHandle *engine,
    const DhruvQueryRequest *request,
    DhruvQueryResult *out);
DhruvStatus dhruv_engine_replace_spks(
    const DhruvEngineHandle *engine,
    const DhruvSpkSetConfig *config,
    DhruvSpkReplaceReport *out);
DhruvStatus dhruv_engine_list_spks(
    const DhruvEngineHandle *engine,
    DhruvLoadedSpkList *out);
DhruvStatus dhruv_engine_free(DhruvEngineHandle *engine);
DhruvStatus dhruv_query_once(
    const DhruvEngineConfig *config,
    const DhruvQuery *query,
    DhruvStateVector *out);

/* --- LSK (leap-second kernel) --- */
DhruvStatus dhruv_lsk_load(const char *path, DhruvLskHandle **out);
DhruvStatus dhruv_lsk_free(DhruvLskHandle *lsk);

/* --- Time conversion --- */
DhruvStatus dhruv_utc_to_tdb_jd(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvUtcToTdbRequest *request,
    DhruvUtcToTdbResult *out);

/* --- Coordinate transform --- */
DhruvStatus dhruv_cartesian_to_spherical(
    const double *position_km,
    DhruvSphericalCoords *out);

/* --- EOP --- */
DhruvStatus dhruv_eop_load(const char *path, DhruvEopHandle **out);
DhruvStatus dhruv_eop_free(DhruvEopHandle *eop);

/* --- Ayanamsha --- */
uint32_t dhruv_ayanamsha_system_count(void);
int32_t  dhruv_reference_plane_default(int32_t system_code);
DhruvStatus dhruv_ayanamsha_compute_ex(
    const DhruvLskHandle *lsk,
    const DhruvAyanamshaComputeRequest *request,
    const DhruvEopHandle *eop,
    double *out);

/* --- Nutation --- */
DhruvStatus dhruv_nutation_iau2000b(double jd_tdb, double *dpsi, double *deps);

/* --- UTC time helpers --- */
DhruvStatus dhruv_jd_tdb_to_utc(
    const DhruvLskHandle *lsk, double jd_tdb,
    DhruvUtcTime *out);
DhruvStatus dhruv_riseset_result_to_utc(
    const DhruvLskHandle *lsk,
    const DhruvRiseSetResult *result,
    DhruvUtcTime *out);

/* --- Rise/set --- */
DhruvRiseSetConfig dhruv_riseset_config_default(void);
DhruvStatus dhruv_compute_rise_set(
    const DhruvEngineHandle *engine,
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    int32_t event_code,
    double  jd_tdb_approx,
    const DhruvRiseSetConfig *config,
    DhruvRiseSetResult *out);
DhruvStatus dhruv_compute_all_events(
    const DhruvEngineHandle *engine,
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    double jd_tdb_approx,
    const DhruvRiseSetConfig *config,
    DhruvRiseSetResult *out);
double dhruv_approximate_local_noon_jd(double jd_ut_midnight, double longitude_deg);

/* --- Bhava --- */
DhruvBhavaConfig dhruv_bhava_config_default(void);
uint32_t dhruv_bhava_system_count(void);
DhruvStatus dhruv_compute_bhavas(
    const DhruvEngineHandle *engine,
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    double jd_tdb,
    const DhruvBhavaConfig *config,
    DhruvBhavaResult *out);
DhruvStatus dhruv_lagna_deg(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    double jd_tdb,
    double *out);
DhruvStatus dhruv_lagna_deg_with_config(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    double jd_tdb,
    const DhruvBhavaConfig *config,
    double *out);
DhruvStatus dhruv_mc_deg(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    double jd_tdb,
    double *out);
DhruvStatus dhruv_mc_deg_with_config(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    double jd_tdb,
    const DhruvBhavaConfig *config,
    double *out);
DhruvStatus dhruv_ramc_deg(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    double jd_tdb,
    double *out);

/* --- Lunar node --- */
DhruvStatus dhruv_lunar_node_deg(
    int32_t node_code, int32_t mode_code,
    double jd_tdb, double *out);
DhruvStatus dhruv_lunar_node_deg_with_engine(
    const DhruvEngineHandle *engine,
    int32_t node_code, int32_t mode_code,
    double jd_tdb, double *out);
DhruvStatus dhruv_lunar_node_compute_ex(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvLunarNodeRequest *request,
    double *out);
uint32_t dhruv_lunar_node_count(void);

/* --- Conjunction --- */
DhruvConjunctionConfig dhruv_conjunction_config_default(void);
DhruvStatus dhruv_conjunction_search_ex(
    const DhruvEngineHandle *engine,
    const DhruvConjunctionSearchRequest *request,
    DhruvConjunctionEvent *out_event,
    uint8_t *out_found,
    DhruvConjunctionEvent *out_events,
    uint32_t out_capacity,
    uint32_t *out_count);

/* --- Grahan (eclipse) --- */
DhruvGrahanConfig dhruv_grahan_config_default(void);
/* Writes the configuration actually applied after clamping/sanitizing;
   build cache keys against this echo rather than the raw request. */
DhruvStatus dhruv_grahan_config_effective(
    const DhruvGrahanConfig *config,
    DhruvGrahanConfig *out);
DhruvStatus dhruv_grahan_search_ex(
    const DhruvEngineHandle *engine,
    const DhruvGrahanSearchRequest *request,
    DhruvChandraGrahanResult *out_chandra,
    DhruvSuryaGrahanResult *out_surya,
    uint8_t *out_found,
    DhruvChandraGrahanResult *out_chandra_events,
    DhruvSuryaGrahanResult *out_surya_events,
    uint32_t out_capacity,
    uint32_t *out_count);
DhruvStatus dhruv_surya_grahan_path_point_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t index,
    DhruvSuryaGrahanPathPoint *out);
DhruvStatus dhruv_surya_grahan_footprint_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t index,
    DhruvSuryaGrahanFootprint *out);
DhruvStatus dhruv_surya_grahan_footprint_point_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t footprint_index,
    uint32_t point_index,
    DhruvEclipseGeoPoint *out);
DhruvStatus dhruv_surya_grahan_footprint_magnitude_ring_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t footprint_index,
    uint32_t ring_index,
    DhruvSuryaMagnitudeRing *out);
DhruvStatus dhruv_surya_grahan_footprint_magnitude_ring_point_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t footprint_index,
    uint32_t ring_index,
    uint32_t point_index,
    DhruvEclipseGeoPoint *out);
DhruvStatus dhruv_surya_grahan_contact_magnitude_ring_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t footprint_index,
    uint32_t ring_index,
    DhruvSuryaMagnitudeRing *out);
DhruvStatus dhruv_surya_grahan_contact_magnitude_ring_point_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t footprint_index,
    uint32_t ring_index,
    uint32_t point_index,
    DhruvEclipseGeoPoint *out);
DhruvStatus dhruv_surya_grahan_contact_footprint_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t index,
    DhruvSuryaContactFootprint *out);
DhruvStatus dhruv_surya_grahan_contact_footprint_point_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t footprint_index,
    uint32_t point_index,
    DhruvEclipseGeoPoint *out);
DhruvStatus dhruv_surya_grahan_umbra_footprint_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t index,
    DhruvSuryaUmbraFootprint *out);
DhruvStatus dhruv_surya_grahan_umbra_footprint_point_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t footprint_index,
    uint32_t point_index,
    DhruvEclipseGeoPoint *out);
DhruvStatus dhruv_surya_grahan_local_grid_sample_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    uint32_t index,
    DhruvSuryaLocalGridSample *out);
DhruvStatus dhruv_surya_grahan_ring_set_level_count(
    DhruvSuryaGrahanGeometryHandle geometry,
    int32_t set_kind,
    uint32_t *out);
DhruvStatus dhruv_surya_grahan_ring_set_level_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    int32_t set_kind,
    uint32_t level_index,
    DhruvSuryaRingSetLevel *out);
DhruvStatus dhruv_surya_grahan_ring_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    int32_t set_kind,
    uint32_t level_index,
    uint32_t ring_index,
    DhruvSuryaIsolineRing *out);
DhruvStatus dhruv_surya_grahan_ring_point_at(
    DhruvSuryaGrahanGeometryHandle geometry,
    int32_t set_kind,
    uint32_t level_index,
    uint32_t ring_index,
    uint32_t point_index,
    DhruvEclipseGeoPoint *out);
void dhruv_surya_grahan_geometry_free(
    DhruvSuryaGrahanGeometryHandle geometry);

/* --- Stationary / max-speed --- */
DhruvStationaryConfig dhruv_stationary_config_default(void);
DhruvStatus dhruv_motion_search_ex(
    const DhruvEngineHandle *engine,
    const DhruvMotionSearchRequest *request,
    DhruvStationaryEvent *out_stationary,
    DhruvMaxSpeedEvent *out_max_speed,
    uint8_t *out_found,
    DhruvStationaryEvent *out_stationary_events,
    DhruvMaxSpeedEvent *out_max_speed_events,
    uint32_t out_capacity,
    uint32_t *out_count);

/* --- Rashi / Nakshatra --- */
DhruvStatus dhruv_deg_to_dms(double degrees, DhruvDms *out);
DhruvStatus dhruv_rashi_from_longitude(double sidereal_lon, DhruvRashiInfo *out);
DhruvStatus dhruv_nakshatra_from_longitude(double sidereal_lon, DhruvNakshatraInfo *out);
DhruvStatus dhruv_nakshatra28_from_longitude(double sidereal_lon, DhruvNakshatra28Info *out);
DhruvStatus dhruv_rashi_from_tropical(
    double tropical_lon, uint32_t ayanamsha_system,
    double jd_tdb, uint8_t use_nutation,
    DhruvRashiInfo *out);
DhruvStatus dhruv_nakshatra_from_tropical(
    double tropical_lon, uint32_t ayanamsha_system,
    double jd_tdb, uint8_t use_nutation,
    DhruvNakshatraInfo *out);
DhruvStatus dhruv_nakshatra28_from_tropical(
    double tropical_lon, uint32_t ayanamsha_system,
    double jd_tdb, uint8_t use_nutation,
    DhruvNakshatra28Info *out);
uint32_t dhruv_rashi_count(void);
uint32_t dhruv_nakshatra_count(uint32_t scheme_code);
const char *dhruv_rashi_name(uint32_t index);
const char *dhruv_nakshatra_name(uint32_t index);
const char *dhruv_nakshatra28_name(uint32_t index);

/* --- Sankranti / Lunar phase --- */
DhruvSankrantiConfig dhruv_sankranti_config_default(void);
DhruvStatus dhruv_lunar_phase_search_ex(
    const DhruvEngineHandle *engine,
    const DhruvLunarPhaseSearchRequest *request,
    DhruvLunarPhaseEvent *out_event,
    uint8_t *out_found,
    DhruvLunarPhaseEvent *out_events,
    uint32_t out_capacity,
    uint32_t *out_count);
DhruvStatus dhruv_sankranti_search_ex(
    const DhruvEngineHandle *engine,
    const DhruvSankrantiSearchRequest *request,
    DhruvSankrantiEvent *out_event,
    uint8_t *out_found,
    DhruvSankrantiEvent *out_events,
    uint32_t out_capacity,
    uint32_t *out_count);
/* --- Calendar --- */
DhruvStatus dhruv_masa_for_date(
    const DhruvEngineHandle *engine,
    const DhruvUtcTime *utc,
    const DhruvSankrantiConfig *config,
    DhruvMasaInfo *out);
DhruvStatus dhruv_ayana_for_date(
    const DhruvEngineHandle *engine,
    const DhruvUtcTime *utc,
    const DhruvSankrantiConfig *config,
    DhruvAyanaInfo *out);
DhruvStatus dhruv_varsha_for_date(
    const DhruvEngineHandle *engine,
    const DhruvUtcTime *utc,
    const DhruvSankrantiConfig *config,
    DhruvVarshaInfo *out);
const char *dhruv_masa_name(uint32_t index);
const char *dhruv_ayana_name(uint32_t index);
const char *dhruv_samvatsara_name(uint32_t index);

/* --- Pure-math panchang classifiers --- */
DhruvStatus dhruv_tithi_from_elongation(double elongation_deg, DhruvTithiPosition *out);
DhruvStatus dhruv_karana_from_elongation(double elongation_deg, DhruvKaranaPosition *out);
DhruvStatus dhruv_yoga_from_sum(double sum_deg, DhruvYogaPosition *out);
int32_t dhruv_vaar_from_jd(double jd);
int32_t dhruv_masa_from_rashi_index(uint32_t rashi_index);
int32_t dhruv_ayana_from_sidereal_longitude(double lon_deg);
DhruvStatus dhruv_samvatsara_from_year(int32_t ce_year, DhruvSamvatsaraResult *out);
int32_t dhruv_nth_rashi_from(uint32_t rashi_index, uint32_t offset);

/* --- UTC wrapper functions --- */
DhruvStatus dhruv_compute_rise_set_utc(
    const DhruvEngineHandle *engine,
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    int32_t event_code,
    const DhruvUtcTime *utc,
    const DhruvRiseSetConfig *config,
    DhruvRiseSetResultUtc *out);
DhruvStatus dhruv_compute_all_events_utc(
    const DhruvEngineHandle *engine,
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    const DhruvUtcTime *utc,
    const DhruvRiseSetConfig *config,
    DhruvRiseSetResultUtc *out);
DhruvStatus dhruv_compute_bhavas_utc(
    const DhruvEngineHandle *engine,
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    const DhruvUtcTime *utc,
    const DhruvBhavaConfig *config,
    DhruvBhavaResult *out);
DhruvStatus dhruv_lagna_deg_utc(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    const DhruvUtcTime *utc,
    double *out);
DhruvStatus dhruv_lagna_deg_utc_with_config(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    const DhruvUtcTime *utc,
    const DhruvBhavaConfig *config,
    double *out);
DhruvStatus dhruv_mc_deg_utc(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    const DhruvUtcTime *utc,
    double *out);
DhruvStatus dhruv_mc_deg_utc_with_config(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    const DhruvUtcTime *utc,
    const DhruvBhavaConfig *config,
    double *out);
DhruvStatus dhruv_ramc_deg_utc(
    const DhruvLskHandle *lsk,
    const DhruvEopHandle *eop,
    const DhruvGeoLocation *location,
    const DhruvUtcTime *utc,
    double *out);
DhruvStatus dhruv_nutation_iau2000b_utc(
    const DhruvLskHandle *lsk,
    const DhruvUtcTime *utc,
    double *dpsi, double *deps);
DhruvStatus dhruv_lunar_node_deg_utc(
    const DhruvLskHandle *lsk,
    int32_t node_code, int32_t mode_code,
    const DhruvUtcTime *utc,
    double *out);
DhruvStatus dhruv_lunar_node_deg_utc_with_engine(
    const DhruvEngineHandle *engine,
    const DhruvLskHandle *lsk,
    int32_t node_code, int32_t mode_code,
    const DhruvUtcTime *utc,
    double *out);
DhruvStatus dhruv_rashi_from_tropical_utc(
    const DhruvLskHandle *lsk,
    double tropical_lon, uint32_t ayanamsha_system,
    const DhruvUtcTime *utc, uint8_t use_nutation,
    DhruvRashiInfo *out);
DhruvStatus dhruv_nakshatra_from_tropical_utc(
    const DhruvLskHandle *lsk,
    double tropical_lon, uint32_t ayanamsha_system,
    const DhruvUtcTime *utc, uint8_t use_nutation,
    DhruvNakshatraInfo *out);
DhruvStatus dhruv_nakshatra28_from_tropical_utc(
    const DhruvLskHandle *lsk,
    double tropical_lon, uint32_t ayanamsha_system,
    const DhruvUtcTime *utc, uint8_t use_nutation,
    DhruvNakshatra28Info *out);
/* --- Panchang for-date functions --- */
DhruvStatus dhruv_tithi_for_date(
    const DhruvEngineHandle *engine,
    const DhruvUtcTime *utc,
    DhruvTithiInfo *out);
DhruvStatus dhruv_karana_for_date(
    const DhruvEngineHandle *engine,
    const DhruvUtcTime *utc,
    DhruvKaranaInfo *out);
DhruvStatus dhruv_yoga_for_date(
    const DhruvEngineHandle *engine,
    const DhruvUtcTime *utc,
    const DhruvSankrantiConfig *config,
    DhruvYogaInfo *out);
DhruvStatus dhruv_nakshatra_for_date(
    const DhruvEngineHandle *engine,
    const DhruvUtcTime *utc,
    const DhruvSankrantiConfig *config,
    DhruvPanchangNakshatraInfo *out);
DhruvStatus dhruv_vaar_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvRiseSetConfig *riseset_config,
    DhruvVaarInfo *out);
DhruvStatus dhruv_hora_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvRiseSetConfig *riseset_config,
    DhruvHoraInfo *out);
DhruvStatus dhruv_ghatika_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvRiseSetConfig *riseset_config,
    DhruvGhatikaInfo *out);

/* --- Unified panchang --- */
DhruvStatus dhruv_panchang_compute_ex(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvLskHandle *lsk,
    const DhruvPanchangComputeRequest *request,
    DhruvPanchangOperationResult *out);

/* --- Panchang events (range sweep) ---
   Streams exact panchang element segments overlapping [from_utc, to_utc].
   include_mask selects any DHRUV_PANCHANG_INCLUDE_* element bits; when a
   location-dependent bit (DHRUV_PANCHANG_INCLUDE_LOCATION_DEPENDENT: vaar,
   hora, ghatika) is set, has_location must be non-zero and location must
   point to the observer location (rejected otherwise); riseset_config may
   be NULL for defaults and is read only for those elements. max_events 0
   selects the hard ceiling DHRUV_MAX_PANCHANG_EVENTS. Consecutive segments
   of one kind chain exactly (end == next.start); vaar segments are
   sunrise-to-sunrise Vedic days, hora/ghatika their 24/60 subdivisions.
   The first segment of each kind may start before from_utc and the last
   may end after to_utc. When truncated, dhruv_panchang_events_meta yields
   the resume point. The returned handle must be freed with
   dhruv_panchang_events_free. */
typedef void *DhruvPanchangEventsHandle;

DhruvStatus dhruv_panchang_events(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *from_utc,
    const DhruvUtcTime *to_utc,
    uint32_t include_mask,
    uint8_t has_location,
    const DhruvGeoLocation *location,
    const DhruvRiseSetConfig *riseset_config,
    const DhruvSankrantiConfig *sankranti_config,
    uint32_t max_events,
    DhruvPanchangEventsHandle *out);
DhruvStatus dhruv_panchang_events_tithi_count(
    DhruvPanchangEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_panchang_events_tithi_at(
    DhruvPanchangEventsHandle handle,
    uint32_t idx,
    DhruvTithiInfo *out);
DhruvStatus dhruv_panchang_events_karana_count(
    DhruvPanchangEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_panchang_events_karana_at(
    DhruvPanchangEventsHandle handle,
    uint32_t idx,
    DhruvKaranaInfo *out);
DhruvStatus dhruv_panchang_events_yoga_count(
    DhruvPanchangEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_panchang_events_yoga_at(
    DhruvPanchangEventsHandle handle,
    uint32_t idx,
    DhruvYogaInfo *out);
DhruvStatus dhruv_panchang_events_nakshatra_count(
    DhruvPanchangEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_panchang_events_nakshatra_at(
    DhruvPanchangEventsHandle handle,
    uint32_t idx,
    DhruvPanchangNakshatraInfo *out);
DhruvStatus dhruv_panchang_events_vaar_count(
    DhruvPanchangEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_panchang_events_vaar_at(
    DhruvPanchangEventsHandle handle,
    uint32_t idx,
    DhruvVaarInfo *out);
DhruvStatus dhruv_panchang_events_hora_count(
    DhruvPanchangEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_panchang_events_hora_at(
    DhruvPanchangEventsHandle handle,
    uint32_t idx,
    DhruvHoraInfo *out);
DhruvStatus dhruv_panchang_events_ghatika_count(
    DhruvPanchangEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_panchang_events_ghatika_at(
    DhruvPanchangEventsHandle handle,
    uint32_t idx,
    DhruvGhatikaInfo *out);
DhruvStatus dhruv_panchang_events_masa_count(
    DhruvPanchangEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_panchang_events_masa_at(
    DhruvPanchangEventsHandle handle,
    uint32_t idx,
    DhruvMasaInfo *out);
DhruvStatus dhruv_panchang_events_ayana_count(
    DhruvPanchangEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_panchang_events_ayana_at(
    DhruvPanchangEventsHandle handle,
    uint32_t idx,
    DhruvAyanaInfo *out);
DhruvStatus dhruv_panchang_events_varsha_count(
    DhruvPanchangEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_panchang_events_varsha_at(
    DhruvPanchangEventsHandle handle,
    uint32_t idx,
    DhruvVarshaInfo *out);
DhruvStatus dhruv_panchang_events_meta(
    DhruvPanchangEventsHandle handle,
    uint8_t *out_truncated,
    uint8_t *out_next_from_valid,
    DhruvUtcTime *out_next_from_utc);
void dhruv_panchang_events_free(DhruvPanchangEventsHandle handle);

/* --- Panchang name lookups --- */
const char *dhruv_tithi_name(uint32_t index);
const char *dhruv_karana_name(uint32_t index);
const char *dhruv_yoga_name(uint32_t index);
const char *dhruv_vaar_name(uint32_t index);
const char *dhruv_hora_name(uint32_t index);

/* --- Panchang composable intermediates --- */
DhruvStatus dhruv_elongation_at(
    const DhruvEngineHandle *engine,
    double jd_tdb, double *out);
DhruvStatus dhruv_sidereal_sum_at(
    const DhruvEngineHandle *engine,
    double jd_tdb,
    const DhruvSankrantiConfig *config,
    double *out);
DhruvStatus dhruv_vedic_day_sunrises(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvRiseSetConfig *config,
    double *out_sunrise,
    double *out_next_sunrise);
DhruvStatus dhruv_body_ecliptic_lon_lat(
    const DhruvEngineHandle *engine,
    int32_t body_code, double jd_tdb,
    double *out_lon, double *out_lat);
DhruvStatus dhruv_tithi_at(
    const DhruvEngineHandle *engine,
    double jd_tdb, double sunrise_jd,
    DhruvTithiInfo *out);
DhruvStatus dhruv_karana_at(
    const DhruvEngineHandle *engine,
    double jd_tdb, double sunrise_jd,
    DhruvKaranaInfo *out);
DhruvStatus dhruv_yoga_at(
    const DhruvEngineHandle *engine,
    double jd_tdb, double sunrise_jd,
    const DhruvSankrantiConfig *config,
    DhruvYogaInfo *out);
DhruvStatus dhruv_vaar_from_sunrises(
    const DhruvLskHandle *lsk,
    double sunrise_jd, double next_sunrise_jd,
    DhruvVaarInfo *out);
DhruvStatus dhruv_hora_from_sunrises(
    const DhruvLskHandle *lsk,
    double query_jd,
    double sunrise_jd, double next_sunrise_jd,
    DhruvHoraInfo *out);
DhruvStatus dhruv_ghatika_from_sunrises(
    const DhruvLskHandle *lsk,
    double query_jd,
    double sunrise_jd, double next_sunrise_jd,
    DhruvGhatikaInfo *out);

/* --- Graha identifiers --- */
const char *dhruv_graha_name(uint32_t index);
const char *dhruv_yogini_name(uint32_t index);
int32_t dhruv_rashi_lord(uint32_t rashi_index);
int32_t dhruv_hora_lord(uint32_t vaar_index, uint32_t hora_index);
int32_t dhruv_masa_lord(uint32_t masa_index);
int32_t dhruv_samvatsara_lord(uint32_t samvatsara_index);

/* --- Graha relationship / dignity / combustion helper codes --- */
#define DHRUV_NAISARGIKA_FRIEND 0
#define DHRUV_NAISARGIKA_ENEMY 1
#define DHRUV_NAISARGIKA_NEUTRAL 2

#define DHRUV_TATKALIKA_FRIEND 0
#define DHRUV_TATKALIKA_ENEMY 1

#define DHRUV_PANCHADHA_ADHI_SHATRU 0
#define DHRUV_PANCHADHA_SHATRU 1
#define DHRUV_PANCHADHA_SAMA 2
#define DHRUV_PANCHADHA_MITRA 3
#define DHRUV_PANCHADHA_ADHI_MITRA 4

#define DHRUV_DIGNITY_EXALTED 0
#define DHRUV_DIGNITY_MOOLATRIKONE 1
#define DHRUV_DIGNITY_OWN_SIGN 2
#define DHRUV_DIGNITY_ADHI_MITRA 3
#define DHRUV_DIGNITY_MITRA 4
#define DHRUV_DIGNITY_SAMA 5
#define DHRUV_DIGNITY_SHATRU 6
#define DHRUV_DIGNITY_ADHI_SHATRU 7
#define DHRUV_DIGNITY_DEBILITATED 8

#define DHRUV_NODE_DIGNITY_SIGN_LORD_BASED 0
#define DHRUV_NODE_DIGNITY_ALWAYS_SAMA 1

#define DHRUV_BENEFIC_NATURE_BENEFIC 0
#define DHRUV_BENEFIC_NATURE_MALEFIC 1
#define DHRUV_CHANDRA_BENEFIC_RULE_BRIGHTNESS_72 0
#define DHRUV_CHANDRA_BENEFIC_RULE_WAXING_180 1
#define DHRUV_SAYANADI_GHATIKA_ROUNDING_FLOOR 0
#define DHRUV_SAYANADI_GHATIKA_ROUNDING_CEIL 1

#define DHRUV_GRAHA_GENDER_MALE 0
#define DHRUV_GRAHA_GENDER_FEMALE 1
#define DHRUV_GRAHA_GENDER_NEUTER 2

/* --- Graha relationship / dignity / combustion helpers --- */
DhruvStatus dhruv_exaltation_degree(
    uint32_t graha_index,
    uint8_t *out_has_value,
    double *out_value);
DhruvStatus dhruv_debilitation_degree(
    uint32_t graha_index,
    uint8_t *out_has_value,
    double *out_value);
DhruvStatus dhruv_moolatrikone_range(
    uint32_t graha_index,
    uint8_t *out_has_value,
    uint8_t *out_rashi_index,
    double *out_start_deg,
    double *out_end_deg);
DhruvStatus dhruv_combustion_threshold(
    uint32_t graha_index,
    uint8_t is_retrograde,
    uint8_t *out_has_value,
    double *out_threshold_deg);
DhruvStatus dhruv_is_combust(
    uint32_t graha_index,
    double graha_sid_lon,
    double sun_sid_lon,
    uint8_t is_retrograde,
    uint8_t *out_is_combust);
DhruvStatus dhruv_all_combustion_status(
    const double *sidereal_lons_9,
    const uint8_t *retrograde_flags_9,
    uint8_t *out_combust_flags_9);
DhruvStatus dhruv_naisargika_maitri(
    uint32_t graha_index,
    uint32_t other_index,
    int32_t *out_code);
DhruvStatus dhruv_tatkalika_maitri(
    uint32_t graha_rashi_index,
    uint32_t other_rashi_index,
    int32_t *out_code);
DhruvStatus dhruv_panchadha_maitri(
    int32_t naisargika_code,
    int32_t tatkalika_code,
    int32_t *out_code);
DhruvStatus dhruv_dignity_in_rashi(
    uint32_t graha_index,
    double sidereal_lon,
    uint32_t rashi_index,
    int32_t *out_code);
DhruvStatus dhruv_dignity_in_rashi_with_positions(
    uint32_t graha_index,
    double sidereal_lon,
    uint32_t rashi_index,
    const uint8_t *sapta_rashi_indices_7,
    int32_t *out_code);
DhruvStatus dhruv_node_dignity_in_rashi(
    uint32_t graha_index,
    uint32_t rashi_index,
    const uint8_t *graha_rashi_indices_9,
    int32_t policy_code,
    int32_t *out_code);
DhruvStatus dhruv_natural_benefic_malefic(
    uint32_t graha_index,
    int32_t *out_code);
DhruvStatus dhruv_moon_benefic_nature(
    double moon_sun_elongation,
    int32_t *out_code);
DhruvStatus dhruv_graha_gender(
    uint32_t graha_index,
    int32_t *out_code);

/* --- Sphuta --- */
const char *dhruv_sphuta_name(uint32_t index);
DhruvStatus dhruv_all_sphutas(
    const DhruvSphutalInputs *inputs,
    DhruvSphutalResult *out);
double dhruv_bhrigu_bindu(double rahu, double moon);
double dhruv_prana_sphuta(double lagna, double moon);
double dhruv_deha_sphuta(double moon, double lagna);
double dhruv_mrityu_sphuta(double eighth_lord, double lagna);
double dhruv_tithi_sphuta(double moon, double sun, double lagna);
double dhruv_yoga_sphuta(double sun, double moon);
double dhruv_yoga_sphuta_normalized(double sun, double moon);
double dhruv_rahu_tithi_sphuta(double rahu, double sun, double lagna);
double dhruv_kshetra_sphuta(
    double venus, double moon, double mars,
    double jupiter, double lagna);
double dhruv_beeja_sphuta(double sun, double venus, double jupiter);
double dhruv_trisphuta(double lagna, double moon, double gulika);
double dhruv_chatussphuta(double trisphuta_val, double sun);
double dhruv_panchasphuta(double chatussphuta_val, double rahu);
double dhruv_sookshma_trisphuta(
    double lagna, double moon, double gulika, double sun);
double dhruv_avayoga_sphuta(double sun, double moon);
double dhruv_kunda(double lagna, double moon, double mars);

/* --- Special Lagnas --- */
const char *dhruv_special_lagna_name(uint32_t index);
double dhruv_bhava_lagna(double sun_lon, double ghatikas);
double dhruv_hora_lagna(double sun_lon, double ghatikas);
double dhruv_ghati_lagna(double sun_lon, double ghatikas);
double dhruv_vighati_lagna(double lagna_lon, double vighatikas);
double dhruv_varnada_lagna(double lagna_lon, double hora_lagna_lon);
double dhruv_sree_lagna(double moon_lon, double lagna_lon);
double dhruv_pranapada_lagna(double sun_lon, double ghatikas);
double dhruv_indu_lagna(double moon_lon, uint32_t lagna_lord, uint32_t moon_9th_lord);
DhruvStatus dhruv_special_lagnas_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvRiseSetConfig *riseset_config,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    DhruvSpecialLagnas *out);

/* --- Arudha Padas --- */
const char *dhruv_arudha_pada_name(uint32_t index);
double dhruv_arudha_pada(
    double bhava_cusp_lon,
    double lord_lon,
    uint8_t *out_rashi);
DhruvStatus dhruv_arudha_padas_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    DhruvArudhaResult *out);

/* --- Upagrahas --- */
const char *dhruv_upagraha_name(uint32_t index);
DhruvTimeUpagrahaConfig dhruv_time_upagraha_config_default(void);
DhruvStatus dhruv_sun_based_upagrahas(
    double sun_sid_lon,
    DhruvAllUpagrahas *out);
DhruvStatus dhruv_time_upagraha_jd(
    uint32_t upagraha_index,
    uint32_t weekday,
    uint8_t is_day,
    double sunrise_jd,
    double sunset_jd,
    double next_sunrise_jd,
    double *out_jd);
DhruvStatus dhruv_time_upagraha_jd_with_config(
    uint32_t upagraha_index,
    uint32_t weekday,
    uint8_t is_day,
    double sunrise_jd,
    double sunset_jd,
    double next_sunrise_jd,
    const DhruvTimeUpagrahaConfig *upagraha_config,
    double *out_jd);
DhruvStatus dhruv_time_upagraha_jd_utc(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvRiseSetConfig *riseset_config,
    uint32_t upagraha_index,
    double *out_jd);
DhruvStatus dhruv_time_upagraha_jd_utc_with_config(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvRiseSetConfig *riseset_config,
    const DhruvTimeUpagrahaConfig *upagraha_config,
    uint32_t upagraha_index,
    double *out_jd);
DhruvStatus dhruv_all_upagrahas_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    DhruvAllUpagrahas *out);
DhruvStatus dhruv_all_upagrahas_for_date_with_config(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    const DhruvTimeUpagrahaConfig *upagraha_config,
    DhruvAllUpagrahas *out);

/* --- Ashtakavarga --- */
DhruvStatus dhruv_calculate_ashtakavarga(
    const uint8_t *graha_rashis, uint8_t lagna_rashi,
    DhruvAshtakavargaResult *out);
DhruvStatus dhruv_calculate_bav(
    uint8_t graha_index,
    const uint8_t *graha_rashis, uint8_t lagna_rashi,
    DhruvBhinnaAshtakavarga *out);
DhruvStatus dhruv_calculate_all_bav(
    const uint8_t *graha_rashis, uint8_t lagna_rashi,
    DhruvBhinnaAshtakavarga *out);
DhruvStatus dhruv_calculate_sav(
    const DhruvBhinnaAshtakavarga *bavs,
    DhruvSarvaAshtakavarga *out);
DhruvStatus dhruv_trikona_sodhana(const uint8_t *totals, uint8_t *out);
DhruvStatus dhruv_ekadhipatya_sodhana(
    const uint8_t *totals,
    const uint8_t *graha_rashis, uint8_t lagna_rashi,
    uint8_t *out);
DhruvStatus dhruv_ashtakavarga_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    DhruvAshtakavargaResult *out);

/* --- Drishti --- */
DhruvStatus dhruv_graha_drishti(
    uint32_t graha_index,
    double source_lon, double target_lon,
    DhruvDrishtiEntry *out);
DhruvStatus dhruv_graha_drishti_matrix(
    const double *sidereal_lons,
    DhruvGrahaDrishtiMatrix *out);
DhruvStatus dhruv_drishti(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvBhavaConfig *bhava_config,
    const DhruvRiseSetConfig *riseset_config,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    const DhruvDrishtiConfig *config,
    DhruvDrishtiResult *out);

/* --- Ghatika / hora helpers --- */
DhruvStatus dhruv_ghatika_from_elapsed(
    double query_jd, double sunrise_jd, double next_sunrise_jd,
    int32_t *out);
DhruvStatus dhruv_ghatikas_since_sunrise(
    double query_jd, double sunrise_jd,
    double *out);
int32_t dhruv_hora_at(uint32_t vaar_index, uint32_t hora_index);

/* --- Graha positions --- */
DhruvStatus dhruv_graha_positions(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvBhavaConfig *bhava_config,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    const DhruvGrahaPositionsConfig *config,
    DhruvGrahaPositions *out);

/* Fixed-cadence sampling of graha positions over [from_utc, to_utc].
   One point per step_minutes (endpoints inclusive when on the grid).
   Rejects step_minutes == 0, reversed ranges, and grids over 10000 points.
   The returned handle must be freed with dhruv_graha_positions_series_free. */
typedef void *DhruvGrahaPositionsSeriesHandle;

typedef struct {
    DhruvUtcTime        utc;      /* epoch as Gregorian UTC */
    double              jd_utc;   /* epoch as JD UTC */
    DhruvGrahaPositions positions;
} DhruvGrahaPositionsPoint;

DhruvStatus dhruv_graha_positions_series(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *from_utc,
    const DhruvUtcTime *to_utc,
    uint32_t step_minutes,
    const DhruvGeoLocation *location,
    const DhruvBhavaConfig *bhava_config,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    const DhruvGrahaPositionsConfig *config,
    DhruvGrahaPositionsSeriesHandle *out);
DhruvStatus dhruv_graha_positions_series_count(
    DhruvGrahaPositionsSeriesHandle handle,
    uint32_t *out);
DhruvStatus dhruv_graha_positions_series_at(
    DhruvGrahaPositionsSeriesHandle handle,
    uint32_t idx,
    DhruvGrahaPositionsPoint *out);
void dhruv_graha_positions_series_free(DhruvGrahaPositionsSeriesHandle handle);

/* --- Core Bindus --- */
DhruvStatus dhruv_core_bindus(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvBhavaConfig *bhava_config,
    const DhruvRiseSetConfig *riseset_config,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    const DhruvBindusConfig *config,
    DhruvBindusResult *out);

/* --- Graha longitudes --- */
DhruvGrahaLongitudesConfig dhruv_graha_longitudes_config_default(void);
DhruvStatus dhruv_graha_longitudes(
    const DhruvEngineHandle *engine,
    double jd_tdb,
    const DhruvGrahaLongitudesConfig *config,
    DhruvGrahaLongitudes *out);
DhruvStatus dhruv_moving_osculating_apogees_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const uint8_t *graha_indices,
    uint8_t graha_count,
    const DhruvGrahaLongitudesConfig *config,
    DhruvMovingOsculatingApogees *out);

/* --- Nakshatra at --- */
DhruvStatus dhruv_nakshatra_at(
    const DhruvEngineHandle *engine,
    double jd_tdb,
    double moon_sidereal_deg,
    const DhruvSankrantiConfig *config,
    DhruvPanchangNakshatraInfo *out);

/* --- Amsha (divisional chart) --- */
/* Number of points in an amsha point family (0 for an unknown family). */
uint32_t dhruv_amsha_point_count(uint32_t family);
/* Display name / stable snake_case key of the point at (family, index).
 * Both return NUL-terminated UTF-8 valid for the process lifetime, or NULL for
 * an unknown family or out-of-range index. */
const char *dhruv_amsha_point_name(uint32_t family, uint32_t index);
const char *dhruv_amsha_point_key(uint32_t family, uint32_t index);
/* Sanskrit display name of the amsha with this D-number code (for example
 * "Navamsha" for 9). Returns NUL-terminated UTF-8 valid for the process
 * lifetime, or NULL for a code outside the 34 supported amshas. */
const char *dhruv_amsha_sanskrit_name(uint16_t amsha_code);
DhruvStatus dhruv_amsha_longitude(
    double sidereal_lon,
    uint16_t amsha_code,
    uint8_t variation_code,
    double *out);
DhruvStatus dhruv_amsha_rashi_info(
    double sidereal_lon,
    uint16_t amsha_code,
    uint8_t variation_code,
    DhruvRashiInfo *out);
DhruvStatus dhruv_amsha_longitudes(
    double sidereal_lon,
    const uint16_t *amsha_codes,
    const uint8_t *variation_codes,
    uint32_t count,
    double *out);
DhruvStatus dhruv_amsha_chart_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvBhavaConfig *bhava_config,
    const DhruvRiseSetConfig *riseset_config,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    uint16_t amsha_code,
    uint8_t variation_code,
    const DhruvAmshaChartScope *scope,
    DhruvAmshaChart *out);
DhruvStatus dhruv_amsha_variations(
    uint16_t amsha_code,
    DhruvAmshaVariationList *out);
DhruvStatus dhruv_amsha_variations_many(
    const uint16_t *amsha_codes,
    uint32_t count,
    DhruvAmshaVariationCatalogs *out);

/* --- Amsha series (fixed-cadence slim varga charts) ---
   Grid semantics match dhruv_graha_positions_series: one point per
   step_minutes starting at from_utc, endpoints inclusive when on the grid.
   Each point carries one chart per request, in request order (duplicates
   repeated); variation_codes may be NULL (all default). The varga lagna is
   always computed; graha entries are added when include_grahas is non-zero.
   Rejects step_minutes == 0, reversed ranges, empty or invalid request
   lists, and grids whose points * unique_requests exceed
   DHRUV_MAX_AMSHA_SERIES_CELLS. The returned handle must be freed with
   dhruv_amsha_series_free. */
typedef void *DhruvAmshaSeriesHandle;

DhruvStatus dhruv_amsha_series(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *from_utc,
    const DhruvUtcTime *to_utc,
    uint32_t step_minutes,
    const DhruvGeoLocation *location,
    const DhruvSankrantiConfig *sankranti_config,
    const uint16_t *amsha_codes,
    const uint8_t *variation_codes,
    uint32_t request_count,
    uint8_t include_grahas,
    DhruvAmshaSeriesHandle *out);
DhruvStatus dhruv_amsha_series_point_count(
    DhruvAmshaSeriesHandle handle,
    uint32_t *out);
DhruvStatus dhruv_amsha_series_chart_count(
    DhruvAmshaSeriesHandle handle,
    uint32_t *out);
DhruvStatus dhruv_amsha_series_point_at(
    DhruvAmshaSeriesHandle handle,
    uint32_t idx,
    DhruvUtcTime *out_utc,
    double *out_jd_utc);
DhruvStatus dhruv_amsha_series_chart_at(
    DhruvAmshaSeriesHandle handle,
    uint32_t point_idx,
    uint32_t chart_idx,
    DhruvAmshaSeriesChart *out);
void dhruv_amsha_series_free(DhruvAmshaSeriesHandle handle);

/* --- Amsha lagna events (exact varga-lagna transitions) ---
   Streams exact varga-lagna rashi segments overlapping [from_utc, to_utc],
   one entry per unique request (duplicates collapsed), in request order;
   variation_codes may be NULL (all default). max_segments caps the total
   segments across all amshas (0 selects the hard ceiling
   DHRUV_MAX_AMSHA_LAGNA_SEGMENTS). When truncated,
   dhruv_amsha_lagna_events_meta yields the resume point. The returned
   handle must be freed with dhruv_amsha_lagna_events_free. */
typedef void *DhruvAmshaLagnaEventsHandle;

DhruvStatus dhruv_amsha_lagna_events(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *from_utc,
    const DhruvUtcTime *to_utc,
    const DhruvGeoLocation *location,
    const DhruvSankrantiConfig *sankranti_config,
    const uint16_t *amsha_codes,
    const uint8_t *variation_codes,
    uint32_t request_count,
    uint32_t max_segments,
    DhruvAmshaLagnaEventsHandle *out);
DhruvStatus dhruv_amsha_lagna_events_entry_count(
    DhruvAmshaLagnaEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_amsha_lagna_events_entry_info(
    DhruvAmshaLagnaEventsHandle handle,
    uint32_t entry_idx,
    uint16_t *out_amsha_code,
    uint8_t *out_variation_code);
DhruvStatus dhruv_amsha_lagna_events_segment_count(
    DhruvAmshaLagnaEventsHandle handle,
    uint32_t entry_idx,
    uint32_t *out);
DhruvStatus dhruv_amsha_lagna_events_segment_at(
    DhruvAmshaLagnaEventsHandle handle,
    uint32_t entry_idx,
    uint32_t seg_idx,
    DhruvAmshaLagnaSegment *out);
DhruvStatus dhruv_amsha_lagna_events_meta(
    DhruvAmshaLagnaEventsHandle handle,
    uint8_t *out_truncated,
    uint8_t *out_next_from_valid,
    DhruvUtcTime *out_next_from_utc);
void dhruv_amsha_lagna_events_free(DhruvAmshaLagnaEventsHandle handle);

/* --- Charakaraka --- */
DhruvStatus dhruv_charakaraka_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    uint8_t scheme,
    DhruvCharakarakaResult *out);

/* --- Charakaraka ranking-change events ---
   Finds every chara-karaka ranking change in [from_utc, to_utc] for a
   scheme (DHRUV_CHARAKARAKA_SCHEME_*). Rankings are sidereal per
   sankranti_config (including node_mode — the same longitude computation
   as dhruv_charakaraka_for_date). max_events caps the emitted events
   (0 selects the hard ceiling DHRUV_MAX_CHARAKARAKA_EVENTS). When
   truncated, dhruv_charakaraka_events_meta yields the resume point; the
   seam event is re-found by the resumed sweep (deduplicate on the event
   time). The returned handle must be freed with
   dhruv_charakaraka_events_free. */
typedef void *DhruvCharakarakaEventsHandle;

DhruvStatus dhruv_charakaraka_events(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *from_utc,
    const DhruvUtcTime *to_utc,
    const DhruvSankrantiConfig *sankranti_config,
    uint8_t scheme,
    uint32_t max_events,
    DhruvCharakarakaEventsHandle *out);
DhruvStatus dhruv_charakaraka_events_count(
    DhruvCharakarakaEventsHandle handle,
    uint32_t *out);
DhruvStatus dhruv_charakaraka_events_at(
    DhruvCharakarakaEventsHandle handle,
    uint32_t idx,
    DhruvCharakarakaChangeEvent *out);
DhruvStatus dhruv_charakaraka_events_meta(
    DhruvCharakarakaEventsHandle handle,
    uint8_t *out_truncated,
    uint8_t *out_next_from_valid,
    DhruvUtcTime *out_next_from_utc);
void dhruv_charakaraka_events_free(DhruvCharakarakaEventsHandle handle);

/* First ranking change strictly after at_utc (*out_found = 1 when *out
   carries an event; 0 when none found before the coverage edge). */
DhruvStatus dhruv_next_charakaraka_event(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *at_utc,
    const DhruvSankrantiConfig *sankranti_config,
    uint8_t scheme,
    uint8_t *out_found,
    DhruvCharakarakaChangeEvent *out);
/* Last ranking change strictly before at_utc. */
DhruvStatus dhruv_prev_charakaraka_event(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *at_utc,
    const DhruvSankrantiConfig *sankranti_config,
    uint8_t scheme,
    uint8_t *out_found,
    DhruvCharakarakaChangeEvent *out);

/* --- Build identity ---
   Static NUL-terminated strings; do not free. dhruv_build_git_hash()
   returns "unknown" when the library was built outside a git checkout. */
const char *dhruv_library_version(void);
const char *dhruv_build_git_hash(void);

/* --- Shadbala --- */
DhruvStatus dhruv_shadbala_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvBhavaConfig *bhava_config,
    const DhruvRiseSetConfig *riseset_config,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    const DhruvAmshaSelectionConfig *amsha_selection,
    DhruvShadbalaResult *out);

/* --- Bhava Bala --- */
DhruvStatus dhruv_calculate_bhavabala(
    const DhruvBhavaBalaInputs *inputs,
    DhruvBhavaBalaResult *out);
DhruvStatus dhruv_bhavabala_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvBhavaConfig *bhava_config,
    const DhruvRiseSetConfig *riseset_config,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    DhruvBhavaBalaResult *out);

/* --- Vimsopaka --- */
DhruvStatus dhruv_vimsopaka_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    uint32_t node_dignity_policy,
    const DhruvAmshaSelectionConfig *amsha_selection,
    DhruvVimsopakaResult *out);

/* --- Bala Bundle --- */
DhruvStatus dhruv_balas_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvBhavaConfig *bhava_config,
    const DhruvRiseSetConfig *riseset_config,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    uint32_t node_dignity_policy,
    const DhruvAmshaSelectionConfig *amsha_selection,
    DhruvBalaBundleResult *out);

/* --- Avastha --- */
DhruvStatus dhruv_avastha_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvBhavaConfig *bhava_config,
    const DhruvRiseSetConfig *riseset_config,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    uint32_t node_dignity_policy,
    const DhruvAmshaSelectionConfig *amsha_selection,
    DhruvAllGrahaAvasthas *out);

/* --- Dasha --- */
DhruvDashaVariationConfig dhruv_dasha_variation_config_default(void);
DhruvDashaSelectionConfig dhruv_dasha_selection_config_default(void);
DhruvStatus dhruv_dasha_hierarchy_level_count(
    DhruvDashaHierarchyHandle handle, uint8_t *out);
DhruvStatus dhruv_dasha_hierarchy_period_count(
    DhruvDashaHierarchyHandle handle,
    uint8_t level, uint32_t *out);
DhruvStatus dhruv_dasha_hierarchy_period_at(
    DhruvDashaHierarchyHandle handle,
    uint8_t level, uint32_t idx,
    DhruvDashaPeriod *out);
void dhruv_dasha_hierarchy_free(DhruvDashaHierarchyHandle handle);
DhruvStatus dhruv_dasha_period_list_count(
    DhruvDashaPeriodListHandle handle, uint32_t *out);
DhruvStatus dhruv_dasha_period_list_at(
    DhruvDashaPeriodListHandle handle,
    uint32_t idx,
    DhruvDashaPeriod *out);
void dhruv_dasha_period_list_free(DhruvDashaPeriodListHandle handle);
DhruvStatus dhruv_dasha_hierarchy(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvDashaHierarchyRequest *request,
    DhruvDashaHierarchyHandle *out);
DhruvStatus dhruv_dasha_snapshot(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvDashaSnapshotRequest *request,
    DhruvDashaSnapshot *out);
DhruvStatus dhruv_dasha_level0(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvDashaLevel0Request *request,
    DhruvDashaPeriodListHandle *out);
DhruvStatus dhruv_dasha_level0_entity(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvDashaLevel0EntityRequest *request,
    uint8_t *out_found,
    DhruvDashaPeriod *out);
DhruvStatus dhruv_dasha_children(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvDashaChildrenRequest *request,
    DhruvDashaPeriodListHandle *out);
DhruvStatus dhruv_dasha_child_period(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvDashaChildPeriodRequest *request,
    uint8_t *out_found,
    DhruvDashaPeriod *out);
DhruvStatus dhruv_dasha_complete_level(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvDashaCompleteLevelRequest *request,
    const DhruvDashaPeriod *parent_periods,
    uint32_t parent_count,
    DhruvDashaPeriodListHandle *out);

/* --- Full Kundali --- */
DhruvFullKundaliConfig dhruv_full_kundali_config_default(void);
DhruvStatus dhruv_full_kundali_for_date(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvUtcTime *utc,
    const DhruvGeoLocation *location,
    const DhruvBhavaConfig *bhava_config,
    const DhruvRiseSetConfig *riseset_config,
    uint32_t ayanamsha_system,
    uint8_t use_nutation,
    const DhruvFullKundaliConfig *config,
    DhruvFullKundaliResult *out);
void dhruv_full_kundali_result_free(DhruvFullKundaliResult *result);
DhruvGocharEventsConfig dhruv_gochar_events_config_default(void);
DhruvStatus dhruv_gochar_events(
    const DhruvEngineHandle *engine,
    const DhruvEopHandle *eop,
    const DhruvGocharEventsRequest *request,
    DhruvGocharEventsHandle *out_handle);
void dhruv_gochar_events_free(DhruvGocharEventsHandle handle);
DhruvStatus dhruv_gochar_events_summary(
    DhruvGocharEventsHandle handle,
    DhruvGocharEventsSummary *out);
DhruvStatus dhruv_gochar_events_tajaka_count(
    DhruvGocharEventsHandle handle,
    uint8_t monthly_series,
    uint8_t before_side,
    uint32_t *out_count);
DhruvStatus dhruv_gochar_events_tajaka_at(
    DhruvGocharEventsHandle handle,
    uint8_t monthly_series,
    uint8_t before_side,
    uint32_t idx,
    DhruvTajakaReturnEventRow *out);
DhruvStatus dhruv_gochar_events_tajaka_chart_at(
    DhruvGocharEventsHandle handle,
    uint8_t monthly_series,
    uint8_t before_side,
    uint32_t idx,
    DhruvFullKundaliResult *out);
DhruvStatus dhruv_gochar_events_tithi_count(
    DhruvGocharEventsHandle handle,
    uint8_t monthly_series,
    uint8_t before_side,
    uint32_t *out_count);
DhruvStatus dhruv_gochar_events_tithi_at(
    DhruvGocharEventsHandle handle,
    uint8_t monthly_series,
    uint8_t before_side,
    uint32_t idx,
    DhruvTithiPraveshaEventRow *out);
DhruvStatus dhruv_gochar_events_tithi_chart_at(
    DhruvGocharEventsHandle handle,
    uint8_t monthly_series,
    uint8_t before_side,
    uint32_t idx,
    DhruvFullKundaliResult *out);
DhruvStatus dhruv_gochar_events_transit_count(
    DhruvGocharEventsHandle handle,
    uint32_t *out_count);
DhruvStatus dhruv_gochar_events_transit_at(
    DhruvGocharEventsHandle handle,
    uint32_t idx,
    DhruvTransitToNatalAspectEventRow *out);

/* --- Tara (fixed star) --- */
DhruvStatus dhruv_tara_catalog_load(
    const uint8_t *path_utf8, uint32_t path_len,
    DhruvTaraCatalogHandle **out_handle);
void dhruv_tara_catalog_free(DhruvTaraCatalogHandle *handle);
DhruvStatus dhruv_tara_compute_ex(
    const DhruvTaraCatalogHandle *handle,
    const DhruvTaraComputeRequest *request,
    DhruvTaraComputeResult *out);
DhruvStatus dhruv_tara_galactic_center_ecliptic(
    const DhruvTaraCatalogHandle *handle,
    double jd_tdb,
    DhruvSphericalCoords *out);
DhruvStatus dhruv_tara_propagate_position(
    double ra_deg,
    double dec_deg,
    double parallax_mas,
    double pm_ra_mas_yr,
    double pm_dec_mas_yr,
    double rv_km_s,
    double dt_years,
    DhruvEquatorialPosition *out);
DhruvStatus dhruv_tara_apply_aberration(
    const double *direction_3,
    const double *earth_vel_au_day_3,
    double *out_direction_3);
DhruvStatus dhruv_tara_apply_light_deflection(
    const double *direction_3,
    const double *sun_to_observer_3,
    double sun_observer_distance_au,
    double *out_direction_3);
DhruvStatus dhruv_tara_galactic_anticenter_icrs(double *out_direction_3);

#ifdef __cplusplus
}
#endif

#endif /* DHRUV_H */
"""

# ---------------------------------------------------------------------------
# Extract API version from #define in header
# ---------------------------------------------------------------------------
_m = _re.search(r"#define\s+DHRUV_API_VERSION\s+(\d+)", _RAW_HEADER)
if not _m:
    raise RuntimeError("DHRUV_API_VERSION not found in embedded C header")
EXPECTED_API_VERSION: int = int(_m.group(1))
DHRUV_API_VERSION: int = EXPECTED_API_VERSION


def _clean_for_cffi(raw: str) -> str:
    """Strip preprocessor directives for cffi ABI-mode cdef().

    cffi's ``ffi.cdef()`` does not support ``#define``, ``#include``,
    ``#ifdef``, or any other C preprocessor directives.  This function
    removes them while preserving all typedef/struct/function declarations.
    """
    numeric_defines = {
        name: value
        for name, value in _re.findall(r"#define\s+(DHRUV_[A-Z0-9_]+)\s+([0-9]+)\b", raw)
    }
    lines = raw.split("\n")
    out: list[str] = []
    in_continuation = False
    for line in lines:
        s = line.strip()
        if in_continuation:
            in_continuation = s.endswith("\\")
            continue
        if s.startswith("#"):
            in_continuation = s.endswith("\\")
            continue
        if s == 'extern "C" {':
            continue
        out.append(line)
    # Remove trailing closing brace from extern "C" block
    while out and out[-1].strip() in ("", "}"):
        out.pop()
    cleaned = "\n".join(out)
    for name, value in sorted(numeric_defines.items(), key=lambda item: len(item[0]), reverse=True):
        cleaned = cleaned.replace(name, value)
    return cleaned


CDEF: str = _clean_for_cffi(_RAW_HEADER)
#define DHRUV_DASHA_TIME_NONE   -1
#define DHRUV_DASHA_TIME_JD_UTC 0
#define DHRUV_DASHA_TIME_UTC    1
