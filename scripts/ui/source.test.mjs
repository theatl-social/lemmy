import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

const workflow = readFileSync(new URL('../../.github/workflows/docker-build.yml', import.meta.url), 'utf8');
test('UI checkout uses an immutable release commit, never upstream main', () => {
  const checkout = workflow.match(/repository: LemmyNet\/lemmy-ui\n\s+ref: (\S+)/);
  assert.ok(checkout, 'UI checkout must be explicit');
  assert.match(checkout[1], /^[a-f0-9]{40}$/, 'UI source must be pinned to a release commit');
});

test('source validation rejects a newer UI major version', async () => {
  const { validateVersions } = await import('./validate-source.mjs');
  assert.throws(() => validateVersions('0.19.21', '1.0.0-alpha.12'), /does not match/);
});
test('source validation rejects stale UI on the next backend release', async () => {
  const { validateVersions } = await import('./validate-source.mjs');
  assert.throws(() => validateVersions('0.19.22', '0.19.21'), /does not match/);
});
test('source validation accepts the matching stable release', async () => {
  const { validateVersions } = await import('./validate-source.mjs');
  assert.doesNotThrow(() => validateVersions('0.19.21', '0.19.21'));
});

test('release tags cannot mislabel a different tested version', async () => {
  const { validateReleaseTag } = await import('./validate-source.mjs');
  assert.throws(() => validateReleaseTag('0.19.21', 'tag', 'v0.19.22'), /does not match/);
});
test('release tag validation accepts matching prefixed and upstream tags', async () => {
  const { validateReleaseTag } = await import('./validate-source.mjs');
  assert.doesNotThrow(() => validateReleaseTag('0.19.21', 'tag', 'v0.19.21'));
  assert.doesNotThrow(() => validateReleaseTag('0.19.21', 'tag', '0.19.21'));
});
test('branch names are not interpreted as release tags', async () => {
  const { validateReleaseTag } = await import('./validate-source.mjs');
  assert.doesNotThrow(() => validateReleaseTag('0.19.21', 'branch', 'merge/0.19.21'));
});
