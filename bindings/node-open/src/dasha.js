'use strict';

const { addon } = require('./native');
const { checkStatus } = require('./errors');

class DashaHierarchy {
  constructor(handle) {
    this._handle = handle;
    this._closed = false;
  }

  close() {
    if (this._closed) return;
    addon.dashaHierarchyFree(this._handle);
    this._closed = true;
    this._handle = null;
  }

  levelCount() {
    const r = addon.dashaHierarchyLevelCount(this._handle);
    checkStatus('dasha_hierarchy_level_count', r.status);
    return r.count;
  }

  periodCount(level) {
    const r = addon.dashaHierarchyPeriodCount(this._handle, level);
    checkStatus('dasha_hierarchy_period_count', r.status);
    return r.count;
  }

  periodAt(level, idx) {
    const r = addon.dashaHierarchyPeriodAt(this._handle, level, idx);
    checkStatus('dasha_hierarchy_period_at', r.status);
    return r.period;
  }
}

function dashaSelectionConfigDefault() {
  return addon.dashaSelectionConfigDefault();
}

function dashaHierarchyUtc(
  engine,
  eop,
  birthUtc,
  location,
  {
    ayanamshaSystem = 0,
    useNutation = true,
    system = 0,
    maxLevel = 3,
  } = {},
) {
  const r = addon.dashaHierarchyUtc(
    engine._handle,
    eop._handle,
    birthUtc,
    location,
    ayanamshaSystem,
    !!useNutation,
    system,
    maxLevel,
  );
  checkStatus('dasha_hierarchy_utc', r.status);
  return new DashaHierarchy(r.handle);
}

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
  DashaHierarchy,
  dashaSelectionConfigDefault,
  dashaHierarchyUtc,
  dashaSnapshotUtc,
};
