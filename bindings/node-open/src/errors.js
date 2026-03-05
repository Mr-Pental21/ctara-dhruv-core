'use strict';

const { STATUS, statusName } = require('./status');

class DhruvError extends Error {
  constructor(operation, status) {
    super(`${operation} failed: ${statusName(status)} (${status})`);
    this.name = 'DhruvError';
    this.operation = operation;
    this.status = status;
  }
}

function checkStatus(operation, status) {
  if (status !== STATUS.OK) {
    throw new DhruvError(operation, status);
  }
}

module.exports = {
  DhruvError,
  checkStatus,
};
