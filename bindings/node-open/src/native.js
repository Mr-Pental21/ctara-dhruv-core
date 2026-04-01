'use strict';

const fs = require('node:fs');
const path = require('node:path');

function resolvePrebuildPath() {
  const platform = process.platform;
  const arch = process.arch;
  const candidate = path.resolve(
    __dirname,
    '..',
    'prebuilds',
    `${platform}-${arch}`,
    'dhruv_node.node',
  );
  return fs.existsSync(candidate) ? candidate : null;
}

function resolveAddonPath() {
  const envPath = process.env.DHRUV_NODE_ADDON_PATH;
  if (envPath) {
    const abs = path.resolve(envPath);
    if (!fs.existsSync(abs)) {
      throw new Error(`DHRUV_NODE_ADDON_PATH points to missing file: ${abs}`);
    }
    return abs;
  }

  const local = path.resolve(__dirname, '..', 'build', 'Release', 'dhruv_node.node');
  if (fs.existsSync(local)) {
    return local;
  }

  const prebuild = resolvePrebuildPath();
  if (prebuild) {
    return prebuild;
  }

  throw new Error(
    [
      `Cannot find native addon at ${local}.`,
      'Install a published package with bundled prebuilds or run: npm run build (from bindings/node-open)',
      'Or set DHRUV_NODE_ADDON_PATH=/abs/path/to/dhruv_node.node',
    ].join(' '),
  );
}

const addon = require(resolveAddonPath());

module.exports = {
  addon,
};
