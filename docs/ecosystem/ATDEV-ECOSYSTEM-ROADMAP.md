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

## Repository work plans

The following companion documents were added as planning artifacts. They do not imply that the proposed tests or integrations have already been implemented.

| Repository | Work plan |
| --- | --- |
| CloudForge | This roadmap |
| atdev-db | [Integration contract](../../atdev-db/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| atdev-cloud | [Integration plan](../../atdev-cloud/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| atdev-accounts | [Identity integration](../../atdev-accounts/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| Memingo | [Runtime integration](../../Memingo/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| Memingo-IDE | [IDE integration](../../Memingo-IDE/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| Walletie | [Wallet integration](../../Walletie/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| Joynexa | [Platform integration](../../Joynexa/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| MemeCode | [Agent safety and migration](../../MemeCode/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| Memer | [Model/API integration](../../Memer/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| NEXORA | [Terminal integration](../../NEXORA/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| DistroBoxer | [Runtime integration](../../DistroBoxer/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| openweb-linux | [Simulation boundaries](../../openweb-linux/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| NEXORA-OS | [OS build integration](../../NEXORA-OS/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| FileRadar | [Read-only scan contract](../../FileRadar/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| ForgeKit | [Diagnostics contract](../../ForgeKit/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| LibreCoding | [Learning platform integration](../../LibreCoding/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| Atdev-OS-Hub | [Supabase/RLS integration](../../Atdev-OS-Hub/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| CodePaws | [Desktop build integration](../../CodePaws/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| Void-Memingo | [CLI/platform integration](../../Void-Memingo/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| Node.mgo- | [Runtime compatibility plan](../../Node.mgo-/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| Gold-Diamond-Browser | [Browser integration](../../Gold-Diamond-Browser/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| Lapislazuli-WebEngine | [Engine milestone plan](../../Lapislazuli-WebEngine/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |
| AtDev-Billing-EcoSystem- | [Billing scope and safety](../../AtDev-Billing-EcoSystem-/blob/main/docs/ECOSYSTEM-INTEGRATION.md) |

## Repositories not found during this pass

- AtDev Rooting: no repository with this exact name was visible in the account repository listing.
- GitVoid: no repository with this exact name was visible in the account repository listing. It may be hosted on another forge or under a different name.

Add these repositories to the inventory once their canonical URLs are confirmed.
