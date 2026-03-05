'use strict';

const { addon } = require('./native');
const { checkStatus } = require('./errors');

function tithiForDate(engine, utc) {
  const r = addon.tithiForDate(engine._handle, utc);
  checkStatus('tithi_for_date', r.status);
  return r.tithi;
}

function vaarForDate(engine, eop, utc, location) {
  const r = addon.vaarForDate(engine._handle, eop._handle, utc, location);
  checkStatus('vaar_for_date', r.status);
  return r.vaar;
}

module.exports = {
  tithiForDate,
  vaarForDate,
};
