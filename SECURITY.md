# Security Policy

The **GoldSrc.rs** team takes the security and stability of game servers and WebAssembly runtimes seriously. We appreciate the responsible disclosure of any vulnerabilities found in the project.

## Supported Versions

Only the latest active development and release branches receive security updates:

| Version | Supported |
| :--- | :---: |
| `0.9.x` (dev / main) | :white_check_mark: |
| `< 0.9.0` | :x: |

## Reporting a Vulnerability

> [!IMPORTANT]
> **Please do not report security vulnerabilities through public GitHub issues, pull requests, or public discussions.**

If you discover a security vulnerability in GoldSrc.rs, please report it privately using one of the following methods:

1. **GitHub Security Advisory (Preferred):**
   - Navigate to the [Security Advisories](https://github.com/goldsrc-rs/goldsrc-rs/security/advisories) tab of this repository.
   - Click **"Report a vulnerability"** to open a confidential report directly with the maintainers.

2. **Email Disclosure:**
   - If GitHub Private Reporting is unavailable, send an encrypted or direct email to the project maintainer: [@ulquiorracode](https://github.com/ulquiorracode).

### What to Include in Your Report

To help us triage and resolve the issue quickly, please provide:

- A clear description of the vulnerability and its potential impact.
- Affected component(s) (e.g., `goldsrc-wasm-host`, `goldsrc-metamod`, `goldsrc-standalone`, `goldsrc-sys`).
- Step-by-step reproduction steps or a minimal proof-of-concept (`.wasm` plugin or script).
- Target OS and engine environment (e.g., Windows MSVC / Linux GNU, HLDS build version).

## Scope of Security Concerns

We are particularly interested in reports concerning:

- **WASM Sandbox Escapes:** Unintended execution of host system code from within a guest `.wasm` plugin.
- **Memory Safety Violations & Panics:** Unhandled panics or pointer corruption crossing C-ABI / engine boundaries.
- **Host Server Denial of Service (DoS):** Malicious plugins causing unrecoverable host server crashes without proper panic isolation.
- **Privilege Escalation:** Bypassing host management authentication or unauthorized command execution.

## Response Process

- **Acknowledgement:** We aim to acknowledge receipt of your vulnerability report within **48 hours**.
- **Assessment:** We will validate the issue, determine its severity, and keep you informed of our progress.
- **Remediation & Disclosure:** Once a fix is verified, a patched release will be published alongside a coordinated security advisory crediting your discovery (unless you request anonymity).
