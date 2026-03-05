'use strict';

const fs = require('node:fs');
const path = require('node:path');

function repoRoot() {
  return path.resolve(__dirname, '..', '..', '..');
}

function kernelPaths() {
  const root = repoRoot();
  const base = path.join(root, 'kernels', 'data');
  return {
    spk: path.join(base, 'de442s.bsp'),
    lsk: path.join(base, 'naif0012.tls'),
    eop: path.join(base, 'finals2000A.all'),
    tara: path.join(base, 'tara_catalog.json'),
  };
}

function exists(p) {
  try {
    fs.accessSync(p, fs.constants.R_OK);
    return true;
  } catch {
    return false;
  }
}

function hasKernels() {
  const k = kernelPaths();
  return exists(k.spk) && exists(k.lsk);
}

function hasEop() {
  return exists(kernelPaths().eop);
}

function hasTara() {
  return exists(kernelPaths().tara);
}

module.exports = {
  kernelPaths,
  hasKernels,
  hasEop,
  hasTara,
};
