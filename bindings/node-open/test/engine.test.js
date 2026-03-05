'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');

const dhruv = require('..');
const { hasKernels, hasEop, kernelPaths } = require('./helpers');

test('api version matches expected ABI', () => {
  assert.equal(dhruv.apiVersion(), dhruv.EXPECTED_API_VERSION);
  assert.doesNotThrow(() => dhruv.verifyAbi());
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

  assert.ok(Number.isFinite(state.positionKm[0]));

  const utc = { year: 2025, month: 1, day: 1, hour: 0, minute: 0, second: 0 };
  const jd = dhruv.utcToTdbJd(lsk, utc);
  const back = dhruv.jdTdbToUtc(lsk, jd);

  assert.equal(back.year, utc.year);
  assert.equal(back.month, utc.month);
  assert.equal(back.day, utc.day);

  lsk.close();
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

  const search = dhruv.lunarPhaseSearch(
    engine,
    {
      phaseKind: 1,
      queryMode: 0,
      atJdTdb: 2460000.5,
      startJdTdb: 0,
      endJdTdb: 0,
    },
    8,
  );

  assert.equal(search.found, true);

  const tithi = dhruv.tithiForDate(engine, {
    year: 2025,
    month: 1,
    day: 15,
    hour: 12,
    minute: 0,
    second: 0,
  });
  assert.ok(Number.isInteger(tithi.tithiIndex));

  const loc = { latitudeDeg: 12.9716, longitudeDeg: 77.5946, altitudeM: 920 };
  const vaar = dhruv.vaarForDate(engine, eop, {
    year: 2025,
    month: 1,
    day: 15,
    hour: 12,
    minute: 0,
    second: 0,
  }, loc);
  assert.ok(Number.isInteger(vaar.vaarIndex));

  const shadbala = dhruv.shadbalaForDate(engine, eop, {
    year: 2025,
    month: 1,
    day: 15,
    hour: 12,
    minute: 0,
    second: 0,
  }, loc, 0, true);
  assert.equal(shadbala.totalRupas.length, 7);

  const kundali = dhruv.fullKundaliSummaryForDate(engine, eop, {
    year: 2025,
    month: 1,
    day: 15,
    hour: 12,
    minute: 0,
    second: 0,
  }, loc, 0, true);
  assert.ok(Number.isFinite(kundali.ayanamshaDeg));

  eop.close();
  engine.close();
});
