#!/usr/bin/with-contenv bashio

set -e
umask 077

bashio::log.info "Creating lxp-bridge config from options..."

# JSON is valid YAML and can be read directly by serde_yaml.
jq . /data/options.json > /etc/config.yaml

bashio::log "Done."

bashio::log.info "Starting lxp-bridge..."

exec /usr/local/bin/lxp-bridge -c /etc/config.yaml
