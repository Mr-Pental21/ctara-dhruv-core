'use strict';

const engine = require('./engine');
const time = require('./time');
const search = require('./search');
const panchang = require('./panchang');
const jyotish = require('./jyotish');
const shadbala = require('./shadbala');
const dasha = require('./dasha');
const tara = require('./tara');
const { STATUS, EXPECTED_API_VERSION } = require('./status');

module.exports = {
  ...engine,
  ...time,
  ...search,
  ...panchang,
  ...jyotish,
  ...shadbala,
  ...dasha,
  ...tara,
  STATUS,
  EXPECTED_API_VERSION,
};
