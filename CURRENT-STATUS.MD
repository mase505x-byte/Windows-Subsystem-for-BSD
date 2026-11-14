## Current status

Windows Subsystem for BSD (WSB) is an early-stage prototype for running a BSD environment on Windows. The repository currently contains buildable foundations for protocol types, interop primitives, and a minimal CLI, but it does not yet implement a working BSD runtime or a full WSL-style subsystem.

The project is intentionally staged as follows:
- build the host-side management tools
- add support for a single BSD distribution
- verify installation and runtime lifecycle
- extend integration features only after the guest runs reliably

At present, the project does not provide:
- a turnkey installer
- a packaged BSD runtime
- a working `shell` command
- a live host/server implementation

## Planned first guest: FreeBSD

The first supported guest should be FreeBSD. This is the most practical target for a working first release because it is a mature BSD operating system with official release images and well-defined checksum verification.

### FreeBSD amd64 ISO

The official FreeBSD release images are hosted at:

- https://download.freebsd.org/releases/ISO-IMAGES/14.5/

The current production release is FreeBSD 14.5-RELEASE for amd64.

### Checksum verification

FreeBSD publishes official SHA256 and SHA512 checksum files alongside the ISOs. These are the recommended verification steps before use:

```sh
sha256sum -c CHECKSUM.SHA256-FreeBSD-14.5-RELEASE-amd64
