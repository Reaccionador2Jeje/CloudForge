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

pub async fn list_components() -> Json<Vec<ComponentStatus>> {
    Json(vec![
        ComponentStatus {
            id: "api-gateway",
            name: "API Gateway",
            state: "available",
            scope: "control-plane",
            description: "Axum HTTP API and process health endpoints.",
        },
        ComponentStatus {
            id: "postgresql",
            name: "PostgreSQL",
            state: "optional",
            scope: "control-plane",
            description: "Persistent instance metadata when DATABASE_URL is configured.",
        },
        ComponentStatus {
            id: "web-console",
            name: "Web Console",
            state: "available",
            scope: "client",
            description: "Local development console and public static demo.",
        },
        ComponentStatus {
            id: "node-agent",
            name: "Node Agent",
            state: "local-only",
            scope: "data-plane",
            description: "Read-only host inventory; no remote registration or commands.",
        },
        ComponentStatus {
            id: "node-identity",
            name: "Node Identity",
            state: "preflight-only",
            scope: "security",
            description: "Local configuration checks only; mTLS enrollment is not implemented.",
        },
        ComponentStatus {
            id: "compute-qemu",
            name: "Compute / QEMU-KVM",
            state: "planned",
            scope: "data-plane",
            description: "Virtual machine lifecycle management is not implemented.",
        },
        ComponentStatus {
            id: "storage",
            name: "Storage",
            state: "planned",
            scope: "data-plane",
            description: "Volumes, snapshots, and backups are not implemented.",
        },
        ComponentStatus {
            id: "networking",
            name: "Networking",
            state: "planned",
            scope: "data-plane",
            description: "Virtual networks and address management are not implemented.",
        },
    ])
}
