#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
bridge_arch="${BRIDGE_ARCH:-amd64}"
requested_bridge_version="${BRIDGE_VERSION:-dev}"
registry_namespace="${REGISTRY_NAMESPACE:-ghcr.io/einstein-docker}"
addon_context=addon
if [[ "$requested_bridge_version" == dev ]]; then
    addon_context=addon.dev
fi
bridge_version=$(sed -n 's/^version: //p' "$addon_context/config.yaml")
binary_version=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml)
if [[ "$bridge_version" != "v$binary_version" ]]; then
    echo "App version $bridge_version does not match bridge version $binary_version" >&2
    exit 1
fi
if [[ "$requested_bridge_version" != dev && "$requested_bridge_version" != "$bridge_version" ]]; then
    echo "Requested image version $requested_bridge_version does not match app version $bridge_version" >&2
    exit 1
fi
bridge_image="${registry_namespace}/lxp-bridge-eg4-18kpv-${bridge_arch}:${bridge_version}"
addon_image="${registry_namespace}/lxp-bridge-eg4-18kpv-addon-${bridge_arch}:${bridge_version}"

docker build --tag "$bridge_image" .
test "$(docker run --rm --network none --entrypoint /usr/local/bin/lxp-bridge "$bridge_image" --version)" = "lxp-bridge $binary_version"
docker run --rm --network none --entrypoint /usr/local/bin/lxp-bridge "$bridge_image" --help
docker build --target test .
docker build --tag "$addon_image" \
    --build-arg "BUILD_VERSION=$bridge_version" \
    --build-arg "BUILD_ARCH=$bridge_arch" \
    --build-arg "BRIDGE_IMAGE=${bridge_image%:*}" "$addon_context"
test "$(docker run --rm --network none --entrypoint /usr/local/bin/lxp-bridge "$addon_image" --version)" = "lxp-bridge $binary_version"
test "$(docker image inspect --format '{{ index .Config.Labels "io.hass.version" }}' "$addon_image")" = "$bridge_version"
docker run --rm --network none --entrypoint /usr/local/bin/lxp-bridge "$addon_image" --help

# Verify the actual app entry point and JSON options with every connection disabled.
smoke_container=""
cleanup() {
    if [[ -n "$smoke_container" ]]; then
        docker rm --force "$smoke_container" > /dev/null
    fi
}
trap cleanup EXIT
smoke_container=$(docker run --detach --network none \
    --mount "type=bind,src=$PWD/ci/smoke-options.json,dst=/data/options.json,readonly" \
    "$addon_image")
sleep 3
docker logs "$smoke_container"
smoke_state=$(docker inspect --format '{{.State.Status}}' "$smoke_container")
if [[ "$smoke_state" == running ]]; then
    docker exec "$smoke_container" test -s /etc/config.yaml
    docker stop --time 10 "$smoke_container" > /dev/null
fi
test "$(docker inspect --format '{{.State.ExitCode}}' "$smoke_container")" = 0

if [[ "$requested_bridge_version" == dev ]]; then
    docker tag "$bridge_image" "${registry_namespace}/lxp-bridge-eg4-18kpv-${bridge_arch}:dev"
    docker tag "$addon_image" "${registry_namespace}/lxp-bridge-eg4-18kpv-addon-${bridge_arch}:dev"
fi
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    echo "version=$bridge_version" >> "$GITHUB_OUTPUT"
fi
