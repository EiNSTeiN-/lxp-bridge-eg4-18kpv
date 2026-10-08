# lxp-bridge

lxp-bridge is a tool to communicate with a LuxPower inverter (commonly used with home-battery and solar setups), written in Rust.

It allows you to monitor and control your inverter locally without any dependence on the manufacturer's own servers in China.

## Home Assistant add-on
Click the icon below to add this repository to your Home Assistant instance or follow the procedure highlighted on the [Home Assistant website](https://home-assistant.io/hassio/installing_third_party_addons).

[![Install lxp-bridge add-on repo.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2FEiNSTeiN-%2Flxp-bridge-eg4-18kpv)

## Pre-built images
This fork builds its own images in GitHub Container Registry. Native ARM64 and
AMD64 builds compile the checked-out source and verify the standalone bridge and
Home Assistant app before exporting image archives or publishing.

- Standalone bridge: `ghcr.io/einstein-docker/lxp-bridge-eg4-18kpv-aarch64:dev`
- Home Assistant app: `ghcr.io/einstein-docker/lxp-bridge-eg4-18kpv-addon-aarch64:dev`

Replace `aarch64` with `amd64` for x86 systems. With publishing credentials configured,
development images are published on pushes to `master`; published GitHub releases use the release tag. The release
tag must match `version` in `addon/config.yaml` before installing the stable app.
The workflow also supports manual builds, with publishing disabled by default.
Publishing requires a `GHCR_RELEASE_TOKEN` repository secret with `read:packages`
and `write:packages` access to the `EiNSTeiN-docker` organization. This is the same
namespace used by the Documentarian app; the personal GitHub account's trailing
hyphen cannot be used in a Docker image name. Builds and downloadable image
artifacts do not require publishing credentials. Without the secret, publishing
is skipped with a workflow notice and the verified archives remain available.

Home Assistant installs the prebuilt app image. Its Dockerfile uses the Home
Assistant Debian Bookworm base explicitly, so it does not depend on Supervisor's
removed `BUILD_FROM` default. ARMv7 is no longer advertised because current Home
Assistant base images support ARM64 and AMD64.

To build and verify the images locally on the target architecture:

```bash
BRIDGE_ARCH=amd64 bash ci/build-images.sh
```

Use `BRIDGE_ARCH=aarch64` on an ARM64 machine. Registry packages must be public
for anonymous pulls, or Home Assistant must have credentials authorized to read
them.

## Documentation

### Encrypted EG4 dongles

For dongles listed as **E Wi-Fi ENC**, set `tls: true` on that inverter entry.
This enables TLS 1.2 with the dongle's pre-shared-key protocol on port 8000.
The key is derived automatically from `datalog`, which must be the correct
10-character dongle serial. No cloud credentials or dongle PIN are required.
This transport was verified against an EG4 BJ-series dongle running V3.03.

```yaml
inverters:
- enabled: true
  host: 192.168.0.10
  port: 8000
  serial: "1234567890"
  datalog: "BJ12345678"
  tls: true
  heartbeats: true
  publish_holdings_on_connect: true
```

Leave `tls` omitted or `false` for older unencrypted dongles. Each inverter can
use its own transport, so an older BA dongle and an ENC dongle can run together.
TLS authentication failures are logged and retried; the bridge does not fall
back to plaintext when `tls: true` is configured.

Use one local client per ENC dongle. A connection attempt from another add-on
or application can close the existing TLS connection, even if that other client
fails authentication. When switching between the development and stable add-ons,
stop the old instance before starting the new one with the same dongle.

Full documentation is now in the [Wiki](https://github.com/celsworth/lxp-bridge/wiki).
