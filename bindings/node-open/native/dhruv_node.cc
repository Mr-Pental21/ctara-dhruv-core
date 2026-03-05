#include <node_api.h>

#include <algorithm>
#include <cstring>
#include <string>
#include <vector>

#include "dhruv.h"

namespace {

constexpr int32_t STATUS_OK = 0;
constexpr int32_t STATUS_INVALID_INPUT = 13;

#define NAPI_RETURN_IF_FAILED(env, expr) \
    do {                                  \
        napi_status _s = (expr);          \
        if (_s != napi_ok) {              \
            return nullptr;                \
        }                                 \
    } while (0)

bool GetString(napi_env env, napi_value value, std::string* out) {
    size_t len = 0;
    if (napi_get_value_string_utf8(env, value, nullptr, 0, &len) != napi_ok) {
        return false;
    }
    std::string s(len, '\0');
    if (napi_get_value_string_utf8(env, value, s.data(), len + 1, &len) != napi_ok) {
        return false;
    }
    out->assign(s.data(), len);
    return true;
}

bool GetInt32(napi_env env, napi_value value, int32_t* out) {
    return napi_get_value_int32(env, value, out) == napi_ok;
}

bool GetUint32(napi_env env, napi_value value, uint32_t* out) {
    return napi_get_value_uint32(env, value, out) == napi_ok;
}

bool GetUint64(napi_env env, napi_value value, uint64_t* out) {
    bool lossless = false;
    return napi_get_value_bigint_uint64(env, value, out, &lossless) == napi_ok;
}

bool GetDouble(napi_env env, napi_value value, double* out) {
    return napi_get_value_double(env, value, out) == napi_ok;
}

bool GetBool(napi_env env, napi_value value, bool* out) {
    return napi_get_value_bool(env, value, out) == napi_ok;
}

napi_value MakeInt32(napi_env env, int32_t value) {
    napi_value out;
    napi_create_int32(env, value, &out);
    return out;
}

napi_value MakeUint32(napi_env env, uint32_t value) {
    napi_value out;
    napi_create_uint32(env, value, &out);
    return out;
}

napi_value MakeBool(napi_env env, bool value) {
    napi_value out;
    napi_get_boolean(env, value, &out);
    return out;
}

napi_value MakeDouble(napi_env env, double value) {
    napi_value out;
    napi_create_double(env, value, &out);
    return out;
}

napi_value MakeStatusResult(napi_env env, int32_t status) {
    napi_value obj;
    napi_create_object(env, &obj);
    napi_set_named_property(env, obj, "status", MakeInt32(env, status));
    return obj;
}

void SetNamed(napi_env env, napi_value obj, const char* name, napi_value value) {
    napi_set_named_property(env, obj, name, value);
}

bool GetNamedProperty(napi_env env, napi_value obj, const char* name, napi_value* out) {
    return napi_get_named_property(env, obj, name, out) == napi_ok;
}

bool GetOptionalNamedProperty(napi_env env, napi_value obj, const char* name, napi_value* out, bool* has) {
    bool present = false;
    if (napi_has_named_property(env, obj, name, &present) != napi_ok) {
        return false;
    }
    *has = present;
    if (!present) {
        return true;
    }
    return napi_get_named_property(env, obj, name, out) == napi_ok;
}

bool ReadUtcTime(napi_env env, napi_value obj, DhruvUtcTime* out) {
    napi_value v;
    if (!GetNamedProperty(env, obj, "year", &v) || !GetInt32(env, v, &out->year)) {
        return false;
    }
    if (!GetNamedProperty(env, obj, "month", &v) || !GetUint32(env, v, &out->month)) {
        return false;
    }
    if (!GetNamedProperty(env, obj, "day", &v) || !GetUint32(env, v, &out->day)) {
        return false;
    }
    if (!GetNamedProperty(env, obj, "hour", &v) || !GetUint32(env, v, &out->hour)) {
        return false;
    }
    if (!GetNamedProperty(env, obj, "minute", &v) || !GetUint32(env, v, &out->minute)) {
        return false;
    }
    if (!GetNamedProperty(env, obj, "second", &v) || !GetDouble(env, v, &out->second)) {
        return false;
    }
    return true;
}

napi_value WriteUtcTime(napi_env env, const DhruvUtcTime& utc) {
    napi_value obj;
    napi_create_object(env, &obj);
    SetNamed(env, obj, "year", MakeInt32(env, utc.year));
    SetNamed(env, obj, "month", MakeUint32(env, utc.month));
    SetNamed(env, obj, "day", MakeUint32(env, utc.day));
    SetNamed(env, obj, "hour", MakeUint32(env, utc.hour));
    SetNamed(env, obj, "minute", MakeUint32(env, utc.minute));
    SetNamed(env, obj, "second", MakeDouble(env, utc.second));
    return obj;
}

bool ReadGeoLocation(napi_env env, napi_value obj, DhruvGeoLocation* out) {
    napi_value v;
    if (!GetNamedProperty(env, obj, "latitudeDeg", &v) || !GetDouble(env, v, &out->latitude_deg)) {
        return false;
    }
    if (!GetNamedProperty(env, obj, "longitudeDeg", &v) || !GetDouble(env, v, &out->longitude_deg)) {
        return false;
    }
    if (!GetNamedProperty(env, obj, "altitudeM", &v) || !GetDouble(env, v, &out->altitude_m)) {
        return false;
    }
    return true;
}

bool ReadExternalPtr(napi_env env, napi_value value, void** out) {
    return napi_get_value_external(env, value, out) == napi_ok;
}

napi_value MakeExternalPtr(napi_env env, void* ptr) {
    napi_value out;
    napi_create_external(env, ptr, nullptr, nullptr, &out);
    return out;
}

napi_value WriteStateVector(napi_env env, const DhruvStateVector& vec) {
    napi_value obj;
    napi_create_object(env, &obj);

    napi_value pos;
    napi_create_array_with_length(env, 3, &pos);
    for (uint32_t i = 0; i < 3; ++i) {
        napi_set_element(env, pos, i, MakeDouble(env, vec.position_km[i]));
    }

    napi_value vel;
    napi_create_array_with_length(env, 3, &vel);
    for (uint32_t i = 0; i < 3; ++i) {
        napi_set_element(env, vel, i, MakeDouble(env, vec.velocity_km_s[i]));
    }

    SetNamed(env, obj, "positionKm", pos);
    SetNamed(env, obj, "velocityKmS", vel);
    return obj;
}

napi_value WriteSphericalState(napi_env env, const DhruvSphericalState& st) {
    napi_value obj;
    napi_create_object(env, &obj);
    SetNamed(env, obj, "lonDeg", MakeDouble(env, st.lon_deg));
    SetNamed(env, obj, "latDeg", MakeDouble(env, st.lat_deg));
    SetNamed(env, obj, "distanceKm", MakeDouble(env, st.distance_km));
    SetNamed(env, obj, "lonSpeed", MakeDouble(env, st.lon_speed));
    SetNamed(env, obj, "latSpeed", MakeDouble(env, st.lat_speed));
    SetNamed(env, obj, "distanceSpeed", MakeDouble(env, st.distance_speed));
    return obj;
}

napi_value WriteLunarPhaseEvent(napi_env env, const DhruvLunarPhaseEvent& ev) {
    napi_value obj;
    napi_create_object(env, &obj);
    SetNamed(env, obj, "utc", WriteUtcTime(env, ev.utc));
    SetNamed(env, obj, "phase", MakeInt32(env, ev.phase));
    SetNamed(env, obj, "moonLongitudeDeg", MakeDouble(env, ev.moon_longitude_deg));
    SetNamed(env, obj, "sunLongitudeDeg", MakeDouble(env, ev.sun_longitude_deg));
    return obj;
}

napi_value WriteTithiInfo(napi_env env, const DhruvTithiInfo& t) {
    napi_value obj;
    napi_create_object(env, &obj);
    SetNamed(env, obj, "tithiIndex", MakeInt32(env, t.tithi_index));
    SetNamed(env, obj, "paksha", MakeInt32(env, t.paksha));
    SetNamed(env, obj, "tithiInPaksha", MakeInt32(env, t.tithi_in_paksha));
    SetNamed(env, obj, "start", WriteUtcTime(env, t.start));
    SetNamed(env, obj, "end", WriteUtcTime(env, t.end));
    return obj;
}

napi_value WriteVaarInfo(napi_env env, const DhruvVaarInfo& v) {
    napi_value obj;
    napi_create_object(env, &obj);
    SetNamed(env, obj, "vaarIndex", MakeInt32(env, v.vaar_index));
    SetNamed(env, obj, "start", WriteUtcTime(env, v.start));
    SetNamed(env, obj, "end", WriteUtcTime(env, v.end));
    return obj;
}

napi_value ApiVersion(napi_env env, napi_callback_info info) {
    (void)info;
    return MakeUint32(env, dhruv_api_version());
}

napi_value ConfigClearActive(napi_env env, napi_callback_info info) {
    (void)info;
    return MakeInt32(env, dhruv_config_clear_active());
}

napi_value ConfigLoad(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2];
    NAPI_RETURN_IF_FAILED(env, napi_get_cb_info(env, info, &argc, args, nullptr, nullptr));

    if (argc < 2) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    bool is_null = false;
    napi_valuetype t;
    napi_typeof(env, args[0], &t);
    is_null = (t == napi_null || t == napi_undefined);

    uint32_t defaults_mode = 0;
    if (!GetUint32(env, args[1], &defaults_mode)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    const uint8_t* path_ptr = nullptr;
    std::string path;
    uint32_t path_len = 0;
    if (!is_null) {
        if (!GetString(env, args[0], &path)) {
            return MakeStatusResult(env, STATUS_INVALID_INPUT);
        }
        path_ptr = reinterpret_cast<const uint8_t*>(path.data());
        path_len = static_cast<uint32_t>(path.size());
    }

    DhruvConfigHandle* handle = nullptr;
    int32_t status = dhruv_config_load(path_ptr, path_len, &handle);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK && handle != nullptr) {
        SetNamed(env, out, "handle", MakeExternalPtr(env, handle));
    } else {
        napi_value nullv;
        napi_get_null(env, &nullv);
        SetNamed(env, out, "handle", nullv);
    }
    return out;
}

napi_value ConfigFree(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    NAPI_RETURN_IF_FAILED(env, napi_get_cb_info(env, info, &argc, args, nullptr, nullptr));
    if (argc < 1) {
        return MakeInt32(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeInt32(env, STATUS_INVALID_INPUT);
    }

    return MakeInt32(env, dhruv_config_free(static_cast<DhruvConfigHandle*>(ptr)));
}

napi_value EngineNew(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    NAPI_RETURN_IF_FAILED(env, napi_get_cb_info(env, info, &argc, args, nullptr, nullptr));
    if (argc < 1) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvEngineConfig cfg{};
    cfg.cache_capacity = 256;
    cfg.strict_validation = 1;

    napi_value spk_paths_val;
    if (!GetNamedProperty(env, args[0], "spkPaths", &spk_paths_val)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    bool is_array = false;
    napi_is_array(env, spk_paths_val, &is_array);
    if (!is_array) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    uint32_t spk_count = 0;
    napi_get_array_length(env, spk_paths_val, &spk_count);
    if (spk_count > DHRUV_MAX_SPK_PATHS) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    cfg.spk_path_count = spk_count;
    for (uint32_t i = 0; i < spk_count; ++i) {
        napi_value item;
        napi_get_element(env, spk_paths_val, i, &item);
        std::string s;
        if (!GetString(env, item, &s) || s.size() >= DHRUV_PATH_CAPACITY) {
            return MakeStatusResult(env, STATUS_INVALID_INPUT);
        }
        std::memcpy(cfg.spk_paths_utf8[i], s.data(), s.size());
        cfg.spk_paths_utf8[i][s.size()] = '\0';
    }

    napi_value lsk_val;
    bool has_lsk = false;
    if (!GetOptionalNamedProperty(env, args[0], "lskPath", &lsk_val, &has_lsk)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }
    if (has_lsk) {
        napi_valuetype tt;
        napi_typeof(env, lsk_val, &tt);
        if (tt != napi_null && tt != napi_undefined) {
            std::string s;
            if (!GetString(env, lsk_val, &s) || s.size() >= DHRUV_PATH_CAPACITY) {
                return MakeStatusResult(env, STATUS_INVALID_INPUT);
            }
            std::memcpy(cfg.lsk_path_utf8, s.data(), s.size());
            cfg.lsk_path_utf8[s.size()] = '\0';
        }
    }

    napi_value cache_val;
    bool has_cache = false;
    if (GetOptionalNamedProperty(env, args[0], "cacheCapacity", &cache_val, &has_cache) && has_cache) {
        uint64_t cap = 0;
        if (GetUint64(env, cache_val, &cap)) {
            cfg.cache_capacity = cap;
        } else {
            double dcap = 0;
            if (!GetDouble(env, cache_val, &dcap)) {
                return MakeStatusResult(env, STATUS_INVALID_INPUT);
            }
            cfg.cache_capacity = static_cast<uint64_t>(std::max(0.0, dcap));
        }
    }

    napi_value strict_val;
    bool has_strict = false;
    if (GetOptionalNamedProperty(env, args[0], "strictValidation", &strict_val, &has_strict) && has_strict) {
        bool strict = true;
        if (!GetBool(env, strict_val, &strict)) {
            return MakeStatusResult(env, STATUS_INVALID_INPUT);
        }
        cfg.strict_validation = strict ? 1 : 0;
    }

    DhruvEngineHandle* engine = nullptr;
    int32_t status = dhruv_engine_new(&cfg, &engine);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK && engine != nullptr) {
        SetNamed(env, out, "handle", MakeExternalPtr(env, engine));
    } else {
        napi_value nullv;
        napi_get_null(env, &nullv);
        SetNamed(env, out, "handle", nullv);
    }
    return out;
}

napi_value EngineFree(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 1) {
        return MakeInt32(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeInt32(env, STATUS_INVALID_INPUT);
    }

    return MakeInt32(env, dhruv_engine_free(static_cast<DhruvEngineHandle*>(ptr)));
}

napi_value LskLoad(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 1) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    std::string path;
    if (!GetString(env, args[0], &path)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvLskHandle* handle = nullptr;
    int32_t status = dhruv_lsk_load(path.c_str(), &handle);
    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK && handle != nullptr) {
        SetNamed(env, out, "handle", MakeExternalPtr(env, handle));
    } else {
        napi_value nullv;
        napi_get_null(env, &nullv);
        SetNamed(env, out, "handle", nullv);
    }
    return out;
}

napi_value LskFree(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 1) {
        return MakeInt32(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeInt32(env, STATUS_INVALID_INPUT);
    }

    return MakeInt32(env, dhruv_lsk_free(static_cast<DhruvLskHandle*>(ptr)));
}

napi_value EopLoad(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 1) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    std::string path;
    if (!GetString(env, args[0], &path)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvEopHandle* handle = nullptr;
    int32_t status = dhruv_eop_load(path.c_str(), &handle);
    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK && handle != nullptr) {
        SetNamed(env, out, "handle", MakeExternalPtr(env, handle));
    } else {
        napi_value nullv;
        napi_get_null(env, &nullv);
        SetNamed(env, out, "handle", nullv);
    }
    return out;
}

napi_value EopFree(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 1) {
        return MakeInt32(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeInt32(env, STATUS_INVALID_INPUT);
    }

    return MakeInt32(env, dhruv_eop_free(static_cast<DhruvEopHandle*>(ptr)));
}

napi_value EngineQuery(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 2) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvQuery q{};
    napi_value v;
    if (!GetNamedProperty(env, args[1], "target", &v) || !GetInt32(env, v, &q.target)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    if (!GetNamedProperty(env, args[1], "observer", &v) || !GetInt32(env, v, &q.observer)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    if (!GetNamedProperty(env, args[1], "frame", &v) || !GetInt32(env, v, &q.frame)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    if (!GetNamedProperty(env, args[1], "epochTdbJd", &v) || !GetDouble(env, v, &q.epoch_tdb_jd)) return MakeStatusResult(env, STATUS_INVALID_INPUT);

    DhruvStateVector out_vec{};
    int32_t status = dhruv_engine_query(static_cast<const DhruvEngineHandle*>(ptr), &q, &out_vec);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        SetNamed(env, out, "state", WriteStateVector(env, out_vec));
    }
    return out;
}

napi_value QueryUtcSpherical(napi_env env, napi_callback_info info) {
    size_t argc = 5;
    napi_value args[5];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 5) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    int32_t target = 0;
    int32_t observer = 0;
    int32_t frame = 0;
    if (!GetInt32(env, args[1], &target) || !GetInt32(env, args[2], &observer) || !GetInt32(env, args[3], &frame)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvUtcTime utc{};
    if (!ReadUtcTime(env, args[4], &utc)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvSphericalState st{};
    int32_t status = dhruv_query_utc(
        static_cast<const DhruvEngineHandle*>(ptr),
        target,
        observer,
        frame,
        &utc,
        &st);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        SetNamed(env, out, "state", WriteSphericalState(env, st));
    }
    return out;
}

napi_value UtcToTdbJd(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 2) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvUtcTime utc{};
    if (!ReadUtcTime(env, args[1], &utc)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    double jd = 0.0;
    int32_t status = dhruv_utc_to_tdb_jd(
        static_cast<const DhruvLskHandle*>(ptr),
        utc.year,
        utc.month,
        utc.day,
        utc.hour,
        utc.minute,
        utc.second,
        &jd);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        SetNamed(env, out, "jdTdb", MakeDouble(env, jd));
    }
    return out;
}

napi_value JdTdbToUtc(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 2) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    double jd = 0.0;
    if (!GetDouble(env, args[1], &jd)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvUtcTime utc{};
    int32_t status = dhruv_jd_tdb_to_utc(static_cast<const DhruvLskHandle*>(ptr), jd, &utc);
    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        SetNamed(env, out, "utc", WriteUtcTime(env, utc));
    }
    return out;
}

napi_value NutationIau2000b(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 1) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    double jd = 0.0;
    if (!GetDouble(env, args[0], &jd)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    double dpsi = 0.0;
    double deps = 0.0;
    int32_t status = dhruv_nutation_iau2000b(jd, &dpsi, &deps);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        SetNamed(env, out, "dpsi", MakeDouble(env, dpsi));
        SetNamed(env, out, "deps", MakeDouble(env, deps));
    }
    return out;
}

napi_value RiseSetConfigDefault(napi_env env, napi_callback_info info) {
    (void)info;
    DhruvRiseSetConfig cfg = dhruv_riseset_config_default();
    napi_value out;
    napi_create_object(env, &out);
    SetNamed(env, out, "useRefraction", MakeBool(env, cfg.use_refraction != 0));
    SetNamed(env, out, "sunLimb", MakeInt32(env, cfg.sun_limb));
    SetNamed(env, out, "altitudeCorrection", MakeBool(env, cfg.altitude_correction != 0));
    return out;
}

napi_value BhavaConfigDefault(napi_env env, napi_callback_info info) {
    (void)info;
    DhruvBhavaConfig cfg = dhruv_bhava_config_default();
    napi_value out;
    napi_create_object(env, &out);
    SetNamed(env, out, "system", MakeInt32(env, cfg.system));
    SetNamed(env, out, "startingPoint", MakeInt32(env, cfg.starting_point));
    SetNamed(env, out, "customStartDeg", MakeDouble(env, cfg.custom_start_deg));
    SetNamed(env, out, "referenceMode", MakeInt32(env, cfg.reference_mode));
    return out;
}

napi_value SankrantiConfigDefault(napi_env env, napi_callback_info info) {
    (void)info;
    DhruvSankrantiConfig cfg = dhruv_sankranti_config_default();
    napi_value out;
    napi_create_object(env, &out);
    SetNamed(env, out, "ayanamshaSystem", MakeInt32(env, cfg.ayanamsha_system));
    SetNamed(env, out, "useNutation", MakeBool(env, cfg.use_nutation != 0));
    SetNamed(env, out, "referencePlane", MakeInt32(env, cfg.reference_plane));
    SetNamed(env, out, "stepSizeDays", MakeDouble(env, cfg.step_size_days));
    SetNamed(env, out, "maxIterations", MakeUint32(env, cfg.max_iterations));
    SetNamed(env, out, "convergenceDays", MakeDouble(env, cfg.convergence_days));
    return out;
}

napi_value LunarPhaseSearch(napi_env env, napi_callback_info info) {
    size_t argc = 3;
    napi_value args[3];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 3) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvLunarPhaseSearchRequest req{};
    napi_value v;
    if (!GetNamedProperty(env, args[1], "phaseKind", &v) || !GetInt32(env, v, &req.phase_kind)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    if (!GetNamedProperty(env, args[1], "queryMode", &v) || !GetInt32(env, v, &req.query_mode)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    if (!GetNamedProperty(env, args[1], "atJdTdb", &v) || !GetDouble(env, v, &req.at_jd_tdb)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    if (!GetNamedProperty(env, args[1], "startJdTdb", &v) || !GetDouble(env, v, &req.start_jd_tdb)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    if (!GetNamedProperty(env, args[1], "endJdTdb", &v) || !GetDouble(env, v, &req.end_jd_tdb)) return MakeStatusResult(env, STATUS_INVALID_INPUT);

    uint32_t capacity = 0;
    if (!GetUint32(env, args[2], &capacity)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvLunarPhaseEvent out_event{};
    uint8_t found = 0;
    uint32_t out_count = 0;
    std::vector<DhruvLunarPhaseEvent> events(capacity > 0 ? capacity : 1);

    int32_t status = dhruv_lunar_phase_search_ex(
        static_cast<const DhruvEngineHandle*>(ptr),
        &req,
        &out_event,
        &found,
        capacity > 0 ? events.data() : nullptr,
        capacity,
        &out_count);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        SetNamed(env, out, "found", MakeBool(env, found != 0));
        SetNamed(env, out, "count", MakeUint32(env, out_count));
        if (found != 0) {
            SetNamed(env, out, "event", WriteLunarPhaseEvent(env, out_event));
        }

        napi_value arr;
        napi_create_array_with_length(env, out_count, &arr);
        for (uint32_t i = 0; i < out_count; ++i) {
            napi_set_element(env, arr, i, WriteLunarPhaseEvent(env, events[i]));
        }
        SetNamed(env, out, "events", arr);
    }
    return out;
}

napi_value TithiForDate(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 2) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvUtcTime utc{};
    if (!ReadUtcTime(env, args[1], &utc)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvTithiInfo tithi{};
    int32_t status = dhruv_tithi_for_date(static_cast<const DhruvEngineHandle*>(ptr), &utc, &tithi);
    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        SetNamed(env, out, "tithi", WriteTithiInfo(env, tithi));
    }
    return out;
}

napi_value VaarForDate(napi_env env, napi_callback_info info) {
    size_t argc = 4;
    napi_value args[4];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 4) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* e_ptr = nullptr;
    void* ep_ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &e_ptr) || !ReadExternalPtr(env, args[1], &ep_ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvUtcTime utc{};
    if (!ReadUtcTime(env, args[2], &utc)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvGeoLocation loc{};
    if (!ReadGeoLocation(env, args[3], &loc)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvRiseSetConfig rise_cfg = dhruv_riseset_config_default();
    DhruvVaarInfo info_out{};
    int32_t status = dhruv_vaar_for_date(
        static_cast<const DhruvEngineHandle*>(e_ptr),
        static_cast<const DhruvEopHandle*>(ep_ptr),
        &utc,
        &loc,
        &rise_cfg,
        &info_out);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        SetNamed(env, out, "vaar", WriteVaarInfo(env, info_out));
    }
    return out;
}

napi_value GrahaSiderealLongitudes(napi_env env, napi_callback_info info) {
    size_t argc = 4;
    napi_value args[4];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 4) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    double jd = 0.0;
    uint32_t ayanamsha = 0;
    bool use_nutation = false;
    if (!GetDouble(env, args[1], &jd) || !GetUint32(env, args[2], &ayanamsha) || !GetBool(env, args[3], &use_nutation)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvGrahaLongitudes out_lons{};
    int32_t status = dhruv_graha_sidereal_longitudes(
        static_cast<const DhruvEngineHandle*>(ptr),
        jd,
        ayanamsha,
        use_nutation ? 1 : 0,
        &out_lons);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        napi_value arr;
        napi_create_array_with_length(env, DHRUV_GRAHA_COUNT, &arr);
        for (uint32_t i = 0; i < DHRUV_GRAHA_COUNT; ++i) {
            napi_set_element(env, arr, i, MakeDouble(env, out_lons.longitudes[i]));
        }
        SetNamed(env, out, "longitudes", arr);
    }
    return out;
}

napi_value ShadbalaForDate(napi_env env, napi_callback_info info) {
    size_t argc = 6;
    napi_value args[6];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 6) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* e_ptr = nullptr;
    void* ep_ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &e_ptr) || !ReadExternalPtr(env, args[1], &ep_ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvUtcTime utc{};
    DhruvGeoLocation loc{};
    if (!ReadUtcTime(env, args[2], &utc) || !ReadGeoLocation(env, args[3], &loc)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    uint32_t ayanamsha = 0;
    bool use_nutation = false;
    if (!GetUint32(env, args[4], &ayanamsha) || !GetBool(env, args[5], &use_nutation)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvBhavaConfig bhava_cfg = dhruv_bhava_config_default();
    DhruvRiseSetConfig rise_cfg = dhruv_riseset_config_default();
    DhruvShadbalaResult out_result{};

    int32_t status = dhruv_shadbala_for_date(
        static_cast<const DhruvEngineHandle*>(e_ptr),
        static_cast<const DhruvEopHandle*>(ep_ptr),
        &utc,
        &loc,
        &bhava_cfg,
        &rise_cfg,
        ayanamsha,
        use_nutation ? 1 : 0,
        &out_result);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        napi_value arr;
        napi_create_array_with_length(env, 7, &arr);
        for (uint32_t i = 0; i < 7; ++i) {
            napi_set_element(env, arr, i, MakeDouble(env, out_result.entries[i].total_rupas));
        }
        SetNamed(env, out, "totalRupas", arr);
    }

    return out;
}

napi_value FullKundaliSummaryForDate(napi_env env, napi_callback_info info) {
    size_t argc = 6;
    napi_value args[6];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 6) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* e_ptr = nullptr;
    void* ep_ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &e_ptr) || !ReadExternalPtr(env, args[1], &ep_ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvUtcTime utc{};
    DhruvGeoLocation loc{};
    if (!ReadUtcTime(env, args[2], &utc) || !ReadGeoLocation(env, args[3], &loc)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    uint32_t ayanamsha = 0;
    bool use_nutation = false;
    if (!GetUint32(env, args[4], &ayanamsha) || !GetBool(env, args[5], &use_nutation)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvBhavaConfig bhava_cfg = dhruv_bhava_config_default();
    DhruvRiseSetConfig rise_cfg = dhruv_riseset_config_default();
    DhruvFullKundaliConfig full_cfg = dhruv_full_kundali_config_default();
    DhruvFullKundaliResult out_result{};

    int32_t status = dhruv_full_kundali_for_date(
        static_cast<const DhruvEngineHandle*>(e_ptr),
        static_cast<const DhruvEopHandle*>(ep_ptr),
        &utc,
        &loc,
        &bhava_cfg,
        &rise_cfg,
        ayanamsha,
        use_nutation ? 1 : 0,
        &full_cfg,
        &out_result);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        SetNamed(env, out, "ayanamshaDeg", MakeDouble(env, out_result.ayanamsha_deg));
        SetNamed(env, out, "grahaPositionsValid", MakeBool(env, out_result.graha_positions_valid != 0));
        SetNamed(env, out, "panchangValid", MakeBool(env, out_result.panchang_valid != 0));
        SetNamed(env, out, "dashaSnapshotCount", MakeUint32(env, out_result.dasha_snapshot_count));
        dhruv_full_kundali_result_free(&out_result);
    }

    return out;
}

napi_value DashaSnapshotUtc(napi_env env, napi_callback_info info) {
    size_t argc = 9;
    napi_value args[9];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 9) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* e_ptr = nullptr;
    void* ep_ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &e_ptr) || !ReadExternalPtr(env, args[1], &ep_ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvUtcTime birth{};
    DhruvUtcTime query{};
    DhruvGeoLocation loc{};
    if (!ReadUtcTime(env, args[2], &birth) || !ReadUtcTime(env, args[3], &query) || !ReadGeoLocation(env, args[4], &loc)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    uint32_t ayanamsha = 0;
    bool use_nutation = false;
    uint32_t system = 0;
    uint32_t max_level = 0;
    if (!GetUint32(env, args[5], &ayanamsha) || !GetBool(env, args[6], &use_nutation) || !GetUint32(env, args[7], &system) || !GetUint32(env, args[8], &max_level)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvBhavaConfig bhava_cfg = dhruv_bhava_config_default();
    DhruvRiseSetConfig rise_cfg = dhruv_riseset_config_default();
    DhruvDashaSnapshot snapshot{};

    int32_t status = dhruv_dasha_snapshot_utc(
        static_cast<const DhruvEngineHandle*>(e_ptr),
        static_cast<const DhruvEopHandle*>(ep_ptr),
        &birth,
        &query,
        &loc,
        &bhava_cfg,
        &rise_cfg,
        ayanamsha,
        use_nutation ? 1 : 0,
        static_cast<uint8_t>(system),
        static_cast<uint8_t>(max_level),
        &snapshot);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        napi_value snap;
        napi_create_object(env, &snap);
        SetNamed(env, snap, "system", MakeUint32(env, snapshot.system));
        SetNamed(env, snap, "queryJd", MakeDouble(env, snapshot.query_jd));
        SetNamed(env, snap, "count", MakeUint32(env, snapshot.count));

        napi_value periods;
        napi_create_array_with_length(env, snapshot.count, &periods);
        for (uint32_t i = 0; i < snapshot.count; ++i) {
            const DhruvDashaPeriod& p = snapshot.periods[i];
            napi_value po;
            napi_create_object(env, &po);
            SetNamed(env, po, "entityType", MakeUint32(env, p.entity_type));
            SetNamed(env, po, "entityIndex", MakeUint32(env, p.entity_index));
            SetNamed(env, po, "startJd", MakeDouble(env, p.start_jd));
            SetNamed(env, po, "endJd", MakeDouble(env, p.end_jd));
            SetNamed(env, po, "level", MakeUint32(env, p.level));
            SetNamed(env, po, "order", MakeUint32(env, p.order));
            SetNamed(env, po, "parentIdx", MakeUint32(env, p.parent_idx));
            napi_set_element(env, periods, i, po);
        }
        SetNamed(env, snap, "periods", periods);
        SetNamed(env, out, "snapshot", snap);
    }

    return out;
}

napi_value TaraCatalogLoad(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 1) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    std::string path;
    if (!GetString(env, args[0], &path)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvTaraCatalogHandle* handle = nullptr;
    int32_t status = dhruv_tara_catalog_load(reinterpret_cast<const uint8_t*>(path.data()), static_cast<uint32_t>(path.size()), &handle);

    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK && handle != nullptr) {
        SetNamed(env, out, "handle", MakeExternalPtr(env, handle));
    } else {
        napi_value nullv;
        napi_get_null(env, &nullv);
        SetNamed(env, out, "handle", nullv);
    }
    return out;
}

napi_value TaraCatalogFree(napi_env env, napi_callback_info info) {
    size_t argc = 1;
    napi_value args[1];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 1) {
        napi_value undef;
        napi_get_undefined(env, &undef);
        return undef;
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        napi_value undef;
        napi_get_undefined(env, &undef);
        return undef;
    }

    dhruv_tara_catalog_free(static_cast<DhruvTaraCatalogHandle*>(ptr));
    napi_value undef;
    napi_get_undefined(env, &undef);
    return undef;
}

napi_value TaraGalacticCenterEcliptic(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 2) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    double jd = 0.0;
    if (!GetDouble(env, args[1], &jd)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvSphericalCoords coords{};
    int32_t status = dhruv_tara_galactic_center_ecliptic(static_cast<const DhruvTaraCatalogHandle*>(ptr), jd, &coords);
    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        napi_value c;
        napi_create_object(env, &c);
        SetNamed(env, c, "lonDeg", MakeDouble(env, coords.lon_deg));
        SetNamed(env, c, "latDeg", MakeDouble(env, coords.lat_deg));
        SetNamed(env, c, "distanceKm", MakeDouble(env, coords.distance_km));
        SetNamed(env, out, "coords", c);
    }
    return out;
}

napi_value TaraComputeEx(napi_env env, napi_callback_info info) {
    size_t argc = 2;
    napi_value args[2];
    napi_get_cb_info(env, info, &argc, args, nullptr, nullptr);
    if (argc < 2) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    void* ptr = nullptr;
    if (!ReadExternalPtr(env, args[0], &ptr)) {
        return MakeStatusResult(env, STATUS_INVALID_INPUT);
    }

    DhruvTaraComputeRequest req{};
    napi_value v;
    if (!GetNamedProperty(env, args[1], "taraId", &v) || !GetInt32(env, v, &req.tara_id)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    if (!GetNamedProperty(env, args[1], "outputKind", &v) || !GetInt32(env, v, &req.output_kind)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    if (!GetNamedProperty(env, args[1], "jdTdb", &v) || !GetDouble(env, v, &req.jd_tdb)) return MakeStatusResult(env, STATUS_INVALID_INPUT);

    bool has_ayan = false;
    if (!GetOptionalNamedProperty(env, args[1], "ayanamshaDeg", &v, &has_ayan)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    req.ayanamsha_deg = 0.0;
    if (has_ayan && !GetDouble(env, v, &req.ayanamsha_deg)) return MakeStatusResult(env, STATUS_INVALID_INPUT);

    req.config.accuracy = 0;
    req.config.apply_parallax = 1;

    bool has_cfg = false;
    napi_value cfg_obj;
    if (!GetOptionalNamedProperty(env, args[1], "config", &cfg_obj, &has_cfg)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    if (has_cfg) {
        bool has_acc = false;
        if (!GetOptionalNamedProperty(env, cfg_obj, "accuracy", &v, &has_acc)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
        if (has_acc && !GetInt32(env, v, &req.config.accuracy)) return MakeStatusResult(env, STATUS_INVALID_INPUT);

        bool has_par = false;
        if (!GetOptionalNamedProperty(env, cfg_obj, "applyParallax", &v, &has_par)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
        if (has_par) {
            bool b = true;
            if (!GetBool(env, v, &b)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
            req.config.apply_parallax = b ? 1 : 0;
        }
    }

    req.earth_state_valid = 0;
    bool has_earth = false;
    napi_value earth_obj;
    if (!GetOptionalNamedProperty(env, args[1], "earthState", &earth_obj, &has_earth)) return MakeStatusResult(env, STATUS_INVALID_INPUT);
    if (has_earth) {
        req.earth_state_valid = 1;
        napi_value pos, vel;
        bool pos_ok = GetNamedProperty(env, earth_obj, "positionAu", &pos);
        bool vel_ok = GetNamedProperty(env, earth_obj, "velocityAuDay", &vel);
        if (!pos_ok || !vel_ok) return MakeStatusResult(env, STATUS_INVALID_INPUT);

        bool is_arr = false;
        napi_is_array(env, pos, &is_arr);
        if (!is_arr) return MakeStatusResult(env, STATUS_INVALID_INPUT);
        napi_is_array(env, vel, &is_arr);
        if (!is_arr) return MakeStatusResult(env, STATUS_INVALID_INPUT);

        for (uint32_t i = 0; i < 3; ++i) {
            napi_value e;
            napi_get_element(env, pos, i, &e);
            if (!GetDouble(env, e, &req.earth_state.position_au[i])) return MakeStatusResult(env, STATUS_INVALID_INPUT);
            napi_get_element(env, vel, i, &e);
            if (!GetDouble(env, e, &req.earth_state.velocity_au_day[i])) return MakeStatusResult(env, STATUS_INVALID_INPUT);
        }
    }

    DhruvTaraComputeResult out_val{};
    int32_t status = dhruv_tara_compute_ex(static_cast<const DhruvTaraCatalogHandle*>(ptr), &req, &out_val);
    napi_value out = MakeStatusResult(env, status);
    if (status == STATUS_OK) {
        napi_value res;
        napi_create_object(env, &res);
        SetNamed(env, res, "outputKind", MakeInt32(env, out_val.output_kind));

        napi_value eq;
        napi_create_object(env, &eq);
        SetNamed(env, eq, "raDeg", MakeDouble(env, out_val.equatorial.ra_deg));
        SetNamed(env, eq, "decDeg", MakeDouble(env, out_val.equatorial.dec_deg));
        SetNamed(env, eq, "distanceAu", MakeDouble(env, out_val.equatorial.distance_au));
        SetNamed(env, res, "equatorial", eq);

        napi_value ecl;
        napi_create_object(env, &ecl);
        SetNamed(env, ecl, "lonDeg", MakeDouble(env, out_val.ecliptic.lon_deg));
        SetNamed(env, ecl, "latDeg", MakeDouble(env, out_val.ecliptic.lat_deg));
        SetNamed(env, ecl, "distanceKm", MakeDouble(env, out_val.ecliptic.distance_km));
        SetNamed(env, res, "ecliptic", ecl);

        SetNamed(env, res, "siderealLongitudeDeg", MakeDouble(env, out_val.sidereal_longitude_deg));
        SetNamed(env, out, "result", res);
    }

    return out;
}

napi_value Init(napi_env env, napi_value exports) {
    napi_property_descriptor props[] = {
        {"apiVersion", nullptr, ApiVersion, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"configLoad", nullptr, ConfigLoad, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"configFree", nullptr, ConfigFree, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"configClearActive", nullptr, ConfigClearActive, nullptr, nullptr, nullptr, napi_default, nullptr},

        {"engineNew", nullptr, EngineNew, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"engineFree", nullptr, EngineFree, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"engineQuery", nullptr, EngineQuery, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"queryUtcSpherical", nullptr, QueryUtcSpherical, nullptr, nullptr, nullptr, napi_default, nullptr},

        {"lskLoad", nullptr, LskLoad, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"lskFree", nullptr, LskFree, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"eopLoad", nullptr, EopLoad, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"eopFree", nullptr, EopFree, nullptr, nullptr, nullptr, napi_default, nullptr},

        {"utcToTdbJd", nullptr, UtcToTdbJd, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"jdTdbToUtc", nullptr, JdTdbToUtc, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"nutationIau2000b", nullptr, NutationIau2000b, nullptr, nullptr, nullptr, napi_default, nullptr},

        {"riseSetConfigDefault", nullptr, RiseSetConfigDefault, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"bhavaConfigDefault", nullptr, BhavaConfigDefault, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"sankrantiConfigDefault", nullptr, SankrantiConfigDefault, nullptr, nullptr, nullptr, napi_default, nullptr},

        {"lunarPhaseSearch", nullptr, LunarPhaseSearch, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"tithiForDate", nullptr, TithiForDate, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"vaarForDate", nullptr, VaarForDate, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"grahaSiderealLongitudes", nullptr, GrahaSiderealLongitudes, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"shadbalaForDate", nullptr, ShadbalaForDate, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"fullKundaliSummaryForDate", nullptr, FullKundaliSummaryForDate, nullptr, nullptr, nullptr, napi_default, nullptr},

        {"dashaSnapshotUtc", nullptr, DashaSnapshotUtc, nullptr, nullptr, nullptr, napi_default, nullptr},

        {"taraCatalogLoad", nullptr, TaraCatalogLoad, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"taraCatalogFree", nullptr, TaraCatalogFree, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"taraGalacticCenterEcliptic", nullptr, TaraGalacticCenterEcliptic, nullptr, nullptr, nullptr, napi_default, nullptr},
        {"taraComputeEx", nullptr, TaraComputeEx, nullptr, nullptr, nullptr, napi_default, nullptr},
    };

    napi_define_properties(env, exports, sizeof(props) / sizeof(props[0]), props);
    return exports;
}

}  // namespace

NAPI_MODULE(NODE_GYP_MODULE_NAME, Init)
