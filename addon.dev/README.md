# Home Assistant addon: lxp-bridge

> This is the development variant of lxp-bridge. It reports a concrete release
> version so Home Assistant can detect updates; each release must bump that version.
> Only use this in preference to the release version if you need one of the latest unlreeased changes.

Allows local communication with Luxpower inverters and bridges to MQTT.

![Supports aarch64 Architecture][aarch64-shield] ![Supports amd64 Architecture][amd64-shield] ![Supports armv7 Architecture][armv7-shield]

## About

lxp-bridge is a tool to communicate with a LuxPower inverter (commonly used with home-battery and solar setups), written in Rust.

It allows you to monitor and control your inverter locally without any dependence on the manufacturer's own servers in China.

Full documentation can be found in the [Wiki](https://github.com/celsworth/lxp-bridge/wiki).

For an **E Wi-Fi ENC** dongle, set `tls: true` on its inverter entry. The TLS-PSK
key is derived from `datalog`; no cloud login or PIN is needed. Leave `tls`
omitted or `false` for older unencrypted dongles.


[aarch64-shield]: https://img.shields.io/badge/aarch64-yes-green.svg
[amd64-shield]: https://img.shields.io/badge/amd64-yes-green.svg
[armv7-shield]: https://img.shields.io/badge/armv7-yes-green.svg
