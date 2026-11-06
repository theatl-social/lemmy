import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

export function validateVersions(backend, ui) {
  assert.match(backend, /^\d+\.\d+\.\d+$/, 'backend must be a stable release');
  assert.equal(ui, backend, `UI ${ui} does not match backend ${backend}`);
}

export function validateReleaseTag(version, refType, refName) {
  if (refType === 'tag') {
    assert.equal(refName?.replace(/^v/, ''), version, `Release tag ${refName} does not match backend ${version}`);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const uiDir = process.argv[2] || 'lemmy-ui';
  const cargo = readFileSync('Cargo.toml', 'utf8');
  const backend = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  const ui = JSON.parse(readFileSync(`${uiDir}/package.json`, 'utf8')).version;
  validateVersions(backend, ui);
  validateReleaseTag(backend, process.env.GITHUB_REF_TYPE, process.env.GITHUB_REF_NAME);
  const git = (...args) => execFileSync('git', ['-C', uiDir, ...args], { encoding: 'utf8' }).trim();
  const workflow = readFileSync('.github/workflows/docker-build.yml', 'utf8');
  const expected = workflow.match(/repository: LemmyNet\/lemmy-ui\n\s+ref: ([a-f0-9]{40})\b/)?.[1];
  assert.ok(expected, 'UI checkout must use an immutable commit');
  assert.equal(git('rev-parse', 'HEAD'), expected, 'checked-out UI differs from pinned source');
  assert.equal(git('rev-parse', `${ui}^{commit}`), expected, 'UI release tag differs from pinned source');
  assert.equal(git('describe', '--tags', '--exact-match'), ui, 'UI version banner must identify the release');
  assert.ok(readFileSync(`${uiDir}/lemmy-translations/translations/en.json`).length > 0, 'translations submodule must be populated');
  console.log(`Validated backend/UI ${backend}, UI source ${expected}`);
}
