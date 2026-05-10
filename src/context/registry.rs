// Context Registry Module
// This module declares context collectors and is the extension point for future collectors.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ContextCollectorDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub source: &'static str,
}

const BUILTIN_COLLECTORS: &[ContextCollectorDefinition] = &[
    ContextCollectorDefinition {
        name: "system",
        description: "Collect hostname, uptime, and CPU load from Linux procfs.",
        source: "/proc",
    },
    ContextCollectorDefinition {
        name: "interfaces",
        description: "Collect OpenWrt ubus interface data or Linux netdev fallback data.",
        source: "ubus:/sys:ip",
    },
    ContextCollectorDefinition {
        name: "wifi_clients",
        description: "Count associated Wi-Fi clients through OpenWrt hostapd ubus when available.",
        source: "ubus",
    },
];

pub fn builtin_collectors() -> &'static [ContextCollectorDefinition] {
    BUILTIN_COLLECTORS
}

#[cfg(test)]
mod tests {
    use super::builtin_collectors;

    #[test]
    fn exposes_builtin_collectors() {
        let names: Vec<&str> = builtin_collectors()
            .iter()
            .map(|collector| collector.name)
            .collect();

        assert!(names.contains(&"system"));
        assert!(names.contains(&"interfaces"));
        assert!(names.contains(&"wifi_clients"));
    }
}
