#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
bridge_arch="${BRIDGE_ARCH:-amd64}"
bridge_version="${BRIDGE_VERSION:-dev}"
registry_namespace="${REGISTRY_NAMESPACE:-ghcr.io/einstein-docker}"
bridge_image="${registry_namespace}/lxp-bridge-eg4-18kpv-${bridge_arch}:${bridge_version}"
addon_image="${registry_namespace}/lxp-bridge-eg4-18kpv-addon-${bridge_arch}:${bridge_version}"
addon_context=addon
if [[ "$bridge_version" == dev ]]; then
    addon_context=addon.dev
fi

docker build --tag "$bridge_image" .
docker run --rm --network none --entrypoint /usr/local/bin/lxp-bridge "$bridge_image" --help
docker build --target test .
docker build --tag "$addon_image" \
    --build-arg "BUILD_VERSION=$bridge_version" \
    --build-arg "BUILD_ARCH=$bridge_arch" \
    --build-arg "BRIDGE_IMAGE=${bridge_image%:*}" "$addon_context"
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
