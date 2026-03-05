'use strict';

const { addon } = require('./native');
const { checkStatus } = require('./errors');

function shadbalaForDate(engine, eop, utc, location, ayanamshaSystem = 0, useNutation = true) {
  const r = addon.shadbalaForDate(
    engine._handle,
    eop._handle,
    utc,
    location,
    ayanamshaSystem,
    !!useNutation,
  );
  checkStatus('shadbala_for_date', r.status);
  return {
    totalRupas: r.totalRupas,
  };
}

function fullKundaliSummaryForDate(engine, eop, utc, location, ayanamshaSystem = 0, useNutation = true) {
  const r = addon.fullKundaliSummaryForDate(
    engine._handle,
    eop._handle,
    utc,
    location,
    ayanamshaSystem,
    !!useNutation,
  );
  checkStatus('full_kundali_for_date', r.status);
  return {
    ayanamshaDeg: r.ayanamshaDeg,
    grahaPositionsValid: !!r.grahaPositionsValid,
    panchangValid: !!r.panchangValid,
    dashaSnapshotCount: r.dashaSnapshotCount,
  };
}

module.exports = {
  shadbalaForDate,
  fullKundaliSummaryForDate,
};
