'use strict';

const { addon } = require('./native');
const { checkStatus } = require('./errors');

class TaraCatalog {
  constructor(handle) {
    this._handle = handle;
    this._closed = false;
  }

  static load(path) {
    const r = addon.taraCatalogLoad(path);
    checkStatus('tara_catalog_load', r.status);
    return new TaraCatalog(r.handle);
  }

  close() {
    if (this._closed) return;
    addon.taraCatalogFree(this._handle);
    this._closed = true;
    this._handle = null;
  }

  galacticCenterEcliptic(jdTdb) {
    const r = addon.taraGalacticCenterEcliptic(this._handle, jdTdb);
    checkStatus('tara_galactic_center_ecliptic', r.status);
    return r.coords;
  }

  compute(request) {
    const r = addon.taraComputeEx(this._handle, request);
    checkStatus('tara_compute_ex', r.status);
    return r.result;
  }
}

function propagatePosition(raDeg, decDeg, parallaxMas, pmRaMasYr, pmDecMasYr, rvKmS, dtYears) {
  const r = addon.taraPropagatePosition(raDeg, decDeg, parallaxMas, pmRaMasYr, pmDecMasYr, rvKmS, dtYears);
  checkStatus('tara_propagate_position', r.status);
  return r.position;
}

function applyAberration(direction, earthVelAuDay) {
  const r = addon.taraApplyAberration(direction, earthVelAuDay);
  checkStatus('tara_apply_aberration', r.status);
  return r.direction;
}

function applyLightDeflection(direction, sunToObserver, sunObserverDistanceAu) {
  const r = addon.taraApplyLightDeflection(direction, sunToObserver, sunObserverDistanceAu);
  checkStatus('tara_apply_light_deflection', r.status);
  return r.direction;
}

function galacticAnticenterIcrs() {
  const r = addon.taraGalacticAnticenterIcrs();
  checkStatus('tara_galactic_anticenter_icrs', r.status);
  return r.direction;
}

module.exports = {
  TaraCatalog,
  applyAberration,
  applyLightDeflection,
  galacticAnticenterIcrs,
  propagatePosition,
};
