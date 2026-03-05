'use strict';

const { addon } = require('./native');
const { checkStatus } = require('./errors');

function dashaSnapshotUtc(
  engine,
  eop,
  birthUtc,
  queryUtc,
  location,
  {
    ayanamshaSystem = 0,
    useNutation = true,
    system = 0,
    maxLevel = 3,
  } = {},
) {
  const r = addon.dashaSnapshotUtc(
    engine._handle,
    eop._handle,
    birthUtc,
    queryUtc,
    location,
    ayanamshaSystem,
    !!useNutation,
    system,
    maxLevel,
  );
  checkStatus('dasha_snapshot_utc', r.status);
  return r.snapshot;
}

module.exports = {
  dashaSnapshotUtc,
};
