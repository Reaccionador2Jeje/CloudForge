# AtDev Billing — Hybrid plans (CloudForge)

**Status:** proposal only. No paid infrastructure, checkout, or real-money billing is enabled.

## Free-first policy

CloudForge is open-source infrastructure software. Self-hosting the software and managing infrastructure you own must remain available without a subscription. Charges may apply only to optional AtDev-operated hosted capacity, managed operations, support, or convenience services.

## Proposed hosted plans (CLP/month; exploratory)

| Plan | Reference price | Scope |
| --- | ---: | --- |
| Self-hosted | $0 | Open-source software, self-managed deployment |
| Hosted Starter | $990 | Small AtDev-hosted test environment with strict resource quotas |
| Hosted Developer | $2,490 | Larger development environment, snapshots quota, basic usage metrics |
| Managed | Quote later | Human-assisted operations; not offered until support capacity exists |

These prices are not launch prices. Compute, storage, bandwidth, backups, abuse prevention, taxes, and payment fees must be costed before any hosted plan is sold.

## Entitlements

Suggested server-side keys: `cloud.hosted.instances`, `cloud.hosted.cpu_units`, `cloud.hosted.memory_mb`, `cloud.hosted.storage_gb`, `cloud.hosted.snapshots`, `cloud.support.response_tier`.

All quotas must be enforced by the control plane. Never trust client-submitted resource limits or plan labels. On Billing outage, existing workloads must not be unexpectedly deleted or suspended; define a documented grace and recovery policy.

## Security and launch gates

- Separate customer data and infrastructure credentials; never expose provider secrets to clients.
- Meter resource usage and enforce hard spending ceilings.
- Provide export and deletion paths, clear retention terms, and abuse controls.
- Test subscription expiry, cancellation, refunds, and provider outages.
- Real-money billing remains disabled until an adult account owner has reviewed provider eligibility, legal/tax obligations, and operational costs.
