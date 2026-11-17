# ADR 0001: Initial CloudForge Architecture

- **Status:** Accepted
- **Date:** 2026-11-16

## Context

CloudForge needs a foundation that can grow into an infrastructure platform while remaining understandable, testable, and safe to develop incrementally.

## Decision

- Use Rust for core control-plane services and node agents.
- Use Axum and Tokio for the initial HTTP service.
- Keep the control plane separate from privileged node operations.
- Integrate established virtualization and storage technologies instead of creating a hypervisor or filesystem from scratch.
- Begin with a single API Gateway crate and add services when their boundaries and responsibilities are defined.

## Consequences

- The first milestone is not yet a production cloud service.
- Host-level virtualization operations are intentionally out of scope until authentication, authorization, auditing, and agent isolation are designed.
- Additional technology choices must be documented when they materially affect architecture or operations.
