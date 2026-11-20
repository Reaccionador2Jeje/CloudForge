# AtDev Ecosystem — Integration Roadmap

This is a coordination plan, not a claim that integrations are already implemented.

## Operating rules
- Keep every repository independently buildable and testable.
- Integrate through documented, versioned APIs; do not couple internal database schemas.
- Label capabilities as implemented, experimental, planned, unsupported or unverified.
- Prefer read-only discovery and health checks before state-changing operations.
- Never commit credentials, tokens, private keys or production data.
- Require explicit authorization for destructive or privileged operations.

## Project map
| Project | Responsibility |
| --- | --- |
| CloudForge | Infrastructure control plane |
| AtDev Cloud | Backend-as-a-Service and project storage |
| AtDev DB | Experimental SQL engine; not a PostgreSQL replacement |
| AtDev Accounts | Identity, sessions and OIDC |
| Memingo / Memingo IDE | Programming language, runtime and editor |
| MemeCode / Memer | Coding agent and model platform |
| Walletie | Virtual credits and transaction ledger |
| NEXORA / NEXORA OS | Terminal and operating system |
| DistroBoxer / OpenWeb Linux | Linux environments and browser simulation |
| Joynexa / Atdev OS Hub / LibreCoding | User-facing platforms |
| FileRadar / ForgeKit | Local diagnostics |
| Gold & Diamond Browser / Lapislazuli WebEngine | Browser and web-engine projects |

## Workstreams
### Contracts first
- Publish API version, authentication method, request limits, error format and health endpoint for each service.
- Define ownership of user IDs and project IDs before connecting identity systems.
- Keep AtDev DB optional until SQL behavior, backups and recovery are mature.
### Reliability
- Add CI checks for formatting, builds, tests and dependency review.
- Add readiness checks where appropriate.
- Document backup, restore, migration and rollback procedures for stateful services.
### Security
- Keep secrets server-side and use least-privilege scoped credentials.
- Validate ownership on every project-scoped operation.
- Do not expose experimental services publicly without authentication, TLS and operational safeguards.
### Developer experience
- Standardize contribution guidance, local setup, configuration examples and release notes.
- Document mobile and desktop workflows separately.
- State unsupported platform capabilities explicitly.

## Session priorities
1. Confirm each repository's current build and test status.
2. Select one small, independently verifiable improvement per repository.
3. Implement cross-repository work only after the relevant API contract is documented.
4. Record changed files, commits, tests and remaining risks.

## Status vocabulary
- Implemented: present in the default branch and covered by a reproducible check.
- Experimental: present but incomplete or not production-ready.
- Planned: design exists; implementation is not present.
- Blocked: depends on an unresolved technical or external requirement.
- Unverified: not inspected or tested in the current session.