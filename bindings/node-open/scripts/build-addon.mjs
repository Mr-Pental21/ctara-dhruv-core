#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const pkgDir = path.resolve(__dirname, '..');
const repoRoot = path.resolve(pkgDir, '..', '..');

const nativeSrc = path.join(pkgDir, 'native', 'dhruv_node.cc');
const outDir = path.join(pkgDir, 'build', 'Release');
const outFile = path.join(outDir, 'dhruv_node.node');
const headerDir = path.join(repoRoot, 'crates', 'dhruv_ffi_c', 'include');
const targetRelease = path.join(repoRoot, 'target', 'release');

function runOrThrow(cmd, args, opts = {}) {
  const res = spawnSync(cmd, args, { stdio: 'inherit', ...opts });
  if (res.status !== 0) {
    throw new Error(`${cmd} failed with exit code ${res.status}`);
  }
}

function firstExisting(paths) {
  for (const p of paths) {
    if (p && fs.existsSync(p)) {
      return p;
    }
  }
  return null;
}

function findNodeIncludeDir() {
  const candidates = [
    process.env.NODE_INCLUDE_DIR,
    process.config?.variables?.nodedir && path.join(process.config.variables.nodedir, 'include', 'node'),
    process.config?.variables?.node_prefix && path.join(process.config.variables.node_prefix, 'include', 'node'),
    '/usr/include/node',
    '/opt/homebrew/include/node',
    '/usr/local/include/node',
  ];

  const include = firstExisting(candidates);
  if (!include) {
    throw new Error('Unable to locate Node headers. Set NODE_INCLUDE_DIR to a directory containing node_api.h');
  }
  return include;
}

function copyRuntimeLib() {
  const platform = process.platform;
  const libName =
    platform === 'darwin'
      ? 'libdhruv_ffi_c.dylib'
      : platform === 'win32'
      ? 'dhruv_ffi_c.dll'
      : 'libdhruv_ffi_c.so';

  const src = path.join(targetRelease, libName);
  const dst = path.join(outDir, libName);
  if (!fs.existsSync(src)) {
    throw new Error(`Missing ${src}. Run cargo build -p dhruv_ffi_c --release first.`);
  }
  fs.copyFileSync(src, dst);
}

function buildUnix() {
  const includeNode = findNodeIncludeDir();
  const cxx = process.env.CXX || 'c++';

  const args = [
    '-std=c++17',
    '-shared',
    '-fPIC',
    '-DNAPI_VERSION=8',
    '-I', includeNode,
    '-I', headerDir,
    nativeSrc,
    '-o', outFile,
    '-L', targetRelease,
    '-ldhruv_ffi_c',
  ];

  if (process.platform === 'darwin') {
    args.push('-Wl,-rpath,@loader_path');
    args.push('-Wl,-undefined,dynamic_lookup');
  } else {
    args.push('-Wl,-rpath,$ORIGIN');
  }

  runOrThrow(cxx, args, { cwd: pkgDir });
}

function buildWindows() {
  const includeNode = findNodeIncludeDir();
  const nodePrefix = process.config?.variables?.node_prefix;
  const nodeLibDir = nodePrefix || '';
  if (!nodeLibDir) {
    throw new Error('Unable to locate node.lib directory on Windows (node_prefix missing).');
  }

  const args = [
    '/nologo',
    '/std:c++17',
    '/EHsc',
    '/LD',
    '/DNAPI_VERSION=8',
    `/I${includeNode}`,
    `/I${headerDir}`,
    nativeSrc,
    '/link',
    `/OUT:${outFile}`,
    `/LIBPATH:${targetRelease}`,
    'dhruv_ffi_c.lib',
    `/LIBPATH:${nodeLibDir}`,
    'node.lib',
  ];

  runOrThrow('cl', args, { cwd: pkgDir, shell: true });
}

function main() {
  fs.mkdirSync(outDir, { recursive: true });

  // Build C ABI first so linker artifacts exist.
  runOrThrow('cargo', ['build', '-p', 'dhruv_ffi_c', '--release'], { cwd: repoRoot });

  if (process.platform === 'win32') {
    buildWindows();
  } else {
    buildUnix();
  }

  copyRuntimeLib();
  console.log(`Built addon: ${outFile}`);
}

main();
