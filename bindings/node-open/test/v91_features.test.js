'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');

const dhruv = require('..');
const { hasKernels, hasEop, kernelPaths } = require('./helpers');

const NATAL_RASHIS = [0, 1, 2, 3, 4, 5, 6];
const NATAL_LAGNA = 0;

test('tara bala pure function and names', () => {
  assert.equal(dhruv.taraName(0), 'Janma');
  assert.equal(dhruv.taraName(1), 'Sampat');
  assert.equal(dhruv.taraName(8), 'Parama Mitra');
  assert.equal(dhruv.taraName(9), null);
  assert.equal(dhruv.TARA_BALA_REFERENCE_CHART_MOON, 255);

  // Same nakshatra: Janma, count 1, first cycle, unfavourable.
  const janma = dhruv.taraBala(0, 0);
  assert.deepEqual(janma, { taraIndex: dhruv.TARA.JANMA, count: 1, cycle: 1, favorable: false });
  // Next nakshatra: Sampat, favourable.
  const sampat = dhruv.taraBala(0, 1);
  assert.equal(sampat.taraIndex, dhruv.TARA.SAMPAT);
  assert.equal(sampat.count, 2);
  assert.equal(sampat.favorable, true);
  // Wraps modulo 27 and cycles modulo 9.
  const wrapped = dhruv.taraBala(26, 0);
  assert.equal(wrapped.count, 2);
  assert.equal(wrapped.taraIndex, dhruv.TARA.SAMPAT);
  const second = dhruv.taraBala(0, 10);
  assert.equal(second.count, 11);
  assert.equal(second.cycle, 2);
  assert.equal(second.taraIndex, dhruv.TARA.SAMPAT);
  const third = dhruv.taraBala(0, 26);
  assert.equal(third.count, 27);
  assert.equal(third.cycle, 3);
  assert.equal(third.taraIndex, dhruv.TARA.PARAMA_MITRA);
  assert.equal(third.favorable, true);
  // Favourable set across one cycle: Sampat, Kshema, Sadhaka, Mitra, Parama Mitra.
  const favorable = [];
  for (let t = 0; t < 9; t += 1) {
    if (dhruv.taraBala(0, t).favorable) favorable.push(t);
  }
  assert.deepEqual(favorable, [1, 3, 5, 7, 8]);

  assert.throws(() => dhruv.taraBala(27, 0), (err) => err.status === dhruv.STATUS.INVALID_INPUT);
  assert.throws(() => dhruv.taraBala(0, 27), (err) => err.status === dhruv.STATUS.INVALID_INPUT);
});

test('kakshya helpers', () => {
  // Eighths of a rashi: 3.75 degrees each.
  assert.equal(dhruv.kakshyaIndex(0), 0);
  assert.equal(dhruv.kakshyaIndex(3.74), 0);
  assert.equal(dhruv.kakshyaIndex(3.75), 1);
  assert.equal(dhruv.kakshyaIndex(29.99), 7);
  assert.equal(dhruv.kakshyaIndex(30), 0);
  assert.equal(dhruv.kakshyaIndex(359.9), 7);
  assert.equal(dhruv.kakshyaIndex(Number.NaN), -1);
  // Lords: Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Lagna.
  assert.deepEqual([0, 1, 2, 3, 4, 5, 6, 7].map((k) => dhruv.kakshyaLord(k)), [6, 4, 2, 0, 5, 3, 1, 7]);
  assert.equal(dhruv.kakshyaLord(8), -1);

  // Bindu agrees with the BAV contributor table.
  const bav = dhruv.calculateBav(0, NATAL_RASHIS, NATAL_LAGNA);
  for (const lon of [0, 10, 47.5, 123.4, 200, 299.9, 355]) {
    const rashi = Math.floor(lon / 30);
    const lord = dhruv.kakshyaLord(dhruv.kakshyaIndex(lon));
    const expected = bav.contributors[rashi][lord] !== 0;
    assert.equal(dhruv.kakshyaBindu(bav, lon), expected, `lon ${lon}`);
  }
  assert.throws(() => dhruv.kakshyaBindu(bav, Number.NaN), (err) => err.status === dhruv.STATUS.INVALID_INPUT);
});

test('transit ashtakavarga pure function', () => {
  const natal = dhruv.calculateAshtakavarga(NATAL_RASHIS, NATAL_LAGNA);
  const lons = [5, 40, 75, 110, 145, 180, 215];
  const entries = dhruv.transitAshtakavarga(natal, lons);
  assert.equal(entries.length, 7);
  for (let i = 0; i < 7; i += 1) {
    const e = entries[i];
    const rashi = Math.floor(lons[i] / 30);
    assert.equal(e.grahaIndex, i);
    assert.equal(e.rashiIndex, rashi);
    assert.equal(e.bavBindus, natal.bavs[i].points[rashi]);
    assert.equal(e.savTotal, natal.sav.totalPoints[rashi]);
    assert.equal(e.savAfterTrikona, natal.sav.afterTrikona[rashi]);
    assert.equal(e.savAfterEkadhipatya, natal.sav.afterEkadhipatya[rashi]);
    assert.equal(e.kakshyaIndex, dhruv.kakshyaIndex(lons[i]));
    assert.equal(e.kakshyaLord, dhruv.kakshyaLord(e.kakshyaIndex));
    assert.equal(e.kakshyaBindu, dhruv.kakshyaBindu(natal.bavs[i], lons[i]));
  }
  assert.throws(() => dhruv.transitAshtakavarga(natal, [0, 1, 2]));
});

test('samvatsara office names', () => {
  assert.equal(dhruv.samvatsaraOfficeName(dhruv.SAMVATSARA_OFFICE.RAJA), 'Raja');
  assert.equal(dhruv.samvatsaraOfficeName(dhruv.SAMVATSARA_OFFICE.MANTRI), 'Mantri');
  assert.equal(dhruv.samvatsaraOfficeName(dhruv.SAMVATSARA_OFFICE.DURGESHA), 'Durgesha');
  assert.equal(dhruv.samvatsaraOfficeName(9), null);
});

test('full kundali config default carries the v91 sections', () => {
  const cfg = dhruv.fullKundaliConfigDefault();
  assert.equal(cfg.includeTaraBala, false);
  assert.equal(cfg.taraBalaReferenceNakshatraIndex, dhruv.TARA_BALA_REFERENCE_CHART_MOON);
  assert.equal(cfg.includeSamvatsaraOfficers, false);
  assert.equal(cfg.includeTransitAshtakavarga, false);
  assert.deepEqual(cfg.transitAshtakavargaReference, { grahaRashis: [0, 0, 0, 0, 0, 0, 0], lagnaRashi: 0 });
  assert.equal(cfg.grahaPositionsConfig.includeSpeeds, false);
});

function openEngine() {
  const paths = kernelPaths();
  const engine = dhruv.Engine.create({
    spkPaths: [paths.spk],
    lskPath: paths.lsk,
    cacheCapacity: 64,
    strictValidation: false,
  });
  const eop = dhruv.EOP.load(paths.eop);
  return { engine, eop };
}

const UTC = { year: 2025, month: 1, day: 15, hour: 12, minute: 0, second: 0 };
const LOC = { latitudeDeg: 12.9716, longitudeDeg: 77.5946, altitudeM: 920 };

test('graha position speeds and outer-planet retrograde', { skip: !(hasKernels() && hasEop()) }, () => {
  const { engine, eop } = openEngine();
  const bhavaCfg = dhruv.bhavaConfigDefault();
  const base = { includeNakshatra: true, includeLagna: true, includeOuterPlanets: true, includeBhava: true };

  const plain = dhruv.grahaPositionsForDate(engine, eop, UTC, LOC, bhavaCfg, 0, true, base);
  for (const entry of [...plain.grahas, plain.lagna, ...plain.outerPlanets]) {
    assert.equal(entry.speedValid, false);
    assert.equal(entry.longitudeSpeedDegPerDay, undefined);
  }

  const withSpeeds = dhruv.grahaPositionsForDate(engine, eop, UTC, LOC, bhavaCfg, 0, true, {
    ...base,
    includeSpeeds: true,
    basicStatesConfig: { includeBasicStates: true, includeSensitivePointDistances: false },
  });
  for (const entry of [...withSpeeds.grahas, withSpeeds.lagna, ...withSpeeds.outerPlanets]) {
    assert.equal(entry.speedValid, true);
    assert.ok(Number.isFinite(entry.longitudeSpeedDegPerDay));
    assert.ok(Number.isFinite(entry.latitudeSpeedDegPerDay));
  }
  // Sun and Moon always move direct; the Moon covers roughly 11-15 deg/day.
  assert.ok(withSpeeds.grahas[0].longitudeSpeedDegPerDay > 0.9 && withSpeeds.grahas[0].longitudeSpeedDegPerDay < 1.1);
  assert.ok(withSpeeds.grahas[1].longitudeSpeedDegPerDay > 11 && withSpeeds.grahas[1].longitudeSpeedDegPerDay < 15.5);
  // Mean-node Rahu/Ketu and the lagna are ecliptic points: latitude rate 0.
  assert.equal(withSpeeds.grahas[7].latitudeSpeedDegPerDay, 0);
  assert.equal(withSpeeds.grahas[8].latitudeSpeedDegPerDay, 0);
  assert.equal(withSpeeds.lagna.latitudeSpeedDegPerDay, 0);
  // Rahu and Ketu share the same rate; the lagna advances ~360 deg/day.
  assert.ok(Math.abs(withSpeeds.grahas[7].longitudeSpeedDegPerDay - withSpeeds.grahas[8].longitudeSpeedDegPerDay) < 1e-9);
  assert.ok(withSpeeds.lagna.longitudeSpeedDegPerDay > 300);
  // Retrograde flag agrees with the sign of the longitude rate for every
  // graha and outer planet with basic states.
  for (const entry of [...withSpeeds.grahas.slice(0, 7), ...withSpeeds.outerPlanets]) {
    assert.equal(entry.basicStatesValid, true);
    assert.equal(entry.basicStates.retrograde, entry.longitudeSpeedDegPerDay < 0);
  }
  // Mid-January 2025: Mars is retrograde (Dec 2024 - Feb 2025).
  assert.equal(withSpeeds.grahas[2].basicStates.retrograde, true);
  assert.ok(withSpeeds.grahas[2].longitudeSpeedDegPerDay < 0);

  // Also reachable through the full kundali graha positions config.
  const fullCfg = dhruv.fullKundaliConfigDefault();
  fullCfg.grahaPositionsConfig.includeSpeeds = true;
  const kundali = dhruv.fullKundaliForDate(engine, eop, UTC, LOC, bhavaCfg, dhruv.riseSetConfigDefault(), 0, true, fullCfg);
  assert.equal(kundali.grahaPositions.grahas[1].speedValid, true);
  assert.ok(Math.abs(kundali.grahaPositions.grahas[1].longitudeSpeedDegPerDay - withSpeeds.grahas[1].longitudeSpeedDegPerDay) < 1e-9);

  eop.close();
  engine.close();
});

test('amsha entries carry vargottama', { skip: !(hasKernels() && hasEop()) }, () => {
  const { engine, eop } = openEngine();
  const bhavaCfg = dhruv.bhavaConfigDefault();
  const riseCfg = dhruv.riseSetConfigDefault();
  const scope = {
    includeBhavaCusps: false,
    includeArudhaPadas: false,
    includeUpagrahas: false,
    includeSphutas: false,
    includeSpecialLagnas: false,
    includeOuterPlanets: false,
  };
  const d1 = dhruv.amshaChartForDate(engine, eop, UTC, LOC, bhavaCfg, riseCfg, 0, true, 1, 0, scope);
  const d9 = dhruv.amshaChartForDate(engine, eop, UTC, LOC, bhavaCfg, riseCfg, 0, true, 9, 0, scope);
  // Never vargottama on D1 itself.
  assert.equal(d1.lagna.vargottama, false);
  for (const g of d1.grahas) assert.equal(g.vargottama, false);
  // On D9: true exactly when the navamsha rashi equals the D1 rashi.
  assert.equal(d9.lagna.vargottama, d9.lagna.rashiIndex === d1.lagna.rashiIndex);
  for (let i = 0; i < 9; i += 1) {
    assert.equal(typeof d9.grahas[i].vargottama, 'boolean');
    assert.equal(d9.grahas[i].vargottama, d9.grahas[i].rashiIndex === d1.grahas[i].rashiIndex);
  }
  eop.close();
  engine.close();
});

test('panchang infos report progress', { skip: !(hasKernels() && hasEop()) }, () => {
  const { engine, eop } = openEngine();
  const lsk = dhruv.LSK.load(kernelPaths().lsk);
  const riseCfg = dhruv.riseSetConfigDefault();
  const sankCfg = dhruv.sankrantiConfigDefault();
  const inUnit = (p) => Number.isFinite(p) && p >= 0 && p <= 1;

  assert.ok(inUnit(dhruv.tithiForDate(engine, UTC).progress));
  assert.ok(inUnit(dhruv.karanaForDate(engine, UTC).progress));
  assert.ok(inUnit(dhruv.yogaForDate(engine, UTC, sankCfg).progress));
  assert.ok(inUnit(dhruv.nakshatraForDate(engine, UTC, sankCfg).progress));
  assert.ok(inUnit(dhruv.vaarForDate(engine, eop, UTC, LOC, riseCfg).progress));
  assert.ok(inUnit(dhruv.horaForDate(engine, eop, UTC, LOC, riseCfg).progress));
  assert.ok(inUnit(dhruv.ghatikaForDate(engine, eop, UTC, LOC, riseCfg).progress));
  assert.ok(inUnit(dhruv.masaForDate(engine, UTC, sankCfg).progress));
  assert.ok(inUnit(dhruv.ayanaForDate(engine, UTC, sankCfg).progress));
  assert.ok(inUnit(dhruv.varshaForDate(engine, UTC, sankCfg).progress));

  // Progress is the elapsed fraction of [start, end) at the query moment.
  const tithi = dhruv.tithiForDate(engine, UTC);
  const utcMs = (t) => Date.UTC(t.year, t.month - 1, t.day, t.hour, t.minute, 0) + t.second * 1000;
  const expected = (utcMs(UTC) - utcMs(tithi.start)) / (utcMs(tithi.end) - utcMs(tithi.start));
  assert.ok(Math.abs(tithi.progress - expected) < 1e-3);
  // Later in the same tithi: progress grows.
  const later = dhruv.tithiForDate(engine, { ...UTC, hour: UTC.hour + 1 });
  if (later.tithiIndex === tithi.tithiIndex) assert.ok(later.progress > tithi.progress);

  // Unified panchang and the full kundali section carry it as well.
  const panchang = dhruv.panchangComputeEx(engine, eop, lsk, {
    timeKind: 1,
    jdTdb: -1,
    utc: UTC,
    location: LOC,
    includeMask: dhruv.PANCHANG_INCLUDE.ALL,
  });
  for (const key of ['tithi', 'karana', 'yoga', 'vaar', 'hora', 'ghatika', 'nakshatra', 'masa', 'ayana', 'varsha']) {
    assert.ok(inUnit(panchang[key].progress), key);
  }
  assert.ok(Math.abs(panchang.tithi.progress - tithi.progress) < 1e-9);

  // Range listings report 0.
  const events = dhruv.panchangEvents(engine, eop, UTC, { ...UTC, day: UTC.day + 2 }, dhruv.PANCHANG_INCLUDE.TITHI);
  for (const seg of events.tithis) assert.equal(seg.progress, 0);

  lsk.close();
  eop.close();
  engine.close();
});

test('dasha snapshot reports progress', { skip: !(hasKernels() && hasEop()) }, () => {
  const { engine, eop } = openEngine();
  const birthUtc = { year: 1990, month: 1, day: 1, hour: 12, minute: 0, second: 0 };
  const snapshot = dhruv.dashaSnapshot(engine, eop, {
    birthUtc,
    queryUtc: UTC,
    location: LOC,
    ayanamshaSystem: 0,
    useNutation: true,
    system: 0,
    maxLevel: 3,
  });
  assert.ok(snapshot.count > 0);
  assert.equal(snapshot.progress.length, snapshot.count);
  for (let i = 0; i < snapshot.count; i += 1) {
    const p = snapshot.periods[i];
    const expected = (snapshot.queryJd - p.startJd) / (p.endJd - p.startJd);
    assert.ok(snapshot.progress[i] >= 0 && snapshot.progress[i] <= 1);
    assert.ok(Math.abs(snapshot.progress[i] - expected) < 1e-9);
  }

  // Embedded kundali dasha snapshots carry the same parallel array.
  const fullCfg = dhruv.fullKundaliConfigDefault();
  fullCfg.includeDasha = true;
  fullCfg.dashaConfig.count = 1;
  fullCfg.dashaConfig.systems[0] = 0;
  fullCfg.dashaConfig.maxLevels[0] = 1;
  fullCfg.dashaConfig.snapshotUtc = UTC;
  const kundali = dhruv.fullKundaliForDate(
    engine, eop, birthUtc, LOC, dhruv.bhavaConfigDefault(), dhruv.riseSetConfigDefault(), 0, true, fullCfg,
  );
  assert.ok(Array.isArray(kundali.dashaSnapshots));
  for (const snap of kundali.dashaSnapshots) {
    assert.equal(snap.progress.length, snap.count);
    for (const p of snap.progress) assert.ok(p >= 0 && p <= 1);
  }

  eop.close();
  engine.close();
});

test('tara bala, transit ashtakavarga, samvatsara officers for date and in kundali', { skip: !(hasKernels() && hasEop()) }, () => {
  const { engine, eop } = openEngine();
  const bhavaCfg = dhruv.bhavaConfigDefault();
  const riseCfg = dhruv.riseSetConfigDefault();

  // --- tara bala against the chart's own Moon.
  const tara = dhruv.taraBalaForDate(engine, eop, UTC, LOC);
  assert.equal(tara.referenceIsChartMoon, true);
  assert.equal(tara.grahas.length, 9);
  assert.equal(tara.referenceNakshatraIndex, tara.grahas[1].nakshatraIndex);
  assert.equal(tara.grahas[1].taraBala.taraIndex, dhruv.TARA.JANMA);
  assert.equal(tara.grahas[1].taraBala.count, 1);
  for (const entry of [...tara.grahas, tara.lagna]) {
    assert.ok(entry.nakshatraIndex >= 0 && entry.nakshatraIndex < 27);
    assert.deepEqual(entry.taraBala, dhruv.taraBala(tara.referenceNakshatraIndex, entry.nakshatraIndex));
  }
  // Explicit reference.
  const taraRef = dhruv.taraBalaForDate(engine, eop, UTC, LOC, 0, true, 5);
  assert.equal(taraRef.referenceIsChartMoon, false);
  assert.equal(taraRef.referenceNakshatraIndex, 5);
  assert.deepEqual(taraRef.lagna.taraBala, dhruv.taraBala(5, taraRef.lagna.nakshatraIndex));
  assert.throws(() => dhruv.taraBalaForDate(engine, eop, UTC, LOC, 0, true, 27));

  // --- transit ashtakavarga for date agrees with the pure function.
  const reference = { grahaRashis: NATAL_RASHIS, lagnaRashi: NATAL_LAGNA };
  const transit = dhruv.transitAshtakavargaForDate(engine, eop, UTC, 0, true, reference);
  assert.deepEqual(transit.reference, reference);
  assert.equal(transit.entries.length, 7);
  const positions = dhruv.grahaPositionsForDate(engine, eop, UTC, LOC, bhavaCfg, 0, true, {
    includeNakshatra: false, includeLagna: false, includeOuterPlanets: false, includeBhava: false,
  });
  const natal = dhruv.calculateAshtakavarga(NATAL_RASHIS, NATAL_LAGNA);
  const pure = dhruv.transitAshtakavarga(natal, positions.grahas.slice(0, 7).map((g) => g.siderealLongitude));
  assert.deepEqual(transit.entries, pure);

  // --- samvatsara officers.
  const officers = dhruv.samvatsaraOfficers(engine, eop, UTC, LOC);
  assert.ok(Number.isInteger(officers.varsha.samvatsaraIndex));
  assert.equal(officers.officers.length, 9);
  const varshaStartMs = Date.UTC(officers.varsha.start.year, officers.varsha.start.month - 1, officers.varsha.start.day);
  for (let i = 0; i < 9; i += 1) {
    const o = officers.officers[i];
    assert.equal(o.officeIndex, i);
    assert.equal(o.officeName, dhruv.samvatsaraOfficeName(i));
    assert.ok(o.vaarIndex >= 0 && o.vaarIndex < 7);
    assert.ok(o.lordIndex >= 0 && o.lordIndex < 7);
    assert.equal(o.lordIndex, dhruv.horaLord(o.vaarIndex, 0));
    const eventMs = Date.UTC(o.eventUtc.year, o.eventUtc.month - 1, o.eventUtc.day);
    assert.ok(eventMs >= varshaStartMs - 86400000, `office ${i} event inside the year`);
    assert.ok(Number.isInteger(o.vedicDayStart.year));
  }
  // Raja is the Chaitra Shukla Pratipada day: its event opens the year.
  assert.equal(officers.officers[0].eventUtc.year, officers.varsha.start.year);
  // Explicit rise/set config equal to the defaults reproduces the result.
  const officersExplicit = dhruv.samvatsaraOfficers(engine, eop, UTC, LOC, 0, true, riseCfg);
  assert.deepEqual(officersExplicit, officers);

  // --- full kundali sections: absent by default, present when requested.
  const offCfg = dhruv.fullKundaliConfigDefault();
  const off = dhruv.fullKundaliForDate(engine, eop, UTC, LOC, bhavaCfg, riseCfg, 0, true, offCfg);
  assert.equal(off.taraBala, undefined);
  assert.equal(off.samvatsaraOfficers, undefined);
  assert.equal(off.transitAshtakavarga, undefined);

  const onCfg = dhruv.fullKundaliConfigDefault();
  onCfg.includeTaraBala = true;
  onCfg.includeSamvatsaraOfficers = true;
  onCfg.includeTransitAshtakavarga = true;
  onCfg.transitAshtakavargaReference = reference;
  const on = dhruv.fullKundaliForDate(engine, eop, UTC, LOC, bhavaCfg, riseCfg, 0, true, onCfg);
  assert.deepEqual(on.taraBala, tara);
  assert.deepEqual(on.samvatsaraOfficers, officers);
  assert.deepEqual(on.transitAshtakavarga, transit);

  const refCfg = dhruv.fullKundaliConfigDefault();
  refCfg.includeTaraBala = true;
  refCfg.taraBalaReferenceNakshatraIndex = 5;
  const withRef = dhruv.fullKundaliForDate(engine, eop, UTC, LOC, bhavaCfg, riseCfg, 0, true, refCfg);
  assert.deepEqual(withRef.taraBala, taraRef);

  eop.close();
  engine.close();
});
