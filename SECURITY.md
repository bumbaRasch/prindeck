# Security Policy

prindeck is an unofficial control panel for the Brother MFC-L2710DW: a Tauri
desktop app with a Rust core talking to the printer over the local network.

## Supported versions

Only the latest `0.x` release receives fixes. Older `0.x` releases are not
maintained: upgrade to the current release before reporting an issue.

## Reporting a vulnerability

Use GitHub's private vulnerability reporting: open the **Security** tab of this
repository and choose **Report a vulnerability**. That channel is visible to the
maintainers only.

If the button is not available, open a regular issue that contains no details and
ask for a private channel; a maintainer will follow up there.

Please do not open a public issue, discussion or pull request that describes an
unfixed vulnerability.

## What to include

- Affected version or commit, and the platform (distribution, desktop session).
- The action that triggers the problem and what the app does instead.
- The device model and firmware version, if the printer is involved.
- Relevant logs or a packet capture with credentials removed.
- Impact: what an attacker gains and what access they need to start.

## Redaction

Printer credentials, SNMP community strings, CUPS passwords, network and MAC
addresses, hostnames and absolute home paths must be redacted before anything is
posted. Replace them with placeholders such as `<printer-ip>` or `<password>`.

## What is in scope

- The desktop app itself, its IPC boundary and the commands it exposes to the
  webview.
- Requests the app builds and sends to the device: eSCL scanning, job control,
  SNMP queries and EWS calls.
- Handling of device responses, including malformed or hostile input.
- Local file handling: where scans and the config file are written, and what is
  logged.

## Out of scope

- Brother printer firmware and the device's own web server.
- Vulnerabilities in CUPS, the kernel, or other system packages the app merely
  calls; report those to their own maintainers.
- Brother support channels: this is not a Brother product.

## Response expectations

Reports are handled on a best-effort basis by the maintainers. Every report is
read and acknowledged; there is no guaranteed response or fix deadline. Fixes are
released as a new `0.x` version, and the report is credited unless you ask
otherwise.
