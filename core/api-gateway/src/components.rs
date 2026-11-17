use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ComponentStatus {
    pub id: &'static str,
    pub name: &'static str,
    pub state: &'static str,
    pub scope: &'static str,
    pub description: &'static str,
}

/// Informational inventory of delivered foundations and future integrations.
fn component_catalog() -> Vec<ComponentStatus> {
    vec![
        ComponentStatus { id: "api-gateway", name: "API Gateway", state: "available", scope: "control-plane", description: "Axum HTTP API with health, system, instance, and node read endpoints." },
        ComponentStatus { id: "postgresql", name: "PostgreSQL", state: "optional", scope: "control-plane", description: "Persistent metadata when DATABASE_URL is configured." },
        ComponentStatus { id: "web-console", name: "Web Console", state: "available", scope: "client", description: "Responsive local console and static public demo." },
        ComponentStatus { id: "node-agent", name: "Node Agent", state: "local-only", scope: "data-plane", description: "Read-only host inventory; no remote registration or command execution." },
        ComponentStatus { id: "node-identity", name: "Node Identity", state: "preflight-only", scope: "security", description: "Local identity file preflight; does not establish mTLS." },
        ComponentStatus { id: "node-registry", name: "Node Registry", state: "available", scope: "control-plane", description: "PostgreSQL node metadata and read-only listing endpoint." },
        ComponentStatus { id: "node-protocol", name: "Node Agent Protocol", state: "planned", scope: "security", description: "Protocol specification exists; authenticated transport and handlers remain future work." },
        ComponentStatus { id: "node-mtls", name: "Node mTLS Enrollment", state: "planned", scope: "security", description: "Certificate authority, enrollment, identity binding, rotation, revocation, and heartbeats." },
        ComponentStatus { id: "user-identity", name: "User Identity", state: "available", scope: "security", description: "Protected bootstrap, Argon2id password verification, and local bearer login." },
        ComponentStatus { id: "rbac", name: "Role-Based Access Control", state: "available", scope: "security", description: "RBAC middleware protects instance, node, and operating-system image catalog routes." },
        ComponentStatus { id: "sessions", name: "Session Management", state: "available", scope: "security", description: "256-bit random bearer tokens, SHA-256 token hashes, 12-hour expiry, and revocation." },
        ComponentStatus { id: "secrets", name: "Secrets Management", state: "planned", scope: "security", description: "Encrypted secret storage, key rotation, and scoped secret access." },
        ComponentStatus { id: "audit-service", name: "Audit Log", state: "available", scope: "observability", description: "Authentication and authorization-denial events are written to append-only audit_events." },
        ComponentStatus { id: "policy-engine", name: "Policy Engine", state: "planned", scope: "security", description: "Central policy evaluation for resource ownership, permissions, and operation constraints." },
        ComponentStatus { id: "compute-api", name: "Compute API", state: "planned", scope: "control-plane", description: "Provider-neutral workload definitions and lifecycle API." },
        ComponentStatus { id: "compute-provider-catalog", name: "Compute Provider Catalog", state: "available", scope: "control-plane", description: "Read-only provider capability and prerequisite catalog; does not provision workloads." },
        ComponentStatus { id: "guest-install-profiles", name: "Guest Installation Profiles", state: "planned", scope: "control-plane", description: "Read-only Windows Server 2022/2025 installation profile metadata; no VM installation or QEMU execution." },
        ComponentStatus { id: "compute-qemu", name: "QEMU/KVM Provider", state: "planned", scope: "data-plane", description: "Virtual machine provisioning and lifecycle provider; deliberately deferred." },
        ComponentStatus { id: "containers-api", name: "Container API", state: "planned", scope: "control-plane", description: "Provider-neutral container workload definitions and lifecycle API." },
        ComponentStatus { id: "containers", name: "containerd Runtime", state: "planned", scope: "data-plane", description: "Container runtime integration; deliberately deferred." },
        ComponentStatus { id: "scheduler", name: "Resource Scheduler", state: "planned", scope: "control-plane", description: "Capacity-aware placement, eligibility, quotas, and workload scheduling." },
        ComponentStatus { id: "quotas", name: "Resource Quotas", state: "planned", scope: "control-plane", description: "Per-project CPU, memory, storage, and workload limits." },
        ComponentStatus { id: "projects", name: "Projects and Tenancy", state: "planned", scope: "control-plane", description: "Tenant boundaries, project membership, and resource ownership." },
        ComponentStatus { id: "image-service", name: "Image Service", state: "available", scope: "control-plane", description: "Authenticated OS image metadata catalog supports disk images and installer ISO records; artifact ingestion and verification are not implemented." },
        ComponentStatus { id: "image-builder", name: "Image Builder", state: "planned", scope: "control-plane", description: "Reproducible image creation, validation, signing, and publishing pipeline." },
        ComponentStatus { id: "storage-api", name: "Storage API", state: "planned", scope: "control-plane", description: "Provider-neutral volume, snapshot, backup, and attachment definitions." },
        ComponentStatus { id: "storage", name: "Storage Providers", state: "planned", scope: "data-plane", description: "Local ZFS provider first; distributed Ceph evaluation later. Deliberately deferred." },
        ComponentStatus { id: "backup-service", name: "Backup and Recovery", state: "planned", scope: "data-plane", description: "Backup policies, retention, restore workflows, and recovery verification." },
        ComponentStatus { id: "network-api", name: "Network API", state: "planned", scope: "control-plane", description: "Provider-neutral network, subnet, interface, and security-group definitions." },
        ComponentStatus { id: "networking", name: "Network Providers", state: "planned", scope: "data-plane", description: "Virtual switching, routing, firewall integration, and network isolation." },
        ComponentStatus { id: "ipam", name: "IP Address Management", state: "planned", scope: "control-plane", description: "Address pools, allocation tracking, reservations, and conflict prevention." },
        ComponentStatus { id: "dns", name: "DNS Management", state: "planned", scope: "control-plane", description: "Managed DNS zones and records for tenant workloads." },
        ComponentStatus { id: "load-balancer", name: "Load Balancing", state: "planned", scope: "data-plane", description: "Health-checked service endpoints and traffic distribution." },
        ComponentStatus { id: "metrics", name: "Metrics and Monitoring", state: "planned", scope: "observability", description: "Node and workload telemetry, health aggregation, and resource history." },
        ComponentStatus { id: "logging", name: "Centralized Logging", state: "planned", scope: "observability", description: "Structured log ingestion, indexing, retention, and access controls." },
        ComponentStatus { id: "alerting", name: "Alerting", state: "planned", scope: "observability", description: "Thresholds, alert routing, notification policies, and incident state." },
        ComponentStatus { id: "events", name: "Event Bus", state: "planned", scope: "control-plane", description: "Reliable internal events and asynchronous service coordination." },
        ComponentStatus { id: "notifications", name: "Notifications", state: "planned", scope: "control-plane", description: "User-facing operational notifications and delivery preferences." },
        ComponentStatus { id: "billing", name: "Usage and Billing", state: "planned", scope: "control-plane", description: "Usage metering and optional cost allocation; no payment system is enabled." },
        ComponentStatus { id: "api-docs", name: "API Documentation", state: "planned", scope: "developer-experience", description: "Versioned OpenAPI contract, examples, and compatibility policy." },
        ComponentStatus { id: "mobile-client", name: "Mobile Client", state: "planned", scope: "client", description: "Cross-platform Flutter management client." },
        ComponentStatus { id: "cli", name: "Command-Line Client", state: "planned", scope: "client", description: "Scriptable operator CLI with secure credential handling." },
        ComponentStatus { id: "plugin-system", name: "Provider Plugin System", state: "planned", scope: "extensibility", description: "Stable provider interfaces, capability negotiation, and compatibility checks." },
    ]
}

pub async fn list_components() -> Json<Vec<ComponentStatus>> {
    Json(component_catalog())
}

#[cfg(test)]
mod tests {
    use super::component_catalog;
    use std::collections::HashSet;

    #[test]
    fn component_ids_are_unique() {
        let components = component_catalog();
        let ids: HashSet<_> = components.iter().map(|component| component.id).collect();
        assert_eq!(ids.len(), components.len());
    }

    #[test]
    fn catalog_covers_the_platform_roadmap() {
        let components = component_catalog();
        assert!(components.len() >= 40);
        for required in [
            "rbac",
            "audit-service",
            "compute-qemu",
            "containers",
            "storage",
            "networking",
            "scheduler",
            "metrics",
        ] {
            assert!(components.iter().any(|component| component.id == required));
        }
    }

    #[test]
    fn component_states_use_known_values() {
        for component in component_catalog() {
            assert!(matches!(
                component.state,
                "available" | "optional" | "local-only" | "preflight-only" | "planned"
            ));
        }
    }
}
