// Command Registry Module
// This module declares built-in command capabilities and is the extension point for future modules.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct CommandDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub required_args: &'static [&'static str],
    pub risk: CommandRisk,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandRisk {
    ReadOnly,
    NetworkRestart,
}

const BUILTIN_COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        name: "get_context",
        description: "Return the current device context report.",
        required_args: &[],
        risk: CommandRisk::ReadOnly,
    },
    CommandDefinition {
        name: "report_context",
        description: "Alias for get_context.",
        required_args: &[],
        risk: CommandRisk::ReadOnly,
    },
    CommandDefinition {
        name: "device_describe",
        description: "Return server capabilities and extension points.",
        required_args: &[],
        risk: CommandRisk::ReadOnly,
    },
    CommandDefinition {
        name: "restart_interface",
        description: "Restart one OpenWrt network interface with ifdown/ifup.",
        required_args: &["interface"],
        risk: CommandRisk::NetworkRestart,
    },
    CommandDefinition {
        name: "reload_network",
        description: "Reload the OpenWrt network service.",
        required_args: &[],
        risk: CommandRisk::NetworkRestart,
    },
];

pub fn builtin_commands() -> &'static [CommandDefinition] {
    BUILTIN_COMMANDS
}

pub fn find_command(name: &str) -> Option<&'static CommandDefinition> {
    BUILTIN_COMMANDS
        .iter()
        .find(|definition| definition.name == name)
}

#[cfg(test)]
mod tests {
    use super::{find_command, CommandRisk};

    #[test]
    fn finds_builtin_commands() {
        let command = find_command("restart_interface").unwrap();

        assert_eq!(command.risk, CommandRisk::NetworkRestart);
        assert_eq!(command.required_args, &["interface"]);
    }

    #[test]
    fn rejects_unknown_commands() {
        assert!(find_command("shell_exec").is_none());
    }
}
