ARG BUILD_VERSION=dev
ARG BUILD_ARCH=aarch64
ARG BRIDGE_IMAGE=ghcr.io/einstein-docker/lxp-bridge-eg4-18kpv-${BUILD_ARCH}
FROM ${BRIDGE_IMAGE}:${BUILD_VERSION} AS lxp_bridge

FROM ghcr.io/home-assistant/base-debian:bookworm
ARG BUILD_VERSION
ARG BUILD_ARCH
RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl3 libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=lxp_bridge /usr/local/bin/lxp-bridge /usr/local/bin/lxp-bridge
COPY run.sh /run.sh
RUN chmod +x /run.sh
LABEL io.hass.type="app" \
      io.hass.name="lxp-bridge-eg4-18kpv" \
      io.hass.arch="${BUILD_ARCH}" \
      io.hass.version="${BUILD_VERSION}" \
      org.opencontainers.image.source="https://github.com/EiNSTeiN-/lxp-bridge-eg4-18kpv"
CMD ["/run.sh"]
