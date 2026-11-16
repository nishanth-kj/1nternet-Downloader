# Project website

This directory contains the Internet Downloader static website.

## Development

Requires Node.js 20 or newer.

```bash
npm ci
npm run dev
```

## Production build

```bash
npm run build
```

The static export is generated in `out/`. Deployment is managed by the repository workflow in `.github/workflows/deploy-site.yml`.

For overall project information, see the [root README](../README.md) and [project documentation](../docs/README.md).
