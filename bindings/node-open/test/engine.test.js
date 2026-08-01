'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const dhruv = require('..');
const { hasKernels, hasEop, kernelPaths } = require('./helpers');

test('api version matches expected ABI', () => {
  assert.equal(dhruv.EXPECTED_API_VERSION, 87);
  assert.equal(dhruv.apiVersion(), dhruv.EXPECTED_API_VERSION);
  assert.doesNotThrow(() => dhruv.verifyAbi());
});

test('build identity reports non-empty static strings', () => {
  const version = dhruv.libraryVersion();
  assert.equal(typeof version, 'string');
  assert.ok(version.length > 0);
  const hash = dhruv.buildGitHash();
  assert.equal(typeof hash, 'string');
  assert.ok(hash.length > 0);
});

test('amshaSanskritName resolves D-numbers and rejects unsupported codes', () => {
  assert.equal(dhruv.amshaSanskritName(1), 'Rashi');
  assert.equal(dhruv.amshaSanskritName(9), 'Navamsha');
  assert.equal(dhruv.amshaSanskritName(144), 'Dwadashashtottaramsha');
  // 13 is not an amsha; 65545 must not wrap onto 9 and report 'Navamsha'.
  for (const code of [0, 13, 145, 65545]) {
    assert.equal(dhruv.amshaSanskritName(code), null);
  }
});

test('panchang include mask constants match the C ABI', () => {
  assert.equal(dhruv.PANCHANG_INCLUDE.ALL_CORE, 0x07f);
  assert.equal(dhruv.PANCHANG_INCLUDE.ALL_CALENDAR, 0x380);
  assert.equal(dhruv.PANCHANG_INCLUDE.ALL, 0x3ff);
  assert.equal(dhruv.PANCHANG_INCLUDE.LOCATION_INDEPENDENT, 0x3c7);
  assert.equal(dhruv.PANCHANG_INCLUDE.LOCATION_DEPENDENT, 0x038);
  const fullCfg = dhruv.fullKundaliConfigDefault();
  assert.equal(typeof fullCfg.panchangIncludeMask, 'number');
});

test('calculateBhavaBala opts into node aspects explicitly', () => {
  const aspectVirupas = Array.from({ length: 9 }, () => Array(12).fill(0));
  aspectVirupas[4][0] = 40; // Guru full positive.
  aspectVirupas[7][0] = 20; // Rahu quarter-negative only when included.
  const base = {
    cuspSiderealLons: Array(12).fill(0),
    ascendantSiderealLon: 0,
    meridianSiderealLon: 90,
    grahaBhavaNumbers: Array(9).fill(0),
    houseLordStrengths: Array(12).fill(0),
    aspectVirupas,
    birthPeriod: 0,
  };

  const withoutNodes = dhruv.calculateBhavaBala(base);
  const withNodes = dhruv.calculateBhavaBala({ ...base, includeNodeAspects: true });

  assert.equal(withoutNodes.entries[0].drishti, 40);
  assert.equal(withNodes.entries[0].drishti, 35);
});

test('calculateBhavaBala applies dynamic Chandra and Buddh rules', () => {
  const aspectVirupas = Array.from({ length: 9 }, () => Array(12).fill(0));
  aspectVirupas[4][0] = 40; // Guru full positive.
  aspectVirupas[3][0] = 30; // Buddh full signed by dynamic nature.
  aspectVirupas[1][0] = 20; // Chandra quarter signed by selected rule.
  const base = {
    cuspSiderealLons: Array(12).fill(0),
    ascendantSiderealLon: 0,
    meridianSiderealLon: 90,
    grahaBhavaNumbers: Array(9).fill(0),
    grahaSiderealLons: [0, 10, 60, 60, 0, 0, 0, 0, 0],
    houseLordStrengths: Array(12).fill(0),
    aspectVirupas,
    chandraBeneficRule: dhruv.CHANDRA_BENEFIC_RULE.BRIGHTNESS_72,
    birthPeriod: 0,
  };

  const malefic = dhruv.calculateBhavaBala(base);
  const benefic = dhruv.calculateBhavaBala({
    ...base,
    grahaSiderealLons: [0, 180, 60, 90, 0, 0, 0, 0, 0],
    chandraBeneficRule: dhruv.CHANDRA_BENEFIC_RULE.WAXING_180,
  });

  assert.equal(malefic.entries[0].drishti, 5);
  assert.equal(benefic.entries[0].drishti, 75);
});

test('calculateBhavaBala includes special rules only when requested', () => {
  const base = {
    cuspSiderealLons: [65, ...Array(11).fill(0)],
    ascendantSiderealLon: 15,
    meridianSiderealLon: 105,
    grahaBhavaNumbers: [0, 0, 0, 0, 1, 0, 0, 0, 0],
    houseLordStrengths: [300, ...Array(11).fill(0)],
    aspectVirupas: Array.from({ length: 9 }, () => Array(12).fill(0)),
    birthPeriod: 0,
  };

  const withoutSpecial = dhruv.calculateBhavaBala(base);
  const withSpecial = dhruv.calculateBhavaBala({ ...base, includeSpecialRules: true });

  assert.equal(withSpecial.entries[0].occupationBonus, 60);
  assert.equal(withSpecial.entries[0].risingBonus, 15);
  assert.equal(withSpecial.entries[0].totalVirupas - withoutSpecial.entries[0].totalVirupas, 75);
});

test('amsha variation helpers expose per-amsha catalogs', () => {
  const d2 = dhruv.amshaVariations(2);
  assert.equal(d2.amshaCode, 2);
  assert.equal(d2.defaultVariationCode, 0);
  assert.equal(d2.variations.length, 4);
  assert.deepEqual(
    d2.variations.map((entry) => entry.name),
    ['default', 'cancer-leo-only', 'lunar-hora', 'kashinath-hora'],
  );
  assert.ok(Math.abs(dhruv.amshaLongitude(1.25, 2, 2) - 135) < 0.01);
  assert.ok(Math.abs(dhruv.amshaLongitude(20, 2, 3) - 220) < 0.01);

  const many = dhruv.amshaVariationsMany([2, 9]);
  assert.equal(many.length, 2);
  assert.equal(many[1].amshaCode, 9);
  assert.equal(many[1].variations.length, 1);
  assert.equal(many[1].variations[0].variationCode, 0);
});

test('config loading supports discovery defaults', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'dhruv-config-'));
  const configPath = path.join(dir, 'config.toml');
  const prior = process.env.DHRUV_CONFIG_FILE;
  fs.writeFileSync(configPath, 'version = 1\n');
  process.env.DHRUV_CONFIG_FILE = configPath;
  try {
    const cfg = dhruv.Config.load(null, 0);
    assert.ok(cfg instanceof dhruv.Config);
    assert.doesNotThrow(() => dhruv.clearActiveConfig());
    cfg.close();
  } finally {
    if (prior === undefined) {
      delete process.env.DHRUV_CONFIG_FILE;
    } else {
      process.env.DHRUV_CONFIG_FILE = prior;
    }
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('engine query and UTC roundtrip', { skip: !hasKernels() }, () => {
  const paths = kernelPaths();

  const engine = dhruv.Engine.create({
    spkPaths: [paths.spk],
    lskPath: paths.lsk,
    cacheCapacity: 64,
    strictValidation: false,
  });

  const lsk = dhruv.LSK.load(paths.lsk);

  const state = engine.query({
    target: 301,
    observer: 399,
    frame: 1,
    epochTdbJd: 2451545.0,
  });

  assert.ok(Number.isFinite(state.state.positionKm[0]));
  assert.equal(state.sphericalState, null);

  const utc = { year: 2025, month: 1, day: 1, hour: 0, minute: 0, second: 0 };
  const converted = dhruv.utcToTdbJd(lsk, { utc });
  const back = dhruv.jdTdbToUtc(lsk, converted.jdTdb);
  const spherical = engine.query({
    target: 301,
    observer: 399,
    frame: 1,
    utc,
    outputMode: dhruv.QUERY_OUTPUT.SPHERICAL,
  });

  assert.equal(back.year, utc.year);
  assert.equal(back.month, utc.month);
  assert.equal(back.day, utc.day);
  assert.ok(Array.isArray(converted.diagnostics.warnings));
  assert.equal(spherical.state, null);
  assert.ok(Number.isFinite(spherical.sphericalState.lonDeg));

  lsk.close();
  engine.close();
});

test('engine replaceSpks and listSpks', { skip: !hasKernels() }, () => {
  const paths = kernelPaths();

  const engine = dhruv.Engine.create({
    spkPaths: [paths.spk],
    lskPath: paths.lsk,
    cacheCapacity: 64,
    strictValidation: false,
  });

  const initial = engine.listSpks();
  assert.equal(initial.length, 1);
  assert.equal(initial[0].generation, 0);

  const report = engine.replaceSpks([paths.spk, paths.spk]);
  assert.deepEqual(report, {
    generation: 1,
    activeCount: 2,
    loadedCount: 0,
    reusedCount: 2,
  });

  const active = engine.listSpks();
  assert.equal(active.length, 2);
  assert.equal(active[0].generation, report.generation);
  assert.equal(active[1].generation, report.generation);

  assert.throws(
    () => engine.replaceSpks([path.join(path.dirname(paths.spk), 'missing.bsp')]),
    /engine_replace_spks/,
  );
  assert.equal(engine.listSpks()[0].generation, report.generation);

  engine.close();
});

test('search and panchang smoke', { skip: !(hasKernels() && hasEop()) }, () => {
  const paths = kernelPaths();

  const engine = dhruv.Engine.create({
    spkPaths: [paths.spk],
    lskPath: paths.lsk,
    cacheCapacity: 64,
    strictValidation: false,
  });

  const eop = dhruv.EOP.load(paths.eop);
  const lsk = dhruv.LSK.load(paths.lsk);
  const riseCfg = dhruv.riseSetConfigDefault();
  const sankCfg = dhruv.sankrantiConfigDefault();
  const bhavaCfg = dhruv.bhavaConfigDefault();
  assert.equal(bhavaCfg.useRashiBhavaForBalaAvastha, true);
  assert.equal(bhavaCfg.includeNodeAspectsForDrikBala, false);
  assert.equal(bhavaCfg.includeSpecialBhavaBalaRules, true);
  assert.equal(bhavaCfg.divideGuruBuddhDrishtiBy4ForDrikBala, true);
  assert.equal(bhavaCfg.chandraBeneficRule, 0);
  assert.equal(bhavaCfg.includeRashiBhavaResults, true);
  const utc = {
    year: 2025,
    month: 1,
    day: 15,
    hour: 12,
    minute: 0,
    second: 0,
  };

  const search = dhruv.lunarPhaseSearch(
    engine,
    {
      phaseKind: 1,
      queryMode: 2,
      startUtc: {
        year: 2000,
        month: 1,
        day: 1,
        hour: 12,
        minute: 0,
        second: 0,
      },
      endUtc: {
        year: 2000,
        month: 12,
        day: 31,
        hour: 12,
        minute: 0,
        second: 0,
      },
    },
    1,
  );

  assert.ok(search.events.length >= 12);

  const conj = dhruv.conjunctionSearch(
    engine,
    {
      body1Code: 10,
      body2Code: 301,
      queryMode: 0,
      atUtc: utc,
    },
    4,
  );
  assert.equal(conj.found, true);

  const sank = dhruv.sankrantiSearch(
    engine,
    {
      targetKind: 0,
      queryMode: 0,
      rashiIndex: 0,
      atUtc: utc,
    },
    4,
  );
  assert.equal(sank.found, true);

  const grahan = dhruv.grahanSearch(
    engine,
    {
      grahanKind: 0,
      queryMode: 0,
      atUtc: utc,
    },
    2,
  );
  assert.equal(typeof grahan.found, 'boolean');

  const solar = dhruv.grahanSearch(
    engine,
    {
      grahanKind: 1,
      queryMode: 0,
      atUtc: { year: 2024, month: 3, day: 1, hour: 0, minute: 0, second: 0 },
      location: { latitudeDeg: 25.2854, longitudeDeg: -104.3, altitudeM: 0 },
      config: {
        includePath: true,
        pathStepMinutes: 10,
        boundaryStepDeg: 15,
      },
    },
    2,
  );
  assert.equal(solar.found, true);
  assert.ok(solar.surya.besselian.l1 > 0);
  assert.ok(solar.surya.greatestLocation);
  assert.ok(solar.surya.footprintCount > 0);
  assert.equal(solar.surya.path.length, solar.surya.pathCount);
  assert.equal(solar.surya.footprints.length, solar.surya.footprintCount);
  assert.ok(solar.surya.path[0].center);
  assert.ok(solar.surya.footprints[0].boundary.length > 0);
  assert.equal(typeof solar.surya.local.visible, 'boolean');
  assert.ok(solar.surya.local.c1Utc);
  assert.ok(solar.surya.local.c4Utc);

  const motion = dhruv.motionSearch(
    engine,
    {
      bodyCode: 199,
      motionKind: 0,
      queryMode: 0,
      atUtc: utc,
    },
    2,
  );
  assert.equal(typeof motion.found, 'boolean');

  const tithi = dhruv.tithiForDate(engine, utc);
  assert.ok(Number.isInteger(tithi.tithiIndex));

  const karana = dhruv.karanaForDate(engine, utc);
  assert.ok(Number.isInteger(karana.karanaIndex));

  const yoga = dhruv.yogaForDate(engine, utc, sankCfg);
  assert.ok(Number.isInteger(yoga.yogaIndex));

  const nak = dhruv.nakshatraForDate(engine, utc, sankCfg);
  assert.ok(Number.isInteger(nak.nakshatraIndex));

  const loc = { latitudeDeg: 12.9716, longitudeDeg: 77.5946, altitudeM: 920 };
  const vaar = dhruv.vaarForDate(engine, eop, utc, loc, riseCfg);
  assert.ok(Number.isInteger(vaar.vaarIndex));

  const hora = dhruv.horaForDate(engine, eop, utc, loc, riseCfg);
  assert.ok(Number.isInteger(hora.horaIndex));

  const ghatika = dhruv.ghatikaForDate(engine, eop, utc, loc, riseCfg);
  assert.ok(Number.isInteger(ghatika.value));

  const masa = dhruv.masaForDate(engine, utc, sankCfg);
  assert.ok(Number.isInteger(masa.masaIndex));

  const ayana = dhruv.ayanaForDate(engine, utc, sankCfg);
  assert.ok(Number.isInteger(ayana.ayana));

  const varsha = dhruv.varshaForDate(engine, utc, sankCfg);
  assert.ok(Number.isInteger(varsha.samvatsaraIndex));
  const panchang = dhruv.panchangComputeEx(engine, eop, lsk, {
    timeKind: 1,
    jdTdb: -1,
    utc,
    includeMask: 1,
    location: loc,
    riseSetConfig: riseCfg,
    sankrantiConfig: sankCfg,
  });
  assert.equal(typeof panchang, 'object');
  assert.equal(panchang.tithiValid, true);

  const panchangNoLoc = dhruv.panchangComputeEx(engine, eop, lsk, {
    timeKind: 1,
    jdTdb: -1,
    utc,
    includeMask: dhruv.PANCHANG_INCLUDE.LOCATION_INDEPENDENT,
  });
  assert.equal(panchangNoLoc.tithiValid, true);
  assert.equal(panchangNoLoc.masaValid, true);
  assert.equal(panchangNoLoc.vaarValid, false);
  assert.equal(panchangNoLoc.horaValid, false);
  assert.equal(panchangNoLoc.ghatikaValid, false);

  // Known calendar values are reused verbatim inside their validity window
  // and silently recomputed when stale.
  const calendarReq = {
    timeKind: 1,
    jdTdb: -1,
    utc: { year: 2024, month: 1, day: 15, hour: 12, minute: 0, second: 0 },
    includeMask: dhruv.PANCHANG_INCLUDE.ALL_CALENDAR,
  };
  const firstCalendar = dhruv.panchangComputeEx(engine, eop, lsk, calendarReq);
  assert.equal(firstCalendar.masaValid, true);
  assert.equal(firstCalendar.ayanaValid, true);
  assert.equal(firstCalendar.varshaValid, true);

  const reusedCalendar = dhruv.panchangComputeEx(engine, eop, lsk, {
    ...calendarReq,
    utc: { year: 2024, month: 1, day: 20, hour: 12, minute: 0, second: 0 },
    knownMasa: firstCalendar.masa,
    knownAyana: firstCalendar.ayana,
    knownVarsha: firstCalendar.varsha,
  });
  assert.deepEqual(reusedCalendar.masa, firstCalendar.masa);
  assert.deepEqual(reusedCalendar.ayana, firstCalendar.ayana);
  assert.deepEqual(reusedCalendar.varsha, firstCalendar.varsha);

  const recomputedCalendar = dhruv.panchangComputeEx(engine, eop, lsk, {
    ...calendarReq,
    utc: { year: 2024, month: 5, day: 20, hour: 12, minute: 0, second: 0 },
    knownMasa: firstCalendar.masa,
    knownAyana: firstCalendar.ayana,
    knownVarsha: firstCalendar.varsha,
  });
  assert.equal(recomputedCalendar.masaValid, true);
  assert.notDeepEqual(recomputedCalendar.masa, firstCalendar.masa);

  assert.throws(
    () => dhruv.panchangComputeEx(engine, eop, lsk, {
      timeKind: 1,
      jdTdb: -1,
      utc,
      includeMask: dhruv.PANCHANG_INCLUDE.LOCATION_DEPENDENT,
    }),
    (err) => err.status === dhruv.STATUS.INVALID_SEARCH_CONFIG,
  );
  assert.ok(dhruv.bhavaSystemCount() >= 1);

  const riseSet = dhruv.computeRiseSet(engine, eop, loc, riseCfg, 0, 2460000.5, lsk);
  assert.ok(Number.isInteger(riseSet.eventCode));
  assert.ok(Number.isFinite(riseSet.jdTdb));
  const riseUtc = dhruv.riseSetResultToUtc(lsk, riseSet);
  assert.ok(Number.isInteger(riseUtc.year));

  const allEvents = dhruv.computeAllEvents(engine, eop, loc, riseCfg, 2460000.5, lsk);
  assert.equal(allEvents.length, 8);

  const riseSetUtc = dhruv.computeRiseSetUtc(engine, eop, lsk, loc, 0, utc, riseCfg);
  assert.ok(Number.isInteger(riseSetUtc.eventCode));
  assert.equal(typeof riseSetUtc.utc.year, 'number');

  const allEventsUtc = dhruv.computeAllEventsUtc(engine, eop, lsk, loc, utc, riseCfg);
  assert.equal(allEventsUtc.length, 8);

  const bhavas = dhruv.computeBhavas(engine, eop, loc, lsk, 2460000.5, bhavaCfg);
  assert.equal(bhavas.bhavas.length, 12);
  const bhavasUtc = dhruv.computeBhavasUtc(engine, eop, lsk, loc, utc, bhavaCfg);
  assert.equal(bhavasUtc.bhavas.length, 12);

  const lagna = dhruv.lagnaDeg(lsk, eop, loc, 2460000.5);
  const mc = dhruv.mcDeg(lsk, eop, loc, 2460000.5);
  const ramc = dhruv.ramcDeg(lsk, eop, loc, 2460000.5);
  const lagnaUtc = dhruv.lagnaDegUtc(lsk, eop, loc, utc);
  const mcUtc = dhruv.mcDegUtc(lsk, eop, loc, utc);
  const ramcUtc = dhruv.ramcDegUtc(lsk, eop, loc, utc);
  assert.ok(Number.isFinite(lagna));
  assert.ok(Number.isFinite(mc));
  assert.ok(Number.isFinite(ramc));
  assert.ok(Number.isFinite(lagnaUtc));
  assert.ok(Number.isFinite(mcUtc));
  assert.ok(Number.isFinite(ramcUtc));

  const ayanamshaDeg = dhruv.ayanamshaComputeEx(
    lsk,
    {
      systemCode: 0,
      mode: 2,
      timeKind: 1,
      jdTdb: 0,
      utc,
      useNutation: true,
      deltaPsiArcsec: 0,
    },
    eop,
  );
  assert.ok(Number.isFinite(ayanamshaDeg));

  const node = dhruv.lunarNodeDeg(0, 0, 2451545.0);
  assert.ok(Number.isFinite(node));
  const node2 = dhruv.lunarNodeDegWithEngine(engine, 0, 1, 2451545.0);
  assert.ok(Number.isFinite(node2));
  const nodeUtc = dhruv.lunarNodeDegUtc(lsk, 0, 0, utc);
  assert.ok(Number.isFinite(nodeUtc));
  assert.equal(typeof dhruv.lunarNodeDegUtcWithEngine, 'function');

  const rashi = dhruv.rashiFromLongitude(10.0);
  assert.ok(Number.isInteger(rashi.rashiIndex));
  const nak27 = dhruv.nakshatraFromLongitude(10.0);
  assert.ok(Number.isInteger(nak27.nakshatraIndex));
  const nak28 = dhruv.nakshatra28FromLongitude(10.0);
  assert.ok(Number.isInteger(nak28.nakshatraIndex));
  assert.equal(dhruv.rashiCount(), 12);
  assert.ok(dhruv.nakshatraCount(27) >= 27);

  const shadbala = dhruv.shadbalaForDate(engine, eop, {
    year: 2025,
    month: 1,
    day: 15,
    hour: 12,
    minute: 0,
    second: 0,
  }, loc, 0, true);
  assert.equal(shadbala.totalRupas.length, 7);
  assert.equal(shadbala.entries.length, 7);

  const vimsopaka = dhruv.vimsopakaForDate(engine, eop, utc, loc, 0, true, 0);
  assert.equal(vimsopaka.entries.length, 9);

  const avastha = dhruv.avasthaForDate(engine, eop, utc, loc, bhavaCfg, riseCfg, 0, true, 0);
  assert.equal(avastha.entries.length, 9);
  const charakaraka = dhruv.charakarakaForDate(engine, eop, utc, 0, true, 'parashari');
  assert.ok(charakaraka.count >= 7);
  assert.ok(charakaraka.count <= 8);
  assert.equal(charakaraka.scheme, dhruv.CHARAKARAKA_SCHEME.MIXED_PARASHARA);
  assert.ok(Array.isArray(charakaraka.entries));

  const specialLagnas = dhruv.specialLagnasForDate(engine, eop, utc, loc, riseCfg, 0, true);
  assert.ok(Number.isFinite(specialLagnas.bhavaLagna));
  try {
    const arudhas = dhruv.arudhaPadasForDate(engine, eop, utc, loc, 0, true);
    assert.equal(arudhas.length, 12);
  } catch (err) {
    assert.equal(err.status, dhruv.STATUS.INVALID_QUERY);
  }
  try {
    const upagrahas = dhruv.allUpagrahasForDate(engine, eop, utc, loc, 0, true);
    assert.ok(Number.isFinite(upagrahas.gulika));
  } catch (err) {
    assert.equal(err.status, dhruv.STATUS.INVALID_QUERY);
  }

  const kundali = dhruv.fullKundaliSummaryForDate(engine, eop, {
    year: 2025,
    month: 1,
    day: 15,
    hour: 12,
    minute: 0,
    second: 0,
  }, loc, 0, true);
  assert.ok(Number.isFinite(kundali.ayanamshaDeg));
  assert.equal(typeof kundali.charakarakaValid, 'boolean');

  const fullCfg = dhruv.fullKundaliConfigDefault();
  fullCfg.includeBhavaCusps = true;
  fullCfg.includeBindus = true;
  fullCfg.includeUpagrahas = true;
  fullCfg.includeSphutas = true;
  fullCfg.includeSpecialLagnas = true;
  fullCfg.includeDasha = true;
  fullCfg.includeAmshas = true;
  fullCfg.dashaConfig.count = 2;
  fullCfg.dashaConfig.systems[0] = 0;
  fullCfg.dashaConfig.systems[1] = 1;
  fullCfg.dashaConfig.maxLevels[0] = 0;
  fullCfg.dashaConfig.maxLevels[1] = 1;
  fullCfg.amshaScope = {
    includeBhavaCusps: true,
    includeArudhaPadas: true,
    includeUpagrahas: true,
    includeSphutas: true,
    includeSpecialLagnas: true,
    includeOuterPlanets: true,
  };
  fullCfg.amshaSelection.count = 1;
  fullCfg.amshaSelection.codes[0] = 9;
  fullCfg.amshaSelection.variations[0] = 0;
  fullCfg.panchangIncludeMask = dhruv.PANCHANG_INCLUDE.ALL;
  const kundaliFull = dhruv.fullKundaliForDate(engine, eop, utc, loc, bhavaCfg, riseCfg, 0, true, fullCfg);
  assert.equal(kundaliFull.sphutas.longitudes.length, 16);
  assert.equal(kundaliFull.panchang.tithiValid, true);
  assert.equal(kundaliFull.panchang.vaarValid, true);
  assert.equal(kundaliFull.panchang.masaValid, true);
  assert.equal(kundaliFull.amshas.length, 1);
  assert.equal(kundaliFull.amshas[0].bhavaCusps.length, 12);
  assert.equal(kundaliFull.amshas[0].arudhaPadas.length, 12);
  assert.equal(kundaliFull.amshas[0].upagrahas.length, 11);
  assert.equal(kundaliFull.amshas[0].sphutas.length, 16);
  assert.equal(kundaliFull.amshas[0].specialLagnas.length, 8);
  assert.equal(kundaliFull.dasha.length, 2);
  assert.equal(kundaliFull.dasha[0].system, 0);
  assert.equal(kundaliFull.dasha[1].system, 1);
  assert.equal(kundaliFull.dasha[0].levels.length, 1);
  assert.equal(kundaliFull.dasha[1].levels.length, 2);
  const jdNow = dhruv.utcToTdbJd(lsk, { utc }).jdTdb;

  const elongation = dhruv.elongationAt(engine, jdNow);
  assert.ok(Number.isFinite(elongation));
  const siderealSum = dhruv.siderealSumAt(engine, jdNow, sankCfg);
  assert.ok(Number.isFinite(siderealSum));

  const sunrises = dhruv.vedicDaySunrises(engine, eop, utc, loc, riseCfg);
  assert.ok(Number.isFinite(sunrises.sunriseJd));
  assert.ok(Number.isFinite(sunrises.nextSunriseJd));

  const lonLat = dhruv.bodyEclipticLonLat(engine, 301, jdNow);
  assert.ok(Number.isFinite(lonLat.lonDeg));
  assert.ok(Number.isFinite(lonLat.latDeg));

  try {
    const tithiAt = dhruv.tithiAt(engine, jdNow, sunrises.sunriseJd);
    assert.ok(Number.isInteger(tithiAt.tithiIndex));
  } catch (err) {
    assert.equal(err.status, dhruv.STATUS.NO_CONVERGENCE);
  }
  try {
    const karanaAt = dhruv.karanaAt(engine, jdNow, sunrises.sunriseJd);
    assert.ok(Number.isInteger(karanaAt.karanaIndex));
  } catch (err) {
    assert.equal(err.status, dhruv.STATUS.NO_CONVERGENCE);
  }
  try {
    const yogaAt = dhruv.yogaAt(engine, jdNow, sunrises.sunriseJd, sankCfg);
    assert.ok(Number.isInteger(yogaAt.yogaIndex));
  } catch (err) {
    assert.equal(err.status, dhruv.STATUS.NO_CONVERGENCE);
  }

  const vaarFromSunrises = dhruv.vaarFromSunrises(lsk, sunrises.sunriseJd, sunrises.nextSunriseJd);
  const horaFromSunrises = dhruv.horaFromSunrises(lsk, jdNow, sunrises.sunriseJd, sunrises.nextSunriseJd);
  const ghatikaFromSunrises = dhruv.ghatikaFromSunrises(lsk, jdNow, sunrises.sunriseJd, sunrises.nextSunriseJd);
  assert.ok(Number.isInteger(vaarFromSunrises.vaarIndex));
  assert.ok(Number.isInteger(horaFromSunrises.horaIndex));
  assert.ok(Number.isInteger(ghatikaFromSunrises.value));

  const moonSidereal = dhruv.grahaLongitudes(engine, jdNow, {
    kind: dhruv.GRAHA_LONGITUDE_KIND.SIDEREAL,
    ayanamshaSystem: 0,
    useNutation: true,
  })[1];
  const nakshatraAt = dhruv.nakshatraAt(engine, jdNow, moonSidereal, sankCfg);
  assert.ok(Number.isInteger(nakshatraAt.nakshatraIndex));

  const ghatikaElapsed = dhruv.ghatikaFromElapsed(jdNow, sunrises.sunriseJd, sunrises.nextSunriseJd);
  const ghatikasSinceSunrise = dhruv.ghatikasSinceSunrise(jdNow, sunrises.sunriseJd);
  assert.ok(Number.isInteger(ghatikaElapsed));
  assert.ok(Number.isFinite(ghatikasSinceSunrise));

  const sphutas = dhruv.allSphutas({
    sun: 10,
    moon: 20,
    mars: 30,
    jupiter: 40,
    venus: 50,
    rahu: 60,
    lagna: 70,
    eighthLord: 80,
    gulika: 90,
  });
  assert.equal(sphutas.longitudes.length, 16);
  const kshetraViaScalar = dhruv.kshetraSphuta(20, 30, 40, 50, 70);
  // ALL_SPHUTAS order in dhruv_vedic_base: KshetraSphuta is index 8.
  const kshetraIdx = 8;
  assert.ok(Math.abs(kshetraViaScalar - sphutas.longitudes[kshetraIdx]) < 1e-9);

  const arudhaPada = dhruv.arudhaPada(100, 130, 0);
  assert.ok(Number.isFinite(arudhaPada.longitudeDeg));

  const sunUpagrahas = dhruv.sunBasedUpagrahas(dhruv.grahaLongitudes(engine, jdNow, {
    kind: dhruv.GRAHA_LONGITUDE_KIND.SIDEREAL,
    ayanamshaSystem: 0,
    useNutation: true,
  })[0]);
  assert.ok(Number.isFinite(sunUpagrahas.dhooma));
  try {
    const weekday = dhruv.vaarFromJd(sunrises.sunriseJd);
    const isDay = 1;
    const sunsetEstimate = (sunrises.sunriseJd + sunrises.nextSunriseJd) / 2.0;
    const timeUpagraha = dhruv.timeUpagrahaJd(0, weekday, isDay, sunrises.sunriseJd, sunsetEstimate, sunrises.nextSunriseJd);
    assert.ok(Number.isFinite(timeUpagraha));
  } catch (err) {
    assert.ok(err.status === dhruv.STATUS.INVALID_QUERY || err.status === dhruv.STATUS.NO_CONVERGENCE);
  }
  try {
    const timeUpagrahaUtc = dhruv.timeUpagrahaJdUtc(engine, eop, utc, loc, riseCfg, 0);
    assert.ok(Number.isFinite(timeUpagrahaUtc));
  } catch (err) {
    assert.ok(err.status === dhruv.STATUS.INVALID_QUERY || err.status === dhruv.STATUS.NO_CONVERGENCE);
  }

  const ashtakavarga = dhruv.calculateAshtakavarga([0, 1, 2, 3, 4, 5, 6], 0);
  assert.equal(ashtakavarga.bavs.length, 7);
  assert.equal(ashtakavarga.bavs[0].contributors.length, 12);
  const bav = dhruv.calculateBav(0, [0, 1, 2, 3, 4, 5, 6], 0);
  assert.equal(bav.points.length, 12);
  assert.equal(bav.contributors.length, 12);
  for (let i = 0; i < 12; i += 1) {
    assert.equal(bav.contributors[i].length, 8);
    assert.equal(bav.contributors[i].reduce((a, b) => a + b, 0), bav.points[i]);
  }
  const allBav = dhruv.calculateAllBav([0, 1, 2, 3, 4, 5, 6], 0);
  assert.equal(allBav.length, 7);
  const sav = dhruv.calculateSav(allBav);
  assert.equal(sav.totalPoints.length, 12);
  assert.equal(dhruv.trikonaSodhana(Array(12).fill(1)).length, 12);
  assert.equal(dhruv.ekadhipatyaSodhana(Array(12).fill(1), [0, 1, 2, 3, 4, 5, 6], 0).length, 12);
  const ashtakavargaDate = dhruv.ashtakavargaForDate(engine, eop, utc, loc, 0, true);
  assert.equal(ashtakavargaDate.bavs.length, 7);

  const drishtiEntry = dhruv.grahaDrishti(0, 10, 100);
  assert.ok(Number.isFinite(drishtiEntry.totalVirupa));
  const drishtiMatrix = dhruv.grahaDrishtiMatrixForLongitudes(Array(9).fill(0).map((_, i) => i * 10));
  assert.equal(drishtiMatrix.length, 9);
  const drishtiCfg = { includeBhava: true, includeLagna: true, includeBindus: false };
  const drishti = dhruv.drishtiForDate(engine, eop, utc, loc, bhavaCfg, riseCfg, 0, true, drishtiCfg);
  assert.equal(drishti.grahaToGraha.length, 9);

  const grahaPosCfg = { includeNakshatra: true, includeLagna: true, includeOuterPlanets: true, includeBhava: true };
  const grahaPositions = dhruv.grahaPositionsForDate(engine, eop, utc, loc, bhavaCfg, 0, true, grahaPosCfg);
  assert.equal(grahaPositions.grahas.length, 9);
  assert.equal(grahaPositions.grahas[0].equatorialValid, false);
  assert.equal(grahaPositions.earthOrientationValid, false);

  const grahaPosEqCfg = { ...grahaPosCfg, includeEquatorial: true };
  const grahaPositionsEq = dhruv.grahaPositionsForDate(engine, eop, utc, loc, bhavaCfg, 0, true, grahaPosEqCfg);
  for (const entry of grahaPositionsEq.grahas) {
    assert.equal(entry.equatorialValid, true);
    assert.ok(entry.rightAscensionDeg >= 0 && entry.rightAscensionDeg < 360);
    assert.ok(entry.declinationDeg >= -90 && entry.declinationDeg <= 90);
    assert.ok(Number.isFinite(entry.eclipticLatitudeDeg));
  }
  // Rahu (7) and Ketu (8) are ecliptic points: latitude exactly 0.
  assert.equal(grahaPositionsEq.grahas[7].eclipticLatitudeDeg, 0);
  assert.equal(grahaPositionsEq.grahas[8].eclipticLatitudeDeg, 0);
  assert.equal(grahaPositionsEq.lagna.eclipticLatitudeDeg, 0);
  assert.equal(grahaPositionsEq.earthOrientationValid, true);
  assert.ok(grahaPositionsEq.gmstDeg >= 0 && grahaPositionsEq.gmstDeg < 360);
  assert.ok(grahaPositionsEq.gastDeg >= 0 && grahaPositionsEq.gastDeg < 360);

  // Series: 2h at 60-minute cadence yields 3 points; first matches the
  // single-epoch call exactly.
  const seriesTo = { ...utc, hour: utc.hour + 2 };
  const series = dhruv.grahaPositionsSeriesForDate(engine, eop, utc, seriesTo, 60, loc, bhavaCfg, 0, true, grahaPosEqCfg);
  assert.equal(series.length, 3);
  for (let i = 0; i < 9; i++) {
    assert.equal(series[0].positions.grahas[i].siderealLongitude, grahaPositionsEq.grahas[i].siderealLongitude);
  }
  assert.equal(series[0].positions.gmstDeg, grahaPositionsEq.gmstDeg);
  assert.ok(Number.isFinite(series[0].jdUtc));
  assert.throws(() => dhruv.grahaPositionsSeriesForDate(engine, eop, utc, seriesTo, 0, loc, bhavaCfg, 0, true, grahaPosEqCfg));

  const bindusCfg = { includeNakshatra: true, includeBhava: true };
  const bindus = dhruv.coreBindusForDate(engine, eop, utc, loc, bhavaCfg, riseCfg, 0, true, bindusCfg);
  assert.equal(bindus.arudhaPadas.length, 12);

  const amshaLon = dhruv.amshaLongitude(100, 9, 0);
  assert.ok(Number.isFinite(amshaLon));
  const amshaRashi = dhruv.amshaRashiInfo(100, 9, 0);
  assert.ok(Number.isInteger(amshaRashi.rashiIndex));
  const amshaLons = dhruv.amshaLongitudes(100, [9, 10], [0, 0]);
  assert.equal(amshaLons.length, 2);
  const amshaScope = {
    includeBhavaCusps: true,
    includeArudhaPadas: true,
    includeUpagrahas: true,
    includeSphutas: true,
    includeSpecialLagnas: true,
    includeOuterPlanets: true,
  };
  const amshaChart = dhruv.amshaChartForDate(engine, eop, utc, loc, bhavaCfg, riseCfg, 0, true, 9, 0, amshaScope);
  assert.equal(amshaChart.grahas.length, 9);
  assert.equal(amshaChart.outerPlanets.length, 3);
  assert.equal(amshaChart.bhavaCusps.length, 12);
  assert.equal(amshaChart.arudhaPadas.length, 12);
  assert.equal(amshaChart.upagrahas.length, 11);
  assert.equal(amshaChart.sphutas.length, 16);
  assert.equal(amshaChart.specialLagnas.length, 8);

  const dashaCfg = dhruv.dashaSelectionConfigDefault();
  assert.equal(typeof dashaCfg.count, 'number');
  const dashaHierarchy = dhruv.dashaHierarchy(engine, eop, {
    birthUtc: utc,
    location: loc,
    ayanamshaSystem: 0,
    useNutation: true,
    system: 0,
    maxLevel: 1,
  });
  const levelCount = dashaHierarchy.levelCount();
  assert.ok(levelCount >= 0);
  if (levelCount > 0) {
    const firstLevelCount = dashaHierarchy.periodCount(0);
    assert.ok(firstLevelCount >= 0);
    if (firstLevelCount > 0) {
      const firstPeriod = dashaHierarchy.periodAt(0, 0);
      assert.ok(Number.isFinite(firstPeriod.startJd));
    }
  }
  dashaHierarchy.close();

  lsk.close();
  eop.close();
  engine.close();
});

test('v84 search: any-body sankranti, node bodies, multi-angle conjunction', { skip: !hasKernels() }, () => {
  const paths = kernelPaths();
  const engine = dhruv.Engine.create({
    spkPaths: [paths.spk],
    lskPath: paths.lsk,
    cacheCapacity: 64,
    strictValidation: false,
  });

  const janStart = { year: 2024, month: 1, day: 1, hour: 0, minute: 0, second: 0 };
  const janEnd = { year: 2024, month: 1, day: 31, hour: 0, minute: 0, second: 0 };

  const sankCfg = dhruv.sankrantiConfigDefault();
  assert.equal(sankCfg.nodeMode, 1);
  assert.equal(dhruv.conjunctionConfigDefault().nodeMode, 1);
  assert.equal(dhruv.stationaryConfigDefault().nodeMode, 1);

  // Moon rashi ingresses over one month: roughly one sidereal cycle.
  const moonSank = dhruv.sankrantiSearch(
    engine,
    {
      targetKind: 0,
      queryMode: 2,
      rashiIndex: 0,
      bodyCode: 301,
      startUtc: janStart,
      endUtc: janEnd,
    },
    16,
  );
  assert.ok(moonSank.events.length >= 12);
  for (const ev of moonSank.events) {
    assert.equal(ev.bodyCode, 301);
    assert.equal(typeof ev.isRetrograde, 'boolean');
    assert.equal(ev.sunSiderealLongitudeDeg, ev.siderealLongitudeDeg);
    assert.equal(ev.sunTropicalLongitudeDeg, ev.tropicalLongitudeDeg);
  }

  // Sun-Rahu conjunction with sidereal echoes.
  const sunRahu = dhruv.conjunctionSearch(
    engine,
    {
      body1Code: 10,
      body2Code: 10007,
      queryMode: 0,
      atUtc: { year: 2024, month: 3, day: 1, hour: 0, minute: 0, second: 0 },
      siderealConfig: sankCfg,
    },
    4,
  );
  assert.equal(sunRahu.found, true);
  assert.equal(sunRahu.event.body2Code, 10007);
  assert.equal(sunRahu.event.hasSidereal, true);
  assert.ok(sunRahu.event.body1RashiIndex >= 0 && sunRahu.event.body1RashiIndex < 12);
  assert.ok(sunRahu.event.body2RashiIndex >= 0 && sunRahu.event.body2RashiIndex < 12);

  // True-node Rahu stations roughly weekly.
  const rahuMotion = dhruv.motionSearch(
    engine,
    {
      bodyCode: 10007,
      motionKind: 0,
      queryMode: 2,
      startUtc: janStart,
      endUtc: janEnd,
    },
    16,
  );
  assert.ok(rahuMotion.stationaryEvents.length >= 1);
  assert.equal(rahuMotion.stationaryEvents[0].bodyCode, 10007);

  // Multi-angle sweep: new and full moon in one synodic month.
  const phases = dhruv.conjunctionSearch(
    engine,
    {
      body1Code: 10,
      body2Code: 301,
      queryMode: 2,
      startUtc: janStart,
      endUtc: { year: 2024, month: 1, day: 30, hour: 0, minute: 0, second: 0 },
      targetSeparationsDeg: [0, 180],
    },
    8,
  );
  assert.equal(phases.events.length, 2);
  for (const ev of phases.events) {
    assert.ok([0, 180].includes(ev.targetSeparationDeg));
  }

  engine.close();
});

test('bala wrappers accept amshaSelection and kundali returns resolved amsha union', { skip: !(hasKernels() && hasEop()) }, () => {
  const paths = kernelPaths();
  const engine = dhruv.Engine.create({
    spkPaths: [paths.spk],
    lskPath: paths.lsk,
    cacheCapacity: 64,
    strictValidation: false,
  });
  const eop = dhruv.EOP.load(paths.eop);
  const utc = {
    year: 2025,
    month: 1,
    day: 15,
    hour: 12,
    minute: 0,
    second: 0,
  };
  const loc = { latitudeDeg: 12.9716, longitudeDeg: 77.5946, altitudeM: 920 };
  const bhavaCfg = dhruv.bhavaConfigDefault();
  const riseCfg = dhruv.riseSetConfigDefault();
  const d2Variation = { count: 1, codes: [2], variations: [1] };
  const d9Default = { count: 1, codes: [9], variations: [0] };

  const shadbala = dhruv.shadbalaForDate(engine, eop, utc, loc, 0, true, bhavaCfg, riseCfg, d2Variation);
  assert.equal(shadbala.entries.length, 7);

  const vimsopaka = dhruv.vimsopakaForDate(engine, eop, utc, loc, 0, true, 0, d2Variation);
  assert.equal(vimsopaka.entries.length, 9);

  const balas = dhruv.balasForDate(engine, eop, utc, loc, bhavaCfg, riseCfg, 0, true, 0, d2Variation);
  assert.equal(balas.shadbala.entries.length, 7);
  assert.equal(balas.vimsopaka.entries.length, 9);

  const avastha = dhruv.avasthaForDate(engine, eop, utc, loc, bhavaCfg, riseCfg, 0, true, 0, d9Default);
  assert.equal(avastha.entries.length, 9);

  const fullCfg = dhruv.fullKundaliConfigDefault();
  fullCfg.includeAmshas = true;
  fullCfg.includeShadbala = true;
  fullCfg.includeVimsopaka = true;
  fullCfg.amshaSelection.count = 1;
  fullCfg.amshaSelection.codes[0] = 2;
  fullCfg.amshaSelection.variations[0] = 1;

  const kundali = dhruv.fullKundaliForDate(engine, eop, utc, loc, bhavaCfg, riseCfg, 0, true, fullCfg);
  assert.equal(kundali.amshas.length, 16);
  assert.equal(kundali.amshas[0].amshaCode, 2);
  assert.equal(kundali.amshas[0].variationCode, 1);
  assert.ok(kundali.amshas.some((chart) => chart.amshaCode === 60));

  eop.close();
  engine.close();
});

test('gochar events wrapper preserves named natal targets and optional charts', { skip: !(hasKernels() && hasEop()) }, () => {
  const paths = kernelPaths();
  const engine = dhruv.Engine.create({
    spkPaths: [paths.spk],
    lskPath: paths.lsk,
    cacheCapacity: 64,
    strictValidation: false,
  });
  const eop = dhruv.EOP.load(paths.eop);

  const birthUtc = { year: 1990, month: 1, day: 1, hour: 12, minute: 0, second: 0 };
  const atUtc = { year: 2025, month: 1, day: 15, hour: 12, minute: 0, second: 0 };
  const location = { latitudeDeg: 12.9716, longitudeDeg: 77.5946, altitudeM: 920 };
  const config = dhruv.gocharEventsConfigDefault();
  config.includeReturnCharts = true;

  const result = dhruv.gocharEvents(engine, eop, {
    birthUtc,
    atUtc,
    location,
    config,
    transitBodyCodes: [
      10,
      dhruv.GOCHAR_TRANSIT_BODY.RAHU,
      dhruv.GOCHAR_TRANSIT_BODY.KETU,
      799,
      899,
      999,
    ],
    natalTargets: [
      {
        kind: 5,
        index: 0,
        name: 'Zero Point',
        longitudeDeg: 0,
      },
    ],
  });

  assert.equal(result.birthUtc.year, 1990);
  assert.equal(result.yearlyTajaka.before.length, config.yearlyCount);
  assert.equal(result.yearlyTajaka.after.length, config.yearlyCount);
  assert.equal(result.monthlyTajaka.before.length, config.monthlyCount);
  assert.equal(result.monthlyTajaka.after.length, config.monthlyCount);
  assert.equal(result.yearlyTajaka.before[0].chart !== null, true);
  assert.equal(result.yearlyTithiPravesha.before[0].chart !== null, true);
  assert.ok(result.transitEvents.length > 0);
  assert.ok(result.transitEvents.some((event) => event.targetName === 'Zero Point'));

  eop.close();
  engine.close();
});

test('range operation cap constants match the C ABI', () => {
  assert.equal(dhruv.MAX_AMSHA_SERIES_CELLS, 100000);
  assert.equal(dhruv.MAX_PANCHANG_EVENTS, 50000);
  assert.equal(dhruv.MAX_AMSHA_LAGNA_SEGMENTS, 50000);
  assert.equal(dhruv.MAX_CHARAKARAKA_EVENTS, 50000);
});

test('range operations: amshaSeries, panchangEvents, amshaLagnaEvents', { skip: !(hasKernels() && hasEop()) }, () => {
  const paths = kernelPaths();
  const engine = dhruv.Engine.create({
    spkPaths: [paths.spk],
    lskPath: paths.lsk,
    cacheCapacity: 64,
    strictValidation: false,
  });
  const eop = dhruv.EOP.load(paths.eop);
  const loc = { latitudeDeg: 12.9716, longitudeDeg: 77.5946, altitudeM: 920 };
  const utc = { year: 2025, month: 1, day: 15, hour: 12, minute: 0, second: 0 };
  const sankCfg = dhruv.sankrantiConfigDefault();
  const bhavaCfg = dhruv.bhavaConfigDefault();
  const riseCfg = dhruv.riseSetConfigDefault();
  const angularSep = (a, b) => {
    const d = Math.abs(a - b) % 360;
    return Math.min(d, 360 - d);
  };

  // --- amshaSeries: 2h at 60-minute cadence yields 3 points; first point
  // matches the single-epoch amsha chart.
  const seriesTo = { ...utc, hour: utc.hour + 2 };
  const series = dhruv.amshaSeries(engine, eop, utc, seriesTo, 60, loc, [9, 1, 9], null, true);
  assert.equal(series.length, 3);
  assert.equal(series[0].charts.length, 3);
  assert.ok(Number.isFinite(series[0].jdUtc));
  assert.equal(series[0].utc.year, utc.year);
  const d9 = series[0].charts[0];
  assert.equal(d9.amshaCode, 9);
  assert.equal(d9.variationCode, 0);
  assert.equal(series[0].charts[1].amshaCode, 1);
  // Duplicate D9 request repeats the same chart.
  assert.equal(series[0].charts[2].amshaCode, 9);
  assert.equal(d9.lagna.siderealLongitude, series[0].charts[2].lagna.siderealLongitude);
  assert.equal(d9.grahas.length, 9);
  const amshaScope = {
    includeBhavaCusps: false,
    includeArudhaPadas: false,
    includeUpagrahas: false,
    includeSphutas: false,
    includeSpecialLagnas: false,
    includeOuterPlanets: false,
  };
  const single = dhruv.amshaChartForDate(
    engine, eop, utc, loc, bhavaCfg, riseCfg,
    sankCfg.ayanamshaSystem, sankCfg.useNutation, 9, 0, amshaScope,
  );
  assert.ok(angularSep(d9.lagna.siderealLongitude, single.lagna.siderealLongitude) < 1e-6);
  for (let g = 0; g < 9; g += 1) {
    assert.ok(angularSep(d9.grahas[g].siderealLongitude, single.grahas[g].siderealLongitude) < 1e-6);
  }
  // Lagna-only series omits graha entries.
  const slim = dhruv.amshaSeries(engine, eop, utc, seriesTo, 60, loc, [9], null, false);
  assert.equal(slim[0].charts[0].grahas, null);

  // amshaSeries rejections: zero step, empty request list.
  assert.throws(() => dhruv.amshaSeries(engine, eop, utc, seriesTo, 0, loc, [9]));
  assert.throws(() => dhruv.amshaSeries(engine, eop, utc, seriesTo, 60, loc, []));

  // --- panchangEvents: tithi segments over 7 days chain exactly.
  const weekTo = { ...utc, day: utc.day + 7 };
  const events = dhruv.panchangEvents(engine, eop, utc, weekTo, dhruv.PANCHANG_INCLUDE.TITHI);
  assert.ok(events.tithis.length >= 6);
  assert.equal(events.karanas.length, 0);
  assert.equal(events.vaars.length, 0);
  assert.equal(events.horas.length, 0);
  assert.equal(events.ghatikas.length, 0);
  assert.equal(events.truncated, false);
  assert.equal(events.nextFromUtc, null);
  for (let i = 0; i + 1 < events.tithis.length; i += 1) {
    assert.deepEqual(events.tithis[i].end, events.tithis[i + 1].start);
    assert.ok(Number.isInteger(events.tithis[i].tithiIndex));
  }

  // Truncation and resume: dedup on segment start reproduces the full sweep.
  const truncated = dhruv.panchangEvents(
    engine, eop, utc, weekTo, dhruv.PANCHANG_INCLUDE.TITHI, sankCfg, 3,
  );
  assert.equal(truncated.truncated, true);
  assert.equal(truncated.tithis.length, 3);
  assert.ok(truncated.nextFromUtc !== null);
  const resumed = dhruv.panchangEvents(
    engine, eop, truncated.nextFromUtc, weekTo, dhruv.PANCHANG_INCLUDE.TITHI, sankCfg, 0,
  );
  // Resumed sweeps re-solve boundaries, so allow sub-second drift when
  // deduplicating and comparing against the untruncated sweep.
  const utcMs = (t) => Date.UTC(t.year, t.month - 1, t.day, t.hour, t.minute, 0) + t.second * 1000;
  const seenMs = truncated.tithis.map((t) => utcMs(t.start));
  const merged = truncated.tithis.concat(
    resumed.tithis.filter((t) => !seenMs.some((ms) => Math.abs(ms - utcMs(t.start)) < 500)),
  );
  assert.equal(merged.length, events.tithis.length);
  for (let i = 0; i < merged.length; i += 1) {
    assert.ok(Math.abs(utcMs(merged[i].start) - utcMs(events.tithis[i].start)) < 500);
    assert.equal(merged[i].tithiIndex, events.tithis[i].tithiIndex);
  }

  // --- panchangEvents with a location: vaar/hora segments over 3 days.
  const threeDaysTo = { ...utc, day: utc.day + 3 };
  const located = dhruv.panchangEvents(
    engine, eop, utc, threeDaysTo,
    dhruv.PANCHANG_INCLUDE.VAAR | dhruv.PANCHANG_INCLUDE.HORA,
    sankCfg, 0, loc,
  );
  // 3 days cover 3-5 sunrise-to-sunrise Vedic days and 72-120 horas.
  assert.ok(located.vaars.length >= 3 && located.vaars.length <= 5,
    `vaar count ${located.vaars.length}`);
  assert.ok(located.horas.length >= 72 && located.horas.length <= 120,
    `hora count ${located.horas.length}`);
  assert.equal(located.ghatikas.length, 0);
  assert.equal(located.tithis.length, 0);
  // Vaar segments chain exactly and advance the weekday cyclically.
  for (let i = 0; i + 1 < located.vaars.length; i += 1) {
    assert.deepEqual(located.vaars[i].end, located.vaars[i + 1].start);
    assert.equal(located.vaars[i + 1].vaarIndex, (located.vaars[i].vaarIndex + 1) % 7);
  }
  // Hora segments chain exactly, including across Vedic-day rolls; the lord
  // cycles through the Chaldean sequence and the position cycles 0..23.
  for (let i = 0; i + 1 < located.horas.length; i += 1) {
    assert.deepEqual(located.horas[i].end, located.horas[i + 1].start);
    assert.equal(located.horas[i + 1].horaIndex, (located.horas[i].horaIndex + 1) % 7);
    assert.equal(located.horas[i + 1].horaPosition, (located.horas[i].horaPosition + 1) % 24);
  }
  // An explicit rise/set config equal to the defaults reproduces the sweep.
  const locatedExplicit = dhruv.panchangEvents(
    engine, eop, utc, threeDaysTo,
    dhruv.PANCHANG_INCLUDE.VAAR | dhruv.PANCHANG_INCLUDE.HORA,
    sankCfg, 0, loc, riseCfg,
  );
  assert.equal(locatedExplicit.vaars.length, located.vaars.length);
  assert.equal(locatedExplicit.horas.length, located.horas.length);
  // Ghatikas chain with values cycling 1..60.
  const dayTo = { ...utc, day: utc.day + 1 };
  const gh = dhruv.panchangEvents(
    engine, eop, utc, dayTo, dhruv.PANCHANG_INCLUDE.GHATIKA, sankCfg, 0, loc,
  );
  assert.ok(gh.ghatikas.length >= 60 && gh.ghatikas.length <= 120,
    `ghatika count ${gh.ghatikas.length}`);
  for (let i = 0; i + 1 < gh.ghatikas.length; i += 1) {
    assert.deepEqual(gh.ghatikas[i].end, gh.ghatikas[i + 1].start);
    assert.equal(gh.ghatikas[i + 1].value, (gh.ghatikas[i].value % 60) + 1);
  }

  // panchangEvents rejections: zero mask, location-dependent bits without a
  // location, unknown bit.
  assert.throws(() => dhruv.panchangEvents(engine, eop, utc, weekTo, 0));
  assert.throws(() => dhruv.panchangEvents(engine, eop, utc, weekTo, dhruv.PANCHANG_INCLUDE.VAAR));
  assert.throws(() => dhruv.panchangEvents(
    engine, eop, utc, weekTo, dhruv.PANCHANG_INCLUDE.TITHI | dhruv.PANCHANG_INCLUDE.HORA,
  ));
  assert.throws(() => dhruv.panchangEvents(engine, eop, utc, weekTo, 1 << 20));

  // --- amshaLagnaEvents: D1 lagna segments over 6h chain exactly and
  // advance one rashi at a time.
  const lagnaTo = { ...utc, hour: utc.hour + 6 };
  const lagnaEvents = dhruv.amshaLagnaEvents(engine, eop, utc, lagnaTo, loc, [1, 1]);
  assert.equal(lagnaEvents.entries.length, 1, 'duplicate requests collapse');
  const entry = lagnaEvents.entries[0];
  assert.equal(entry.amshaCode, 1);
  assert.equal(entry.variationCode, 0);
  assert.ok(entry.segments.length >= 3, '6h of D1 lagna spans >= 3 rashis');
  assert.deepEqual(entry.segments[0].start, utc, 'first segment starts at fromUtc');
  for (const seg of entry.segments) {
    assert.ok(seg.rashiIndex >= 0 && seg.rashiIndex <= 11);
  }
  for (let i = 0; i + 1 < entry.segments.length; i += 1) {
    assert.deepEqual(entry.segments[i].end, entry.segments[i + 1].start);
    assert.equal((entry.segments[i].rashiIndex + 1) % 12, entry.segments[i + 1].rashiIndex);
  }
  assert.equal(lagnaEvents.truncated, false);
  assert.equal(lagnaEvents.nextFromUtc, null);

  // amshaLagnaEvents rejections: empty request list, invalid amsha code.
  assert.throws(() => dhruv.amshaLagnaEvents(engine, eop, utc, lagnaTo, loc, []));
  assert.throws(() => dhruv.amshaLagnaEvents(engine, eop, utc, lagnaTo, loc, [65535]));

  eop.close();
  engine.close();
});

test('charakaraka ranking-change events', { skip: !(hasKernels() && hasEop()) }, () => {
  const paths = kernelPaths();
  const engine = dhruv.Engine.create({
    spkPaths: [paths.spk],
    lskPath: paths.lsk,
    cacheCapacity: 64,
    strictValidation: false,
  });
  const eop = dhruv.EOP.load(paths.eop);
  const from = { year: 2025, month: 1, day: 15, hour: 0, minute: 0, second: 0 };
  const to = { year: 2025, month: 1, day: 19, hour: 0, minute: 0, second: 0 };
  const utcMs = (t) => Date.UTC(t.year, t.month - 1, t.day, t.hour, t.minute, 0) + t.second * 1000;

  // --- Range sweep (scheme eight over a few days): non-empty, ascending,
  // every event carries at least one changed role and per-moment rankings.
  const sweep = dhruv.charakarakaEvents(engine, eop, from, to, { scheme: 'eight' });
  assert.ok(sweep.events.length > 0, 'four days of eight-scheme changes');
  assert.equal(sweep.truncated, false);
  assert.equal(sweep.nextFromUtc, null);
  for (const ev of sweep.events) {
    assert.ok(utcMs(ev.at) >= utcMs(from) && utcMs(ev.at) <= utcMs(to));
    assert.ok(Number.isFinite(ev.jdTdb));
    assert.ok(ev.trigger >= 0 && ev.trigger <= 2);
    assert.equal(typeof ev.triggerName, 'string');
    assert.ok(Array.isArray(ev.changedRoles));
    assert.ok(ev.changedRoles.length > 0, 'every event changes a role');
    for (const role of ev.changedRoles) {
      assert.ok(role >= dhruv.CHARAKARAKA_ROLE.ATMA && role <= dhruv.CHARAKARAKA_ROLE.MATRI_PUTRA);
    }
    // before/after reuse the charakarakaForDate result shape.
    for (const side of [ev.before, ev.after]) {
      assert.equal(side.scheme, dhruv.CHARAKARAKA_SCHEME.EIGHT);
      assert.equal(side.count, 8);
      assert.equal(side.entries.length, 8);
      assert.ok(Number.isFinite(side.entries[0].effectiveDegreesInRashi));
    }
  }
  for (let i = 0; i + 1 < sweep.events.length; i += 1) {
    assert.ok(utcMs(sweep.events[i].at) < utcMs(sweep.events[i + 1].at), 'events ascend');
  }

  // --- Truncation: maxEvents caps the sweep and yields a resume point.
  assert.ok(sweep.events.length > 3, 'enough events to exercise truncation');
  const truncated = dhruv.charakarakaEvents(engine, eop, from, to, {
    scheme: dhruv.CHARAKARAKA_SCHEME.EIGHT,
    maxEvents: 3,
  });
  assert.equal(truncated.truncated, true);
  assert.equal(truncated.events.length, 3);
  assert.ok(truncated.nextFromUtc !== null);

  // --- next/prev point lookups agree with the sweep edges (re-solved
  // boundaries, so allow sub-second drift).
  const next = dhruv.nextCharakarakaEvent(engine, eop, from, { scheme: 'eight' });
  assert.ok(next !== null);
  assert.ok(Math.abs(utcMs(next.at) - utcMs(sweep.events[0].at)) < 1500);
  assert.ok(next.changedRoles.length > 0);
  const prev = dhruv.prevCharakarakaEvent(engine, eop, to, { scheme: 'eight' });
  assert.ok(prev !== null);
  assert.ok(Math.abs(utcMs(prev.at) - utcMs(sweep.events[sweep.events.length - 1].at)) < 1500);
  assert.ok(prev.changedRoles.length > 0);

  eop.close();
  engine.close();
});
