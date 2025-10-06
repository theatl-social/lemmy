# Docker Build Workflows

This document explains the Docker image building and publishing process for Lemmy and Lemmy-UI.

## Overview

The `docker-build.yml` workflow automatically builds and publishes Docker images to **GitHub Container Registry (GHCR)** when code is pushed to specific branches or tags.

## Published Images

Images are published to:
- **Lemmy Backend**: `ghcr.io/theatl-social/lemmy`
- **Lemmy UI**: `ghcr.io/theatl-social/lemmy-ui`

## Triggers

The workflow runs on:
- **Push** to `v0.19.13-theatl` branch
- **Tags** matching `v*` pattern (e.g., `v0.19.13`, `v1.0.0`)
- **Pull Requests** to `v0.19.13-theatl` (build only, no push)

## Image Tags

Images are automatically tagged with:
- **Branch name**: `v0.19.13-theatl`
- **PR number**: `pr-123` (for pull requests)
- **Semver**: `0.19.13`, `0.19` (for version tags)
- **SHA**: `v0.19.13-theatl-abc1234` (git commit SHA)
- **Latest**: `latest` (for default branch only)

## Platform Support

Images are built for:
- `linux/amd64` (x86_64)

## Authentication

The workflow uses `GITHUB_TOKEN` which is automatically available in GitHub Actions. **No manual secret configuration is required.**

The token needs the following permissions (automatically granted):
- `contents: read` - Read repository code
- `packages: write` - Push to GitHub Container Registry

## Usage

### Pulling Images

Images are public by default. To pull:

```bash
# Pull Lemmy backend
docker pull ghcr.io/theatl-social/lemmy:latest

# Pull Lemmy UI
docker pull ghcr.io/theatl-social/lemmy-ui:latest

# Pull specific version
docker pull ghcr.io/theatl-social/lemmy:v0.19.13-theatl
```

### Using in Docker Compose

```yaml
version: '3.8'

services:
  lemmy:
    image: ghcr.io/theatl-social/lemmy:latest
    # ... rest of configuration

  lemmy-ui:
    image: ghcr.io/theatl-social/lemmy-ui:latest
    # ... rest of configuration
```

### Using with Private Images

If images are set to private, authenticate first:

```bash
# Create a Personal Access Token (PAT) with read:packages scope
# https://github.com/settings/tokens

echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
docker pull ghcr.io/theatl-social/lemmy:latest
```

## Build Process

### Lemmy Backend

1. Checkout repository with submodules
2. Set up Docker Buildx (multi-platform support)
3. Extract metadata for tags and labels
4. Login to GHCR
5. Build image using `docker/Dockerfile`
6. Push to GHCR (if not a PR)
7. Cache layers for faster subsequent builds

### Lemmy UI

1. Checkout repository
2. Checkout `LemmyNet/lemmy-ui` (main branch)
3. Set up Docker Buildx
4. Extract metadata for tags and labels
5. Login to GHCR
6. Build image from `lemmy-ui` directory
7. Push to GHCR (if not a PR)
8. Cache layers for faster subsequent builds

## Caching

The workflow uses GitHub Actions cache to speed up builds:
- `cache-from: type=gha` - Use cached layers from previous builds
- `cache-to: type=gha,mode=max` - Save all layers for future builds

This significantly reduces build times for subsequent runs.

## Monitoring Builds

### Check Workflow Status

1. Go to: https://github.com/theatl-social/lemmy/actions
2. Click on "Build Docker Images" workflow
3. View individual job logs

### View Published Images

1. Go to: https://github.com/orgs/theatl-social/packages
2. Or directly:
   - https://github.com/theatl-social/lemmy/pkgs/container/lemmy
   - https://github.com/theatl-social/lemmy/pkgs/container/lemmy-ui

## Troubleshooting

### Build Failures

Common issues:
- **Docker file not found**: Check `file: docker/Dockerfile` path is correct
- **Permission denied**: Ensure `packages: write` permission is set
- **Out of disk space**: GitHub runners have limited space, clean up if needed

### Authentication Issues

If images won't push:
1. Check workflow has `packages: write` permission
2. Verify `GITHUB_TOKEN` is being used correctly
3. Check organization/repo settings allow package publishing

### Image Not Found

If you can't pull an image:
1. Check the image was actually pushed (not just built)
2. Verify image visibility (public vs private)
3. Ensure you're using the correct registry URL (`ghcr.io`)

## Manual Triggers

To manually trigger a build:

1. Go to: https://github.com/theatl-social/lemmy/actions/workflows/docker-build.yml
2. Click "Run workflow"
3. Select branch and click "Run workflow"

## Local Testing

To test the Docker build locally:

```bash
# Build Lemmy backend
docker build -f docker/Dockerfile -t lemmy:local .

# Build Lemmy UI (clone lemmy-ui first)
git clone https://github.com/LemmyNet/lemmy-ui.git
docker build -t lemmy-ui:local ./lemmy-ui
```

## Security Considerations

- **GITHUB_TOKEN** has limited scope and automatically expires
- Images can be scanned for vulnerabilities in GHCR
- Consider setting images to private if needed
- Regularly update base images for security patches

## Customization

### Adding More Platforms

To add ARM support, update the workflow:

```yaml
platforms: linux/amd64,linux/arm64
```

### Changing Base Branch

Update the `on.push.branches` section:

```yaml
on:
  push:
    branches: [ "your-branch-name" ]
```

### Adding Build Arguments

Add build args to the build step:

```yaml
- name: Build and push Lemmy Docker image
  uses: docker/build-push-action@v5
  with:
    # ... existing config
    build-args: |
      BUILD_VERSION=${{ github.ref_name }}
      COMMIT_SHA=${{ github.sha }}
```
