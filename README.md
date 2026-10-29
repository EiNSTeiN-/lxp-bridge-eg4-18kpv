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
Both app variants use concrete release versions. Builds verify that the app
version, compiled bridge version, and image label agree. Development builds
also retain the `dev` image aliases; exported archives contain both aliases
and the versioned tags. Bump `Cargo.toml`, `Cargo.lock`, and both app configs
together when releasing a new build so Home Assistant can detect the update.
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

### Connection diagnostics

Each configured inverter has a **Connection status** diagnostic sensor under its
existing Home Assistant MQTT device. It is separate from the inverter's existing
**Status** sensor, which reports operating modes such as Standby or PV On-grid.
The connection sensor is available whenever the bridge is online through MQTT;
it does not require an inverter connection or successful register reads.

States are `disconnected` (including the retry delay), `connecting` (TCP/DNS),
`authenticating` (TLS-PSK), `connected` (transport established), and `disabled`
(the inverter entry is disabled). `connected` does not imply fresh telemetry or
successful holding-register initialization.

The retained JSON topic is `<namespace>/<datalog>/status`. Attributes include
`state_since`, `connection_attempts`, `last_error`, `last_connected_at`,
`last_disconnected_at`, `last_packet_received_at`, `last_input_received_at`,
`initialization`, and `initialization_error`. Initialization can be `not_requested`,
`pending`, `reading`, `ready`, or `failed`; a failed read leaves the connection
state `connected` until the transport actually disconnects. The connection error
clears when a connection succeeds; initialization errors clear when a new attempt
starts. Attempt counts start over when the bridge restarts.

Status uses an independent in-memory snapshot, so telemetry queue overflow does
not discard the current connection state. Discovery and the latest status are
replayed on each MQTT broker connection, including for unreachable or disabled
inverters. Connection changes publish immediately. Packet timestamps are kept
current in memory and reported about once a minute, plus the first packet/input
and lifecycle changes, to limit Home Assistant history writes. If MQTT or the
bridge is offline, Home Assistant marks the sensor unavailable via the bridge's
existing availability topic. This requires MQTT and Home Assistant discovery to
be enabled for automatic sensor creation; the JSON status topic is also published
when discovery is disabled.

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
