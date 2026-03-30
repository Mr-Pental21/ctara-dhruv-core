'use strict';

const { addon } = require('./native');
const { checkStatus } = require('./errors');

const RANGE_QUERY_MODE = 2;
const DEFAULT_RANGE_CAPACITY = 8;

function conjunctionConfigDefault() {
  return addon.conjunctionConfigDefault();
}

function grahanConfigDefault() {
  return addon.grahanConfigDefault();
}

function stationaryConfigDefault() {
  return addon.stationaryConfigDefault();
}

function normalizeRangeCapacity(capacity) {
  if (!Number.isFinite(capacity) || capacity < 1) {
    return DEFAULT_RANGE_CAPACITY;
  }
  return Math.max(1, Math.trunc(capacity));
}

function runSearch(statusName, searchFn, engine, request, capacity) {
  const response = searchFn(engine._handle, request, capacity);
  checkStatus(statusName, response.status);
  return response;
}

function collectRangeSearch(statusName, searchFn, engine, request, capacity) {
  let currentCapacity = normalizeRangeCapacity(capacity);
  let response = runSearch(statusName, searchFn, engine, request, currentCapacity);
  while ((response.count || 0) >= currentCapacity) {
    currentCapacity *= 2;
    response = runSearch(statusName, searchFn, engine, request, currentCapacity);
  }
  return response;
}

function formatSimpleSearch(response) {
  return {
    found: !!response.found,
    count: response.count || 0,
    event: response.event || null,
    events: response.events || [],
  };
}

function formatGrahanSearch(response) {
  return {
    found: !!response.found,
    count: response.count || 0,
    chandra: response.chandra || null,
    surya: response.surya || null,
    chandraEvents: response.chandraEvents || [],
    suryaEvents: response.suryaEvents || [],
  };
}

function formatMotionSearch(response) {
  return {
    found: !!response.found,
    count: response.count || 0,
    stationary: response.stationary || null,
    maxSpeed: response.maxSpeed || null,
    stationaryEvents: response.stationaryEvents || [],
    maxSpeedEvents: response.maxSpeedEvents || [],
  };
}

function searchResult(statusName, searchFn, engine, request, capacity, formatResponse) {
  const response = request && request.queryMode === RANGE_QUERY_MODE
    ? collectRangeSearch(statusName, searchFn, engine, request, capacity)
    : runSearch(statusName, searchFn, engine, request, normalizeRangeCapacity(capacity));
  return formatResponse(response);
}

function lunarPhaseSearch(engine, request, capacity = DEFAULT_RANGE_CAPACITY) {
  return searchResult(
    'lunar_phase_search_ex',
    addon.lunarPhaseSearch,
    engine,
    request,
    capacity,
    formatSimpleSearch,
  );
}

function conjunctionSearch(engine, request, capacity = DEFAULT_RANGE_CAPACITY) {
  return searchResult(
    'conjunction_search_ex',
    addon.conjunctionSearch,
    engine,
    request,
    capacity,
    formatSimpleSearch,
  );
}

function grahanSearch(engine, request, capacity = DEFAULT_RANGE_CAPACITY) {
  return searchResult(
    'grahan_search_ex',
    addon.grahanSearch,
    engine,
    request,
    capacity,
    formatGrahanSearch,
  );
}

function motionSearch(engine, request, capacity = DEFAULT_RANGE_CAPACITY) {
  return searchResult(
    'motion_search_ex',
    addon.motionSearch,
    engine,
    request,
    capacity,
    formatMotionSearch,
  );
}

function sankrantiSearch(engine, request, capacity = DEFAULT_RANGE_CAPACITY) {
  return searchResult(
    'sankranti_search_ex',
    addon.sankrantiSearch,
    engine,
    request,
    capacity,
    formatSimpleSearch,
  );
}

module.exports = {
  conjunctionConfigDefault,
  grahanConfigDefault,
  stationaryConfigDefault,
  conjunctionSearch,
  grahanSearch,
  motionSearch,
  lunarPhaseSearch,
  sankrantiSearch,
};
