'use strict';

const { addon } = require('./native');
const { checkStatus } = require('./errors');

function elongationAt(engine, jdTdb) {
  const r = addon.elongationAt(engine._handle, jdTdb);
  checkStatus('elongation_at', r.status);
  return r.value;
}

function siderealSumAt(engine, jdTdb, config) {
  const r = addon.siderealSumAt(engine._handle, jdTdb, config);
  checkStatus('sidereal_sum_at', r.status);
  return r.value;
}

function vedicDaySunrises(engine, eop, utc, location, config) {
  const r = addon.vedicDaySunrises(engine._handle, eop._handle, utc, location, config);
  checkStatus('vedic_day_sunrises', r.status);
  return { sunriseJd: r.sunriseJd, nextSunriseJd: r.nextSunriseJd };
}

function bodyEclipticLonLat(engine, bodyCode, jdTdb) {
  const r = addon.bodyEclipticLonLat(engine._handle, bodyCode, jdTdb);
  checkStatus('body_ecliptic_lon_lat', r.status);
  return { lonDeg: r.lonDeg, latDeg: r.latDeg };
}

function tithiAt(engine, jdTdb, sunriseJd) {
  const r = addon.tithiAt(engine._handle, jdTdb, sunriseJd);
  checkStatus('tithi_at', r.status);
  return r.tithi;
}

function karanaAt(engine, jdTdb, sunriseJd) {
  const r = addon.karanaAt(engine._handle, jdTdb, sunriseJd);
  checkStatus('karana_at', r.status);
  return r.karana;
}

function yogaAt(engine, jdTdb, sunriseJd, config) {
  const r = addon.yogaAt(engine._handle, jdTdb, sunriseJd, config);
  checkStatus('yoga_at', r.status);
  return r.yoga;
}

function vaarFromSunrises(lsk, sunriseJd, nextSunriseJd) {
  const r = addon.vaarFromSunrises(lsk._handle, sunriseJd, nextSunriseJd);
  checkStatus('vaar_from_sunrises', r.status);
  return r.vaar;
}

function horaFromSunrises(lsk, queryJd, sunriseJd, nextSunriseJd) {
  const r = addon.horaFromSunrises(lsk._handle, queryJd, sunriseJd, nextSunriseJd);
  checkStatus('hora_from_sunrises', r.status);
  return r.hora;
}

function ghatikaFromSunrises(lsk, queryJd, sunriseJd, nextSunriseJd) {
  const r = addon.ghatikaFromSunrises(lsk._handle, queryJd, sunriseJd, nextSunriseJd);
  checkStatus('ghatika_from_sunrises', r.status);
  return r.ghatika;
}

function nakshatraAt(engine, jdTdb, moonSiderealDeg, config) {
  const r = addon.nakshatraAt(engine._handle, jdTdb, moonSiderealDeg, config);
  checkStatus('nakshatra_at', r.status);
  return r.nakshatra;
}

function ghatikaFromElapsed(queryJd, sunriseJd, nextSunriseJd) {
  const r = addon.ghatikaFromElapsed(queryJd, sunriseJd, nextSunriseJd);
  checkStatus('ghatika_from_elapsed', r.status);
  return r.value;
}

function ghatikasSinceSunrise(queryJd, sunriseJd) {
  const r = addon.ghatikasSinceSunrise(queryJd, sunriseJd);
  checkStatus('ghatikas_since_sunrise', r.status);
  return r.value;
}

function allSphutas(inputs) {
  const r = addon.allSphutas(inputs);
  checkStatus('all_sphutas', r.status);
  return r.result;
}

function bhriguBindu(rahu, moon) { return addon.bhriguBindu(rahu, moon); }
function pranaSphuta(lagna, moon) { return addon.pranaSphuta(lagna, moon); }
function dehaSphuta(moon, lagna) { return addon.dehaSphuta(moon, lagna); }
function mrityuSphuta(eighthLord, lagna) { return addon.mrityuSphuta(eighthLord, lagna); }
function tithiSphuta(moon, sun, lagna) { return addon.tithiSphuta(moon, sun, lagna); }
function yogaSphuta(sun, moon) { return addon.yogaSphuta(sun, moon); }
function yogaSphutaNormalized(sun, moon) { return addon.yogaSphutaNormalized(sun, moon); }
function rahuTithiSphuta(rahu, sun, lagna) { return addon.rahuTithiSphuta(rahu, sun, lagna); }
function kshetraSphuta(moon, mars, jupiter, venus, lagna) { return addon.kshetraSphuta(moon, mars, jupiter, venus, lagna); }
function beejaSphuta(sun, venus, jupiter) { return addon.beejaSphuta(sun, venus, jupiter); }
function trisphuta(lagna, moon, gulika) { return addon.trisphuta(lagna, moon, gulika); }
function chatussphuta(trisphutaVal, sun) { return addon.chatussphuta(trisphutaVal, sun); }
function panchasphuta(chatussphutaVal, rahu) { return addon.panchasphuta(chatussphutaVal, rahu); }
function sookshmaTrisphuta(lagna, moon, gulika, sun) { return addon.sookshmaTrisphuta(lagna, moon, gulika, sun); }
function avayogaSphuta(sun, moon) { return addon.avayogaSphuta(sun, moon); }
function kunda(lagna, moon, mars) { return addon.kunda(lagna, moon, mars); }
function bhavaLagna(sunLon, ghatikas) { return addon.bhavaLagna(sunLon, ghatikas); }
function horaLagna(sunLon, ghatikas) { return addon.horaLagna(sunLon, ghatikas); }
function ghatiLagna(sunLon, ghatikas) { return addon.ghatiLagna(sunLon, ghatikas); }
function vighatiLagna(lagnaLon, vighatikas) { return addon.vighatiLagna(lagnaLon, vighatikas); }
function varnadaLagna(lagnaLon, horaLagnaLon) { return addon.varnadaLagna(lagnaLon, horaLagnaLon); }
function sreeLagna(moonLon, lagnaLon) { return addon.sreeLagna(moonLon, lagnaLon); }
function pranapadaLagna(sunLon, ghatikas) { return addon.pranapadaLagna(sunLon, ghatikas); }
function induLagna(moonLon, lagnaLord, moon9thLord) { return addon.induLagna(moonLon, lagnaLord, moon9thLord); }

function arudhaPada(bhavaCuspLon, lordLon, rashiHint = 0) {
  const r = addon.arudhaPada(bhavaCuspLon, lordLon, rashiHint);
  checkStatus('arudha_pada', r.status);
  return r.result;
}

function sunBasedUpagrahas(engineOrSunSiderealLongitude, jdTdb, ayanamshaSystem = 0, useNutation = true) {
  let sunSiderealLongitude = engineOrSunSiderealLongitude;
  if (engineOrSunSiderealLongitude && engineOrSunSiderealLongitude._handle) {
    const sid = addon.grahaLongitudes(engineOrSunSiderealLongitude._handle, jdTdb, {
      kind: 0,
      ayanamshaSystem,
      useNutation: !!useNutation,
    });
    checkStatus('graha_longitudes', sid.status);
    sunSiderealLongitude = sid.longitudes[0];
  }
  const r = addon.sunBasedUpagrahas(sunSiderealLongitude);
  checkStatus('sun_based_upagrahas', r.status);
  return r.result;
}

function timeUpagrahaJd(upagrahaIndex, weekday, isDay, sunriseJd, sunsetJd, nextSunriseJd, upagrahaConfig = undefined) {
  const r = upagrahaConfig === undefined
    ? addon.timeUpagrahaJd(upagrahaIndex, weekday, !!isDay, sunriseJd, sunsetJd, nextSunriseJd)
    : addon.timeUpagrahaJd(upagrahaIndex, weekday, !!isDay, sunriseJd, sunsetJd, nextSunriseJd, upagrahaConfig);
  checkStatus(upagrahaConfig === undefined ? 'time_upagraha_jd' : 'time_upagraha_jd_with_config', r.status);
  return r.jdTdb;
}

function timeUpagrahaJdUtc(engine, eop, utc, location, riseSetConfig, upagrahaIndex, upagrahaConfig = undefined) {
  const r = upagrahaConfig === undefined
    ? addon.timeUpagrahaJdUtc(
      engine._handle,
      eop._handle,
      utc,
      location,
      riseSetConfig,
      upagrahaIndex,
    )
    : addon.timeUpagrahaJdUtc(
      engine._handle,
      eop._handle,
      utc,
      location,
      riseSetConfig,
      upagrahaIndex,
      upagrahaConfig,
    );
  checkStatus(upagrahaConfig === undefined ? 'time_upagraha_jd_utc' : 'time_upagraha_jd_utc_with_config', r.status);
  return r.jdTdb;
}

function calculateAshtakavarga(grahaRashis, lagnaRashi) {
  const r = addon.calculateAshtakavarga(grahaRashis, lagnaRashi);
  checkStatus('calculate_ashtakavarga', r.status);
  return r.result;
}

function calculateBav(grahaIndex, grahaRashis, lagnaRashi) {
  const r = addon.calculateBav(grahaIndex, grahaRashis, lagnaRashi);
  checkStatus('calculate_bav', r.status);
  return r.result;
}

function calculateAllBav(grahaRashis, lagnaRashi) {
  const r = addon.calculateAllBav(grahaRashis, lagnaRashi);
  checkStatus('calculate_all_bav', r.status);
  return r.results;
}

function calculateSav(bavs) {
  const r = addon.calculateSav(bavs);
  checkStatus('calculate_sav', r.status);
  return r.result;
}

function trikonaSodhana(totals) {
  const r = addon.trikonaSodhana(totals);
  checkStatus('trikona_sodhana', r.status);
  return r.result;
}

function ekadhipatyaSodhana(totals, grahaRashis, lagnaRashi) {
  const r = addon.ekadhipatyaSodhana(totals, grahaRashis, lagnaRashi);
  checkStatus('ekadhipatya_sodhana', r.status);
  return r.result;
}

function ashtakavargaForDate(engine, eop, utc, location, ayanamshaSystem = 0, useNutation = true) {
  const r = addon.ashtakavargaForDate(engine._handle, eop._handle, utc, location, ayanamshaSystem, !!useNutation);
  checkStatus('ashtakavarga_for_date', r.status);
  return r.result;
}

// Tara bala tara indices (dhruv_tara_name order).
const TARA = Object.freeze({
  JANMA: 0,
  SAMPAT: 1,
  VIPAT: 2,
  KSHEMA: 3,
  PRATYAK: 4,
  SADHAKA: 5,
  VADHA: 6,
  MITRA: 7,
  PARAMA_MITRA: 8,
});

// Samvatsara office indices (dhruv_samvatsara_office_name order).
const SAMVATSARA_OFFICE = Object.freeze({
  RAJA: 0,
  MANTRI: 1,
  SASYESHA: 2,
  DHANYESHA: 3,
  MEGHESHA: 4,
  RASESHA: 5,
  NIRASESHA: 6,
  PHALESHA: 7,
  DURGESHA: 8,
});

// Reference selecting the chart's own Moon nakshatra for tara bala
// (DHRUV_TARA_BALA_REFERENCE_CHART_MOON).
const TARA_BALA_REFERENCE_CHART_MOON = 255;

// Tara bala of `targetNakshatraIndex` counted inclusively from
// `referenceNakshatraIndex` (both 0..26). Returns { taraIndex, count, cycle,
// favorable }: `taraIndex` is the TARA code, `count` the inclusive count
// 1..27, `cycle` the pass through the nine-fold sequence 1..3, and
// `favorable` true for Sampat, Kshema, Sadhaka, Mitra, and Parama Mitra.
function taraBala(referenceNakshatraIndex, targetNakshatraIndex) {
  const r = addon.taraBala(referenceNakshatraIndex, targetNakshatraIndex);
  checkStatus('tara_bala', r.status);
  return r.result;
}

// Tara bala of every graha and the lagna at `utc` against a reference
// nakshatra (TARA_BALA_REFERENCE_CHART_MOON = this chart's own Moon).
// Returns { referenceNakshatraIndex, referenceIsChartMoon, grahas[9], lagna }
// with entries { nakshatraIndex, taraBala }.
function taraBalaForDate(
  engine,
  eop,
  utc,
  location,
  ayanamshaSystem = 0,
  useNutation = true,
  referenceNakshatraIndex = TARA_BALA_REFERENCE_CHART_MOON,
) {
  const r = addon.taraBalaForDate(
    engine._handle,
    eop._handle,
    utc,
    location,
    ayanamshaSystem,
    !!useNutation,
    referenceNakshatraIndex,
  );
  checkStatus('tara_bala_for_date', r.status);
  return r.result;
}

// Kakshya (eighth of a rashi, 3 deg 45 min) index 0..7 of a sidereal
// longitude; -1 for a non-finite input.
function kakshyaIndex(siderealLongitudeDeg) {
  return addon.kakshyaIndex(siderealLongitudeDeg);
}

// BAV contributor index of a kakshya lord (0=Sun .. 6=Saturn, 7=Lagna) in
// the order Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Lagna; -1 for
// an invalid kakshya index.
function kakshyaLord(kakshyaIdx) {
  return addon.kakshyaLord(kakshyaIdx);
}

// Whether the kakshya lord at `siderealLongitudeDeg` contributed a bindu to
// that rashi in `bav` (a bhinna ashtakavarga object with `contributors`).
function kakshyaBindu(bav, siderealLongitudeDeg) {
  const r = addon.kakshyaBindu(bav, siderealLongitudeDeg);
  checkStatus('kakshya_bindu', r.status);
  return r.value;
}

// Transit ashtakavarga of seven transit sidereal longitudes (Sun..Saturn)
// against a natal ashtakavarga result (the `calculateAshtakavarga` /
// `ashtakavargaForDate` shape). Returns 7 entries { grahaIndex, rashiIndex,
// bavBindus, savTotal, savAfterTrikona, savAfterEkadhipatya, kakshyaIndex,
// kakshyaLord, kakshyaBindu }. Rahu/Ketu and outer planets are not scored.
function transitAshtakavarga(natal, transitSiderealLons) {
  const r = addon.transitAshtakavarga(natal, transitSiderealLons);
  checkStatus('transit_ashtakavarga', r.status);
  return r.entries;
}

// Transit ashtakavarga at `utc` against the natal inputs
// `reference` = { grahaRashis: [7], lagnaRashi }. Returns
// { reference, entries[7] }.
function transitAshtakavargaForDate(engine, eop, utc, ayanamshaSystem = 0, useNutation = true, reference) {
  const r = addon.transitAshtakavargaForDate(
    engine._handle,
    eop._handle,
    utc,
    ayanamshaSystem,
    !!useNutation,
    reference,
  );
  checkStatus('transit_ashtakavarga_for_date', r.status);
  return r.result;
}

// Office-bearers of the lunar year containing `utc`: each office is the vaar
// lord of the Vedic day (sunrise-to-sunrise at `location`) of its defining
// event. Returns { varsha, officers[9] } with officers { officeIndex,
// officeName, vaarIndex, lordIndex, eventUtc, vedicDayStart }. `riseSetConfig`
// null selects the library defaults.
function samvatsaraOfficers(engine, eop, utc, location, ayanamshaSystem = 0, useNutation = true, riseSetConfig = null) {
  const r = addon.samvatsaraOfficers(
    engine._handle,
    eop._handle,
    utc,
    location,
    ayanamshaSystem,
    !!useNutation,
    riseSetConfig,
  );
  checkStatus('samvatsara_officers', r.status);
  return r.result;
}

function grahaDrishti(grahaIndex, sourceLon, targetLon) {
  const r = addon.grahaDrishti(grahaIndex, sourceLon, targetLon);
  checkStatus('graha_drishti', r.status);
  return r.result;
}

function grahaDrishtiMatrixForLongitudes(siderealLongitudes) {
  const r = addon.grahaDrishtiMatrix(siderealLongitudes);
  checkStatus('graha_drishti_matrix', r.status);
  return r.result;
}

function drishtiForDate(engine, eop, utc, location, bhavaConfig, riseSetConfig, ayanamshaSystem = 0, useNutation = true, config) {
  const r = addon.drishtiForDate(
    engine._handle,
    eop._handle,
    utc,
    location,
    bhavaConfig,
    riseSetConfig,
    ayanamshaSystem,
    !!useNutation,
    config,
  );
  checkStatus('drishti', r.status);
  return r.result;
}

function grahaPositionsForDate(engine, eop, utc, location, bhavaConfig, ayanamshaSystem = 0, useNutation = true, config) {
  const r = addon.grahaPositionsForDate(
    engine._handle,
    eop._handle,
    utc,
    location,
    bhavaConfig,
    ayanamshaSystem,
    !!useNutation,
    config,
  );
  checkStatus('graha_positions', r.status);
  return r.result;
}

// Fixed-cadence sampling of grahaPositionsForDate over [fromUtc, toUtc]:
// one point per stepMinutes (endpoints inclusive on the grid, max 10000
// points). Each point carries { utc, jdUtc, positions }.
function grahaPositionsSeriesForDate(engine, eop, fromUtc, toUtc, stepMinutes, location, bhavaConfig, ayanamshaSystem = 0, useNutation = true, config) {
  const r = addon.grahaPositionsSeriesForDate(
    engine._handle,
    eop._handle,
    fromUtc,
    toUtc,
    stepMinutes,
    location,
    bhavaConfig,
    ayanamshaSystem,
    !!useNutation,
    config,
  );
  checkStatus('graha_positions_series', r.status);
  return r.result;
}

function horaLord(vaarIndex, horaIndex) {
  return addon.horaLord(vaarIndex, horaIndex).grahaIndex;
}

function masaLord(masaIndex) {
  return addon.masaLord(masaIndex).grahaIndex;
}

function samvatsaraLord(samvatsaraIndex) {
  return addon.samvatsaraLord(samvatsaraIndex).grahaIndex;
}

function exaltationDegree(grahaIndex) {
  const r = addon.exaltationDegree(grahaIndex);
  checkStatus('exaltation_degree', r.status);
  return r.hasValue ? r.value : null;
}

function debilitationDegree(grahaIndex) {
  const r = addon.debilitationDegree(grahaIndex);
  checkStatus('debilitation_degree', r.status);
  return r.hasValue ? r.value : null;
}

function moolatrikoneRange(grahaIndex) {
  const r = addon.moolatrikoneRange(grahaIndex);
  checkStatus('moolatrikone_range', r.status);
  return r.hasValue ? { rashiIndex: r.rashiIndex, startDeg: r.startDeg, endDeg: r.endDeg } : null;
}

function combustionThreshold(grahaIndex, isRetrograde = false) {
  const r = addon.combustionThreshold(grahaIndex, !!isRetrograde);
  checkStatus('combustion_threshold', r.status);
  return r.hasValue ? r.value : null;
}

function isCombust(grahaIndex, grahaSidLon, sunSidLon, isRetrograde = false) {
  const r = addon.isCombust(grahaIndex, grahaSidLon, sunSidLon, !!isRetrograde);
  checkStatus('is_combust', r.status);
  return r.value;
}

function allCombustionStatus(siderealLons9, retrogradeFlags9) {
  const r = addon.allCombustionStatus(siderealLons9, retrogradeFlags9);
  checkStatus('all_combustion_status', r.status);
  return r.result;
}

function naisargikaMaitri(grahaIndex, otherIndex) {
  const r = addon.naisargikaMaitri(grahaIndex, otherIndex);
  checkStatus('naisargika_maitri', r.status);
  return r.code;
}

function tatkalikaMaitri(grahaRashiIndex, otherRashiIndex) {
  const r = addon.tatkalikaMaitri(grahaRashiIndex, otherRashiIndex);
  checkStatus('tatkalika_maitri', r.status);
  return r.code;
}

function panchadhaMaitri(naisargikaCode, tatkalikaCode) {
  const r = addon.panchadhaMaitri(naisargikaCode, tatkalikaCode);
  checkStatus('panchadha_maitri', r.status);
  return r.code;
}

function dignityInRashi(grahaIndex, siderealLon, rashiIndex) {
  const r = addon.dignityInRashi(grahaIndex, siderealLon, rashiIndex);
  checkStatus('dignity_in_rashi', r.status);
  return r.code;
}

function dignityInRashiWithPositions(grahaIndex, siderealLon, rashiIndex, saptaRashiIndices) {
  const r = addon.dignityInRashiWithPositions(grahaIndex, siderealLon, rashiIndex, saptaRashiIndices);
  checkStatus('dignity_in_rashi_with_positions', r.status);
  return r.code;
}

function nodeDignityInRashi(grahaIndex, rashiIndex, grahaRashiIndices9, policyCode) {
  const r = addon.nodeDignityInRashi(grahaIndex, rashiIndex, grahaRashiIndices9, policyCode);
  checkStatus('node_dignity_in_rashi', r.status);
  return r.code;
}

function naturalBeneficMalefic(grahaIndex) {
  const r = addon.naturalBeneficMalefic(grahaIndex);
  checkStatus('natural_benefic_malefic', r.status);
  return r.code;
}

function moonBeneficNature(moonSunElongation) {
  const r = addon.moonBeneficNature(moonSunElongation);
  checkStatus('moon_benefic_nature', r.status);
  return r.code;
}

function grahaGender(grahaIndex) {
  const r = addon.grahaGender(grahaIndex);
  checkStatus('graha_gender', r.status);
  return r.code;
}

function coreBindusForDate(engine, eop, utc, location, bhavaConfig, riseSetConfig, ayanamshaSystem = 0, useNutation = true, config) {
  const r = addon.coreBindusForDate(
    engine._handle,
    eop._handle,
    utc,
    location,
    bhavaConfig,
    riseSetConfig,
    ayanamshaSystem,
    !!useNutation,
    config,
  );
  checkStatus('core_bindus', r.status);
  return r.result;
}

/**
 * Amsha point families. Each code names one amsha chart section; a point is
 * identified by its family and its index within that section. Every entry in
 * an amsha chart already carries `name`, `displayName`, `family` and
 * `pointIndex`, so these are only needed to enumerate a family up front.
 */
const AMSHA_POINT_FAMILY = Object.freeze({
  LAGNA: 0,
  GRAHA: 1,
  OUTER_PLANET: 2,
  BHAVA_CUSP: 3,
  RASHI_BHAVA_CUSP: 4,
  ARUDHA_PADA: 5,
  RASHI_BHAVA_ARUDHA_PADA: 6,
  UPAGRAHA: 7,
  SPHUTA: 8,
  SPECIAL_LAGNA: 9,
  COUNT: 10,
});

/** Number of points in an amsha point family; 0 for an unknown family. */
function amshaPointCount(family) {
  return addon.amshaPointCount(family);
}

/** Display name of the point at (family, index), or null if out of range. */
function amshaPointName(family, index) {
  return addon.amshaPointName(family, index);
}

/** Stable snake_case key of the point at (family, index), or null. */
function amshaPointKey(family, index) {
  return addon.amshaPointKey(family, index);
}

/**
 * Sanskrit display name for a D-number ('Navamsha' for 9), or null for a code
 * outside the 34 supported amshas.
 */
function amshaSanskritName(amshaCode) {
  return addon.amshaSanskritName(amshaCode);
}

function amshaLongitude(siderealLon, amshaCode, variationCode) {
  const r = addon.amshaLongitude(siderealLon, amshaCode, variationCode);
  checkStatus('amsha_longitude', r.status);
  return r.longitudeDeg;
}

function amshaRashiInfo(siderealLon, amshaCode, variationCode) {
  const r = addon.amshaRashiInfo(siderealLon, amshaCode, variationCode);
  checkStatus('amsha_rashi_info', r.status);
  return r.rashi;
}

function amshaLongitudes(siderealLon, amshaCodes, variationCodes) {
  const r = addon.amshaLongitudes(siderealLon, amshaCodes, variationCodes);
  checkStatus('amsha_longitudes', r.status);
  return r.longitudes;
}

function amshaChartForDate(
  engine,
  eop,
  utc,
  location,
  bhavaConfig,
  riseSetConfig,
  ayanamshaSystem = 0,
  useNutation = true,
  amshaCode,
  variationCode,
  scope,
) {
  const r = addon.amshaChartForDate(
    engine._handle,
    eop._handle,
    utc,
    location,
    bhavaConfig,
    riseSetConfig,
    ayanamshaSystem,
    !!useNutation,
    amshaCode,
    variationCode,
    scope,
  );
  checkStatus('amsha_chart_for_date', r.status);
  return r.result;
}

// Caps enforced by the C ABI range operations.
// amshaSeries rejects grids whose points x unique requests exceed this.
const MAX_AMSHA_SERIES_CELLS = 100000;
// amshaLagnaEvents hard ceiling on total segments across all amshas.
const MAX_AMSHA_LAGNA_SEGMENTS = 50000;

// Fixed-cadence slim varga charts over [fromUtc, toUtc]: one point per
// stepMinutes starting at fromUtc (endpoints inclusive when on the grid).
// `amshaCodes` is a non-empty array of amsha codes; `variationCodes` is null
// (all default variations) or a parallel array. Each point carries one chart
// per request, in request order (duplicates repeated). The varga lagna is
// always computed; per-graha entries are added when `includeGrahas` is true.
// Rejects stepMinutes === 0, reversed ranges, empty or invalid request
// lists, and grids whose points x unique requests exceed
// MAX_AMSHA_SERIES_CELLS. Each point is { utc, jdUtc, charts } with charts
// entries { amshaCode, variationCode, lagna, grahas } (grahas is null unless
// includeGrahas).
function amshaSeries(
  engine,
  eop,
  fromUtc,
  toUtc,
  stepMinutes,
  location,
  amshaCodes,
  variationCodes = null,
  includeGrahas = true,
  sankrantiConfig = addon.sankrantiConfigDefault(),
) {
  const r = addon.amshaSeries(
    engine._handle,
    eop._handle,
    fromUtc,
    toUtc,
    stepMinutes,
    location,
    sankrantiConfig,
    amshaCodes,
    variationCodes,
    !!includeGrahas,
  );
  checkStatus('amsha_series', r.status);
  return r.result;
}

// Exact varga-lagna rashi segments overlapping [fromUtc, toUtc] (no sampling
// grid): one entry per unique request (duplicates collapsed), in request
// order. `variationCodes` is null (all default variations) or a parallel
// array. `maxSegments` caps the total segments across all amshas (0 selects
// MAX_AMSHA_LAGNA_SEGMENTS). Per entry, segments chain exactly (`end` equals
// the next segment's `start`); the first segment starts at fromUtc and the
// last ends at the first transition at or after toUtc. Returns
// { entries: [{ amshaCode, variationCode, segments: [{ rashiIndex, start,
// end }] }], truncated, nextFromUtc }. When `truncated` is true, resume from
// `nextFromUtc` and deduplicate on segment start.
function amshaLagnaEvents(
  engine,
  eop,
  fromUtc,
  toUtc,
  location,
  amshaCodes,
  variationCodes = null,
  maxSegments = 0,
  sankrantiConfig = addon.sankrantiConfigDefault(),
) {
  const r = addon.amshaLagnaEvents(
    engine._handle,
    eop._handle,
    fromUtc,
    toUtc,
    location,
    sankrantiConfig,
    amshaCodes,
    variationCodes,
    maxSegments,
  );
  checkStatus('amsha_lagna_events', r.status);
  return r.result;
}

function amshaVariations(amshaCode) {
  const r = addon.amshaVariations(amshaCode);
  checkStatus('amsha_variations', r.status);
  return r.catalog;
}

function amshaVariationsMany(amshaCodes) {
  const r = addon.amshaVariationsMany(amshaCodes);
  checkStatus('amsha_variations_many', r.status);
  return r.catalogs;
}

module.exports = {
  elongationAt,
  siderealSumAt,
  vedicDaySunrises,
  bodyEclipticLonLat,
  tithiAt,
  karanaAt,
  yogaAt,
  vaarFromSunrises,
  horaFromSunrises,
  ghatikaFromSunrises,
  nakshatraAt,
  ghatikaFromElapsed,
  ghatikasSinceSunrise,
  allSphutas,
  bhriguBindu,
  pranaSphuta,
  dehaSphuta,
  mrityuSphuta,
  tithiSphuta,
  yogaSphuta,
  yogaSphutaNormalized,
  rahuTithiSphuta,
  kshetraSphuta,
  beejaSphuta,
  trisphuta,
  chatussphuta,
  panchasphuta,
  sookshmaTrisphuta,
  avayogaSphuta,
  kunda,
  bhavaLagna,
  horaLagna,
  ghatiLagna,
  vighatiLagna,
  varnadaLagna,
  sreeLagna,
  pranapadaLagna,
  induLagna,
  arudhaPada,
  sunBasedUpagrahas,
  timeUpagrahaJd,
  timeUpagrahaJdUtc,
  calculateAshtakavarga,
  calculateBav,
  calculateAllBav,
  calculateSav,
  trikonaSodhana,
  ekadhipatyaSodhana,
  ashtakavargaForDate,
  TARA,
  SAMVATSARA_OFFICE,
  TARA_BALA_REFERENCE_CHART_MOON,
  taraBala,
  taraBalaForDate,
  kakshyaIndex,
  kakshyaLord,
  kakshyaBindu,
  transitAshtakavarga,
  transitAshtakavargaForDate,
  samvatsaraOfficers,
  grahaDrishti,
  grahaDrishtiMatrixForLongitudes,
  drishtiForDate,
  grahaPositionsForDate,
  grahaPositionsSeriesForDate,
  horaLord,
  masaLord,
  samvatsaraLord,
  exaltationDegree,
  debilitationDegree,
  moolatrikoneRange,
  combustionThreshold,
  isCombust,
  allCombustionStatus,
  naisargikaMaitri,
  tatkalikaMaitri,
  panchadhaMaitri,
  dignityInRashi,
  dignityInRashiWithPositions,
  nodeDignityInRashi,
  naturalBeneficMalefic,
  moonBeneficNature,
  grahaGender,
  coreBindusForDate,
  AMSHA_POINT_FAMILY,
  amshaPointCount,
  amshaPointName,
  amshaPointKey,
  amshaSanskritName,
  amshaLongitude,
  amshaRashiInfo,
  amshaLongitudes,
  amshaChartForDate,
  amshaSeries,
  amshaLagnaEvents,
  amshaVariations,
  amshaVariationsMany,
  MAX_AMSHA_SERIES_CELLS,
  MAX_AMSHA_LAGNA_SEGMENTS,
};
