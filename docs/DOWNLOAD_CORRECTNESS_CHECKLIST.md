# Download correctness and release checklist

This checklist defines the minimum acceptance bar for a release. A successful compile alone does not establish download correctness.

## HTTP and HTTPS

- [ ] A normal HTTP 200 download produces the expected bytes and SHA-256 checksum.
- [ ] A server that supports byte ranges returns valid 206 responses with matching Content-Range.
- [ ] A server that ignores Range is never treated as a successful segmented download.
- [ ] Redirects, 401/403, 404, 429, and 5xx responses are reported correctly.
- [ ] Chunked responses and unknown Content-Length do not produce false 100% completion.
- [ ] A truncated response is not marked completed.
- [ ] A remote file changed between attempts does not silently append stale bytes.

## Pause, resume, and lifecycle

- [ ] Pause during a single-stream download stops writing and preserves a recoverable partial file.
- [ ] Pause during segmented download does not assemble incomplete segments.
- [ ] Resume validates the requested offset and the server's Content-Range.
- [ ] Cancel, remove, restart, and application shutdown do not leave workers writing to invalid objects or destinations.
- [ ] Restarting the application restores metadata without falsely marking incomplete downloads completed.

## Filesystem failures

- [ ] Existing destination files are handled with an explicit overwrite decision.
- [ ] Missing parent directories, permission errors, and disk-full conditions surface a clear error.
- [ ] A failed assembly does not publish a partial destination as complete.
- [ ] Temporary and segment files are cleaned up only when it is safe to do so.

## Android

- [ ] A server that ignores Range is not marked as a successful segmented download.
- [ ] Every segment receives HTTP 206 and a matching Content-Range before assembly.
- [ ] The assembled file length matches the probed content length.
- [ ] Resume writes to a temporary remainder file and appends only after a valid 206 response.
- [ ] Filenames, URLs, and destination paths containing quotes, backslashes, or control characters produce valid JSON.
- [ ] Pause/resume and cancel are exercised on a physical device; incomplete segments are not treated as completed.

## Browser integration

- [ ] Automatic interception is disabled by default and must be explicitly enabled.
- [ ] A failed custom-protocol dispatch leaves the browser's original download running.
- [ ] Unsupported URL schemes are rejected by both popup and background worker.
- [ ] The extension uses only the permissions required for its features.
- [ ] The extension displays a handoff request, not a claim that the desktop app accepted the download.

## Release gate

- [ ] Windows build succeeds on a clean runner.
- [ ] Android build succeeds without silently falling back from a failed release build.
- [ ] Website build and lint checks pass.
- [ ] At least one end-to-end test per HTTP scenario above runs against a deterministic local test server.
- [ ] Release artifacts are attached to a versioned GitHub release with checksums and notes.
- [ ] The website links to real release artifacts and platform support is accurately described.

Do not mark a release ready while a required item is unverified. Record skipped tests and known limitations in the release notes.
