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

/// Describes implemented capabilities and planned integrations.
/// This registry is informational and never claims that planned services run.
pub async fn list_components() -> Json<Vec<ComponentStatus>> {
    Json(vec![
        ComponentStatus {
            id: "api-gateway",
            name: "API Gateway",
            state: "available",
            scope: "control-plane",
            description: "Axum HTTP API with health, system, instance, and node read endpoints.",
        },
        ComponentStatus {
            id: "postgresql",
            name: "PostgreSQL",
            state: "optional",
            scope: "control-plane",
            description: "Persistent instance and node metadata when DATABASE_URL is configured.",
        },
        ComponentStatus {
            id: "web-console",
            name: "Web Console",
            state: "available",
            scope: "client",
            description: "Responsive local console and static public demo with component catalog.",
        },
        ComponentStatus {
            id: "node-agent",
            name: "Node Agent",
            state: "local-only",
            scope: "data-plane",
            description: "Read-only host inventory; no remote registration, listener, or commands.",
        },
        ComponentStatus {
            id: "node-identity",
            name: "Node Identity",
            state: "preflight-only",
            scope: "security",
            description: "Local file and configuration preflight; no TLS session or authentication.",
        },
        ComponentStatus {
            id: "node-registry",
            name: "Node Registry",
            state: "available",
            scope: "control-plane",
            description: "PostgreSQL node metadata and read-only GET /v1/nodes endpoint.",
        },
        ComponentStatus {
            id: "node-protocol",
            name: "Node Agent Protocol",
            state: "planned",
            scope: "security",
            description: "Protocol specification exists; authenticated transport and message handlers are not implemented.",
        },
        ComponentStatus {
            id: "node-mtls",
            name: "Node mTLS Enrollment",
            state: "planned",
            scope: "security",
            description: "Certificate issuance, identity binding, rotation, revocation, and authenticated heartbeats.",
        },
        ComponentStatus {
            id: "identity-service",
            name: "User Identity and RBAC",
            state: "planned",
            scope: "security",
            description: "User authentication, sessions, roles, permissions, and authorization policies.",
        },
        ComponentStatus {
            id: "audit-service",
            name: "Audit Log",
            state: "planned",
            scope: "control-plane",
            description: "Tamper-aware audit events for operator and infrastructure lifecycle actions.",
        },
        ComponentStatus {
            id: "compute-qemu",
            name: "Compute / QEMU-KVM",
            state: "planned",
            scope: "data-plane",
            description: "Virtual machine definition, provisioning, lifecycle, and console integration.",
        },
        ComponentStatus {
            id: "containers",
            name: "Container Runtime",
            state: "planned",
            scope: "data-plane",
            description: "containerd integration and isolated container workload lifecycle.",
        },
        ComponentStatus {
            id: "scheduler",
            name: "Resource Scheduler",
            state: "planned",
            scope: "control-plane",
            description: "Capacity-aware placement, node eligibility, and workload scheduling.",
        },
        ComponentStatus {
            id: "image-service",
            name: "Image Service",
            state: "planned",
            scope: "control-plane",
            description: "Versioned operating-system and container image catalog and distribution.",
        },
        ComponentStatus {
            id: "storage",
            name: "Storage",
            state: "planned",
            scope: "data-plane",
            description: "Persistent volumes, snapshots, backups, and ZFS or distributed storage backends.",
        },
        ComponentStatus {
            id: "networking",
            name: "Networking",
            state: "planned",
            scope: "data-plane",
            description: "Virtual networks, address allocation, routing, and firewall policy integration.",
        },
        ComponentStatus {
            id: "metrics",
            name: "Metrics and Monitoring",
            state: "planned",
            scope: "observability",
            description: "Node and workload telemetry, health aggregation, and resource history.",
        },
        ComponentStatus {
            id: "mobile-client",
            name: "Mobile Client",
            state: "planned",
            scope: "client",
            description: "Cross-platform Flutter client for infrastructure operations.",
        },
    ])
}
