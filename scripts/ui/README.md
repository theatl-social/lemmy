# UI compatibility and image smoke

The backend workflow rebuilds **upstream `LemmyNet/lemmy-ui`**, not a customized
`theatl-social/lemmy-ui` source tree. UI source is pinned to the matching backend
release commit. Update the pinned SHA when updating the backend; the source guard
checks the UI package version, release tag, checked-out commit and translations.
For GitHub tag events, the publishing tag must also match the backend version
(with an optional `v` prefix).

Run the small regression suite with Node 22 or later:

```sh
node --test scripts/ui/*.test.mjs
node scripts/ui/validate-source.mjs /path/to/checked-out-lemmy-ui
```

The validator runs from the backend repository root. The UI checkout needs full
Git tags and recursively initialized submodules. For 0.19.21 its source is
`7cb4cd8c20f3b2c37e9a0a9797aa9caddd79db58`.

## Runtime smoke

With locally built images and Docker Compose available:

```sh
LEMMY_SMOKE_BACKEND_IMAGE=your-backend-image \
LEMMY_SMOKE_UI_IMAGE=your-ui-image \
LEMMY_SMOKE_VERSION=0.19.21 \
bash scripts/ui/smoke-images.sh
```

This starts a separate Compose project with PostgreSQL 16 in tmpfs, a freshly
initialized backend (scheduled tasks and activity sending disabled) and the UI.
It publishes only random loopback ports, uses no
development data directories, and removes its own containers/network/volumes on
exit. PostgreSQL may be pulled; backend/UI images must already exist locally.
It checks backend readiness, the reported version, server-rendered HTML containing
the real configured site name, and nonempty JavaScript assets with the correct
content type. It prints container logs on failure. It does not prove browser
hydration, login, uploads, federation, or production configuration.

The HTTP checks can also run against an already-started disposable native stack:

```sh
node scripts/ui/smoke.mjs http://127.0.0.1:8536 http://127.0.0.1:1234 0.19.21
```

CI exports the backend and UI build results as Docker archives, loads those exact
archives for smoke, and permits publication only when smoke succeeds. Publication
loads and retags those same archives; it never rebuilds after the gate. PRs run the
same smoke without publishing. Runs for the same Git ref are serialized to avoid
interleaving backend/UI publication from different runs. Publishing two images is
still not atomic; downstream deployments should pin the tested image pair by
exact image digests instead of consuming mutable tags independently.
The upstream backend tag must exist in the fork
before a release build, because its runtime banner uses `git describe`.

## Diagnosis evidence (2026-11-05)

There were historical build failures and a distinct compatibility defect:

- [Run 19874791323, UI job 56959380602](https://github.com/theatl-social/lemmy/actions/runs/19874791323/job/56959380602)
  failed on 2025-12-02 at `RUN corepack enable`: `/bin/sh: corepack: not found`,
  exit 127. UI 0.19.13 used moving `node:alpine` and assumed bundled Corepack.
- [Run 20076498816, UI job 57592984028](https://github.com/theatl-social/lemmy/actions/runs/20076498816/job/57592984028)
  failed on 2025-12-09 at `generate_translations.js:8` with
  `TypeError: Cannot read properties of undefined (reading 'forEach')`.
  The checkout had `submodules: false`. Commit `6711450db` restored recursive
  submodules; later builds succeeded.
- That second log records upstream UI source
  `2a898a1c4e106e9675318fcb6ca4641e5ecefc1c` and UI `1.0.0-alpha.12`.
  Its [environment resolver](https://github.com/LemmyNet/lemmy-ui/blob/2a898a1c4e106e9675318fcb6ca4641e5ecefc1c/src/shared/utils/env.ts#L7-L12)
  expects `LEMMY_UI_BACKEND_INTERNAL`/`LEMMY_UI_BACKEND`, while this repo's 0.19
  Compose configuration supplies `LEMMY_UI_LEMMY_INTERNAL_HOST` and
  `LEMMY_UI_LEMMY_EXTERNAL_HOST`. The unmatched settings fall back to
  `localhost:8536`, which points into the UI container itself.
- Even with corrected hosts, that source's pinned `lemmy-js-client`
  [uses API v4](https://github.com/LemmyNet/lemmy-js-client/blob/67a4293fd712e692103d3e3cf5902b6c1e218128/src/http.ts#L240).
  The 0.19 backend registers API v3 in `src/api_routes_http.rs`.
  These are concrete source incompatibilities; no production logs were inspected.
- [Run 30169044869, UI job 89706848903](https://github.com/theatl-social/lemmy/actions/runs/30169044869/job/89706848903)
  reports successful build/publication on 2026-07-25. Its logs have expired (HTTP
  410), so its exact UI source commit cannot be recovered from those logs here.
  A successful image build did not establish compatibility with the backend.

An authenticated exact-repository query for `theatl-social/lemmy-ui` returned
HTTP 404. This means that repository was unavailable to the current credentials;
it is not proof that no private repository exists. The workflow source selection
is explicit regardless.

### Local verification on 2026-11-05

The merged backend built with Rust 1.81 and `cargo build --locked`. The pinned UI
built with its frozen pnpm lockfile and production webpack configuration. Against
a fresh disposable native PostgreSQL 17 database, the HTTP smoke passed for the
real backend/UI pair. API v3 returned HTTP 200 and API v4 returned HTTP 404.

Chrome loaded the UI through a temporary local reverse proxy, navigated to Login,
and successfully authenticated the synthetic admin: the page displayed "Logged
in", the admin username and Admin Settings. No fixture credentials were saved.
The fork's private API also rejected an incorrect secret, created a synthetic
account through `privileged_register`, and found its email through `check_email`.
Eight focused upstream request/IP-validation tests passed with no skips. The
external metadata-fetch test was excluded from that focused run.

These are native build, database and browser results. The Docker image harness
and registry publication still require a successful CI run; no production
configuration or deployed service was tested.

### CI fixture correction on 2026-11-06

The first image smoke in run `42113876657` stopped during site initialization:
the fixture name `Lemmy UI smoke fixture` was 21 characters, exceeding the site's
20-character database limit. The harness now uses `Lemmy UI smoke`, matching the
successful native fixture. Both Linux image builds had succeeded.

The exact backend and UI archives from that run passed the corrected complete
image harness locally in Docker Desktop (amd64 images under ARM emulation),
including real PostgreSQL, API readiness, SSR and JavaScript assets. The harness
removed its own test containers and network. Native Linux CI must still pass
before publication.

The matched 0.19.21 UI Dockerfile installs Corepack explicitly, avoiding the old
missing-Corepack assumption. It still uses mutable `node:current-slim` base images,
unversioned global `node-gyp`/`corepack` installs and OS packages. Pinning UI source
alone does not make builds reproducible. The real-image gate detects resulting
build/startup failures; this change does not maintain a separate UI Dockerfile.
