//! Shell command validation as a defense-in-depth layer.

// ─────────────────────────────────────────────────────────────────────────────
// Shell Command Validator
// ─────────────────────────────────────────────────────────────────────────────

/// Validates shell commands before execution as a defense-in-depth layer.
///
/// This validator runs **before** OS-level sandbox enforcement. The sandbox is
/// the primary security boundary; this is an additional check to catch clearly
/// dangerous or destructive commands early with a clear error message.
///
/// Uses regex patterns for precise matching — e.g. `rm -rf /` blocks root
/// deletion but allows `rm -rf /tmp/build`.
#[derive(Debug, Clone)]
pub struct CommandValidator {
    /// Compiled regex patterns for blocked commands.
    blocked_patterns: Vec<(regex::Regex, String)>,
}

/// Result of command validation.
#[derive(Debug)]
pub enum CommandValidation {
    /// Command is allowed to proceed.
    Allowed,
    /// Command is blocked with a reason.
    Blocked(String),
}

impl Default for CommandValidator {
    fn default() -> Self {
        let patterns: Vec<(&str, &str)> = vec![
            // Destructive filesystem operations — block root only, not subdirs
            // Matches "rm -rf /" or "rm -rf /*" but not "rm -rf /tmp/build"
            (r"rm\s+-[rfRF]{2,}\s+/(\*|\s|$|;|&|\|)", "rm -rf /"),
            (r"rm\s+-[frFR]{2,}\s+/(\*|\s|$|;|&|\|)", "rm -fr /"),
            // Fork bomb
            (r":\(\)\s*\{", "fork bomb"),
            // Raw device access
            (r"dd\s+if=/dev", "dd from device"),
            (r">\s*/dev/sd[a-z]", "redirect to block device"),
            // Filesystem destruction
            (r"\bmkfs\b", "mkfs"),
            // System control — word boundary to avoid matching in paths
            (r"\bshutdown\b", "shutdown"),
            (r"\breboot\b", "reboot"),
            (r"\bhalt\b", "halt"),
            (r"\bpoweroff\b", "poweroff"),
            (r"\binit\s+[06]\b", "init 0/6"),
            // Sandbox escape attempts
            (r"\bsandbox-exec\b", "sandbox-exec"),
            (r"\bcsrutil\b", "csrutil"),
            (r"\bbwrap\b", "bwrap"),
            (r"\bunshare\b", "unshare"),
            (r"\bnsenter\b", "nsenter"),
            (r"\bchroot\b", "chroot"),
            // Kernel module manipulation
            (r"\binsmod\b", "insmod"),
            (r"\brmmod\b", "rmmod"),
            (r"\bmodprobe\b", "modprobe"),
            // Process tracing (could escape sandbox)
            (r"\bptrace\b", "ptrace"),
            (r"\bstrace\s+-p\b", "strace -p"),
            (r"\bgdb\s+--pid\b", "gdb --pid"),
            (r"\bgdb\s+-p\b", "gdb -p"),
            (r"\blldb\s+--attach\b", "lldb --attach"),
            (r"\blldb\s+-p\b", "lldb -p"),
        ];

        Self {
            blocked_patterns: patterns
                .into_iter()
                .map(|(pat, desc)| {
                    (
                        regex::Regex::new(pat).expect("invalid blocked pattern regex"),
                        desc.to_string(),
                    )
                })
                .collect(),
        }
    }
}

impl CommandValidator {
    /// Validate a shell command.
    ///
    /// Returns `Allowed` if the command passes all checks, or `Blocked` with
    /// a human-readable reason if it should be rejected.
    pub fn validate(&self, command: &str) -> CommandValidation {
        let normalized = Self::normalize(command);

        for (pattern, description) in &self.blocked_patterns {
            if pattern.is_match(&normalized) {
                return CommandValidation::Blocked(format!(
                    "Command contains blocked pattern: {}",
                    description
                ));
            }
        }

        CommandValidation::Allowed
    }

    /// Normalize a command for pattern matching.
    ///
    /// Lowercases to defeat trivial blocklist bypasses like `RM -RF /`.
    /// Whitespace is preserved for regex `\s+` matching.
    fn normalize(command: &str) -> String {
        command.to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_blocks_rm_rf_root() {
        let v = CommandValidator::default();
        assert!(matches!(
            v.validate("rm -rf /"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("rm -rf /*"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("rm -fr /"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("rm -fr /*"),
            CommandValidation::Blocked(_)
        ));
    }

    #[test]
    fn test_validator_blocks_system_control() {
        let v = CommandValidator::default();
        assert!(matches!(
            v.validate("shutdown -h now"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("reboot"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(v.validate("halt"), CommandValidation::Blocked(_)));
        assert!(matches!(
            v.validate("poweroff"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("init 0"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("init 6"),
            CommandValidation::Blocked(_)
        ));
    }

    #[test]
    fn test_validator_blocks_sandbox_escape() {
        let v = CommandValidator::default();
        assert!(matches!(
            v.validate("sandbox-exec -n noprofile bash"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("bwrap --ro-bind / / bash"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("unshare -m bash"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("nsenter -t 1 -m bash"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("chroot /tmp bash"),
            CommandValidation::Blocked(_)
        ));
    }

    #[test]
    fn test_validator_blocks_kernel_module_manipulation() {
        let v = CommandValidator::default();
        assert!(matches!(
            v.validate("insmod evil.ko"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("rmmod module"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("modprobe evil"),
            CommandValidation::Blocked(_)
        ));
    }

    #[test]
    fn test_validator_blocks_process_tracing() {
        let v = CommandValidator::default();
        assert!(matches!(
            v.validate("strace -p 1"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("gdb --pid 1234"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("gdb -p 1234"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("lldb --attach 1234"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("lldb -p 1234"),
            CommandValidation::Blocked(_)
        ));
    }

    #[test]
    fn test_validator_blocks_destructive_fs() {
        let v = CommandValidator::default();
        assert!(matches!(
            v.validate("dd if=/dev/zero of=/dev/sda"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("mkfs.ext4 /dev/sda1"),
            CommandValidation::Blocked(_)
        ));
    }

    #[test]
    fn test_validator_blocks_fork_bomb() {
        let v = CommandValidator::default();
        assert!(matches!(
            v.validate(":(){ :|:& };:"),
            CommandValidation::Blocked(_)
        ));
    }

    #[test]
    fn test_validator_normalizes_whitespace() {
        let v = CommandValidator::default();
        // Extra spaces between flags should still match
        assert!(matches!(
            v.validate("rm  -rf  /"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("rm   -rf   /*"),
            CommandValidation::Blocked(_)
        ));
        // Tabs
        assert!(matches!(
            v.validate("rm\t-rf\t/"),
            CommandValidation::Blocked(_)
        ));
    }

    #[test]
    fn test_validator_normalizes_case() {
        let v = CommandValidator::default();
        assert!(matches!(
            v.validate("RM -RF /"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("Shutdown -h now"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("REBOOT"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("MKFS.ext4 /dev/sda1"),
            CommandValidation::Blocked(_)
        ));
    }

    #[test]
    fn test_validator_allows_legitimate_commands() {
        let v = CommandValidator::default();
        assert!(matches!(v.validate("ls -la"), CommandValidation::Allowed));
        assert!(matches!(
            v.validate("echo hello world"),
            CommandValidation::Allowed
        ));
        assert!(matches!(
            v.validate("cat /tmp/file.txt"),
            CommandValidation::Allowed
        ));
        assert!(matches!(
            v.validate("grep -r pattern src/"),
            CommandValidation::Allowed
        ));
        assert!(matches!(
            v.validate("cargo build --release"),
            CommandValidation::Allowed
        ));
        assert!(matches!(
            v.validate("git status"),
            CommandValidation::Allowed
        ));
        assert!(matches!(
            v.validate("python3 script.py"),
            CommandValidation::Allowed
        ));
        assert!(matches!(
            v.validate("npm install"),
            CommandValidation::Allowed
        ));
    }

    #[test]
    fn test_validator_allows_rm_in_subdirectory() {
        let v = CommandValidator::default();
        // rm -rf on a specific subdirectory (not root) should be allowed
        assert!(matches!(
            v.validate("rm -rf /tmp/build"),
            CommandValidation::Allowed
        ));
        assert!(matches!(
            v.validate("rm -rf ./target"),
            CommandValidation::Allowed
        ));
        assert!(matches!(
            v.validate("rm -rf build/"),
            CommandValidation::Allowed
        ));
    }

    #[test]
    fn test_validator_allows_piped_commands() {
        let v = CommandValidator::default();
        assert!(matches!(
            v.validate("ls -la | grep test"),
            CommandValidation::Allowed
        ));
        assert!(matches!(
            v.validate("cat file.txt | wc -l"),
            CommandValidation::Allowed
        ));
        assert!(matches!(
            v.validate("ps aux | grep node"),
            CommandValidation::Allowed
        ));
    }

    #[test]
    fn test_validator_blocks_dangerous_in_pipe() {
        let v = CommandValidator::default();
        // Dangerous command embedded in a pipe should still be blocked
        assert!(matches!(
            v.validate("echo yes | rm -rf /"),
            CommandValidation::Blocked(_)
        ));
        assert!(matches!(
            v.validate("true && shutdown -h now"),
            CommandValidation::Blocked(_)
        ));
    }
}
