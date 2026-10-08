# Home Assistant addon: lxp-bridge

Allows local communication with Luxpower inverters and bridges to MQTT.

![Supports aarch64 Architecture][aarch64-shield] ![Supports amd64 Architecture][amd64-shield] ![Supports armv7 Architecture][armv7-shield]

## About

lxp-bridge is a tool to communicate with a LuxPower inverter (commonly used with home-battery and solar setups), written in Rust.

It allows you to monitor and control your inverter locally without any dependence on the manufacturer's own servers in China.

Full documentation can be found in the [Wiki](https://github.com/celsworth/lxp-bridge/wiki).

## ENC dongles

Set `tls: true` in the inverter entry for an **E Wi-Fi ENC** dongle. Port 8000
uses TLS-PSK; the bridge derives its key from the configured `datalog` serial.
No cloud credentials or dongle PIN are needed. Leave `tls` omitted or `false`
for older unencrypted dongles. Both types can be configured in the same app.

This release supports the BJ-series ENC dongle running V3.03. Ensure `serial`
is the inverter serial and `datalog` is the dongle serial; quote both values.

Use only one local client per ENC dongle. Another add-on or application can
displace its existing connection. Stop the old instance when switching between
the development and stable add-ons.


[aarch64-shield]: https://img.shields.io/badge/aarch64-yes-green.svg
[amd64-shield]: https://img.shields.io/badge/amd64-yes-green.svg
[armv7-shield]: https://img.shields.io/badge/armv7-yes-green.svg
