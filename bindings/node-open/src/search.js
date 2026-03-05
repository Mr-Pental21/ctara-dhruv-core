'use strict';

const { addon } = require('./native');
const { checkStatus } = require('./errors');

function lunarPhaseSearch(engine, request, capacity = 8) {
  const r = addon.lunarPhaseSearch(engine._handle, request, capacity);
  checkStatus('lunar_phase_search_ex', r.status);
  return {
    found: !!r.found,
    count: r.count || 0,
    event: r.event || null,
    events: r.events || [],
  };
}

module.exports = {
  lunarPhaseSearch,
};
