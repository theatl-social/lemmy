import assert from 'node:assert/strict';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

async function get(url) {
  const res = await fetch(url, { signal: AbortSignal.timeout(10000), redirect: 'error' });
  assert.equal(res.status, 200, `${url}: HTTP ${res.status}`);
  return res;
}

export async function smoke(backendUrl, uiUrl, expectedVersion) {
  const site = await (await get(`${backendUrl}/api/v3/site`)).json();
  // git describe retains a release tag's optional v prefix and fork commit suffix.
  const version = typeof site.version === 'string' ? site.version.replace(/^v/, '') : '';
  assert.ok(version === expectedVersion || version.startsWith(`${expectedVersion}-`), `unexpected backend version: ${site.version}`);
  const name = site.site_view?.site?.name;
  assert.ok(name, 'backend site name missing');
  const page = await get(`${uiUrl}/`);
  assert.match(page.headers.get('content-type') || '', /text\/html/);
  const html = await page.text();
  assert.ok(html.includes(name), 'rendered UI must contain the configured site name');
  assert.match(html, /<body[\s>]/i, 'SSR HTML body missing');
  const scripts = [...html.matchAll(/<script\b[^>]*\bsrc=["']([^"']+)["']/gi)].map(match => match[1]);
  assert.ok(scripts.length > 0, 'rendered UI must reference JavaScript assets');
  for (const src of scripts) {
    const url = new URL(src.replaceAll('&amp;', '&'), `${uiUrl}/`);
    assert.equal(url.origin, new URL(uiUrl).origin, 'UI assets must be served by the tested UI');
    const asset = await get(url);
    assert.match(asset.headers.get('content-type') || '', /(?:javascript|ecmascript)/i, `${url}: expected JavaScript`);
    assert.ok((await asset.text()).length > 0, `${url}: empty JavaScript`);
  }
  console.log(`UI smoke passed: backend ${site.version}, SSR site ${name}, ${scripts.length} JavaScript assets`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [backend, ui, version] = process.argv.slice(2);
  assert.ok(backend && ui && version, 'usage: node scripts/ui/smoke.mjs BACKEND_URL UI_URL VERSION');
  await smoke(backend, ui, version);
}
