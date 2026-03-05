'use strict';

const { addon } = require('./native');
const { checkStatus } = require('./errors');

function grahaSiderealLongitudes(engine, jdTdb, ayanamshaSystem = 0, useNutation = true) {
  const r = addon.grahaSiderealLongitudes(engine._handle, jdTdb, ayanamshaSystem, !!useNutation);
  checkStatus('graha_sidereal_longitudes', r.status);
  return r.longitudes;
}

module.exports = {
  grahaSiderealLongitudes,
};
