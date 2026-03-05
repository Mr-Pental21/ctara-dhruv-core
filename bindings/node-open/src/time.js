'use strict';

const { addon } = require('./native');
const { checkStatus } = require('./errors');

function utcToTdbJd(lsk, utc) {
  const r = addon.utcToTdbJd(lsk._handle, utc);
  checkStatus('utc_to_tdb_jd', r.status);
  return r.jdTdb;
}

function jdTdbToUtc(lsk, jdTdb) {
  const r = addon.jdTdbToUtc(lsk._handle, jdTdb);
  checkStatus('jd_tdb_to_utc', r.status);
  return r.utc;
}

function nutationIau2000b(jdTdb) {
  const r = addon.nutationIau2000b(jdTdb);
  checkStatus('nutation_iau2000b', r.status);
  return { dpsi: r.dpsi, deps: r.deps };
}

function riseSetConfigDefault() {
  return addon.riseSetConfigDefault();
}

function bhavaConfigDefault() {
  return addon.bhavaConfigDefault();
}

function sankrantiConfigDefault() {
  return addon.sankrantiConfigDefault();
}

module.exports = {
  utcToTdbJd,
  jdTdbToUtc,
  nutationIau2000b,
  riseSetConfigDefault,
  bhavaConfigDefault,
  sankrantiConfigDefault,
};
