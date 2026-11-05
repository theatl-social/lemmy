import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { test } from 'node:test';

async function withServer(routes, fn) {
  const server = createServer((req, res) => {
    const [status, type, body] = routes[req.url] || [404, 'text/plain', 'missing'];
    res.writeHead(status, { 'content-type': type });
    res.end(body);
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  try { await fn(`http://127.0.0.1:${server.address().port}`); }
  finally { await new Promise(resolve => server.close(resolve)); }
}
const site = [200, 'application/json', JSON.stringify({version:'0.19.21', site_view:{site:{name:'UI smoke fixture'}}})];
const html = '<html><body><main>UI smoke fixture</main><script src="/static/client.js"></script></body></html>';

test('smoke exercises API, SSR and emitted JavaScript', async () => {
  const { smoke } = await import('./smoke.mjs');
  await withServer({'/api/v3/site':site, '/':[200,'text/html',html], '/static/client.js':[200,'application/javascript','console.log("loaded");']}, url => smoke(url, url, '0.19.21'));
});
test('smoke rejects UI runtime HTTP 500 despite healthy backend', async () => {
  const { smoke } = await import('./smoke.mjs');
  await withServer({'/api/v3/site':site, '/':[500,'text/html','UI smoke fixture']}, url => assert.rejects(smoke(url, url, '0.19.21'), /HTTP 500/));
});
test('smoke rejects a successful error shell without the site', async () => {
  const { smoke } = await import('./smoke.mjs');
  await withServer({'/api/v3/site':site, '/':[200,'text/html','<html>error</html>']}, url => assert.rejects(smoke(url, url, '0.19.21'), /site name/));
});
test('smoke rejects missing JavaScript served as HTML', async () => {
  const { smoke } = await import('./smoke.mjs');
  await withServer({'/api/v3/site':site, '/':[200,'text/html',html], '/static/client.js':[200,'text/html',html]}, url => assert.rejects(smoke(url, url, '0.19.21'), /JavaScript/));
});

for (const version of ['v0.19.21', 'v0.19.21-1-gabcdef0']) {
  test(`smoke accepts the matching release banner ${version}`, async () => {
    const { smoke } = await import('./smoke.mjs');
    const taggedSite = [200, 'application/json', JSON.stringify({version, site_view:{site:{name:'UI smoke fixture'}}})];
    await withServer({'/api/v3/site':taggedSite, '/':[200,'text/html',html], '/static/client.js':[200,'application/javascript','console.log("loaded");']}, url => smoke(url, url, '0.19.21'));
  });
}

test('smoke rejects a stale prefixed release banner', async () => {
  const { smoke } = await import('./smoke.mjs');
  const staleSite = [200, 'application/json', JSON.stringify({version:'v0.19.20', site_view:{site:{name:'UI smoke fixture'}}})];
  await withServer({'/api/v3/site':staleSite}, url => assert.rejects(smoke(url, url, '0.19.21'), /unexpected backend version/));
});
