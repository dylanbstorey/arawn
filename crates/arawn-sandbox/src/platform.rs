//! Platform detection and availability checking.

use std::fmt;
use std::process::Command;

/// Supported sandbox platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// macOS using sandbox-exec (Seatbelt).
    MacOS,
    /// Linux using bubblewrap + socat.
    Linux,
    /// Unsupported platform.
    Unsupported,
}

impl Platform {
    /// Detect the current platform.
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            Platform::MacOS
        }

        #[cfg(target_os = "linux")]
        {
            Platform::Linux
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Platform::Unsupported
        }
    }

    /// Get the display name for this platform.
    pub fn name(&self) -> &'static str {
        match self {
            Platform::MacOS => "macOS",
            Platform::Linux => "Linux",
            Platform::Unsupported => "Unsupported",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Status of sandbox availability.
#[derive(Debug, Clone)]
pub enum SandboxStatus {
    /// Sandbox is available and ready to use.
    Available { platform: Platform },

    /// Sandbox dependencies are missing.
    MissingDependency {
        platform: Platform,
        missing: Vec<String>,
        install_hint: String,
    },

    /// Platform is not supported.
    Unsupported { platform_name: String },
}

impl SandboxStatus {
    /// Check if sandbox is available.
    pub fn is_available(&self) -> bool {
        matches!(self, SandboxStatus::Available { .. })
    }

    /// Get the install hint if dependencies are missing.
    pub fn install_hint(&self) -> Option<&str> {
        match self {
            SandboxStatus::MissingDependency { install_hint, .. } => Some(install_hint),
            _ => None,
        }
    }

    /// Detect sandbox availability for the current platform.
    pub fn detect() -> Self {
        let platform = Platform::detect();

        match platform {
            Platform::MacOS => Self::check_macos(),
            Platform::Linux => Self::check_linux(),
            Platform::Unsupported => SandboxStatus::Unsupported {
                platform_name: std::env::consts::OS.to_string(),
            },
        }
    }

    /// Check macOS sandbox availability.
    fn check_macos() -> Self {
        // sandbox-exec is built into macOS, check if it exists
        let sandbox_exec_exists = Command::new("which")
            .arg("sandbox-exec")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if sandbox_exec_exists {
            SandboxStatus::Available {
                platform: Platform::MacOS,
            }
        } else {
            // This shouldn't happen on macOS, but handle it
            SandboxStatus::MissingDependency {
                platform: Platform::MacOS,
                missing: vec!["sandbox-exec".to_string()],
                install_hint: "sandbox-exec should be built into macOS. Please ensure you're running a supported macOS version.".to_string(),
            }
        }
    }

    /// Check Linux sandbox availability.
    fn check_linux() -> Self {
        let mut missing = Vec::new();

        // Check for bubblewrap
        let bwrap_exists = Command::new("which")
            .arg("bwrap")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !bwrap_exists {
            missing.push("bubblewrap".to_string());
        }

        // Check for socat (needed for network proxying)
        let socat_exists = Command::new("which")
            .arg("socat")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !socat_exists {
            missing.push("socat".to_string());
        }

        if missing.is_empty() {
            SandboxStatus::Available {
                platform: Platform::Linux,
            }
        } else {
            let install_hint = format!(
                "Shell commands require sandboxing for security.\n\
                 Install dependencies:\n\
                 \n\
                   Ubuntu/Debian: sudo apt-get install {deps}\n\
                   Fedora:        sudo dnf install {deps}\n\
                   Arch:          sudo pacman -S {deps}\n\
                   Alpine:        sudo apk add {deps}\n\
                   openSUSE:      sudo zypper install {deps}\n\
                 \n\
                 Or run the install script:\n\
                   curl -fsSL https://raw.githubusercontent.com/colliery-io/arawn/main/scripts/install.sh | sh",
                deps = missing.join(" ")
            );

            SandboxStatus::MissingDependency {
                platform: Platform::Linux,
                missing,
                install_hint,
            }
        }
    }
}

impl fmt::Display for SandboxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxStatus::Available { platform } => {
                write!(f, "Sandbox available ({platform})")
            }
            SandboxStatus::MissingDependency {
                platform,
                missing,
                install_hint,
            } => {
                write!(
                    f,
                    "Sandbox unavailable on {platform}: missing {}\n\n{install_hint}",
                    missing.join(", ")
                )
            }
            SandboxStatus::Unsupported { platform_name } => {
                write!(f, "Sandbox not supported on {platform_name}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detect() {
        let platform = Platform::detect();

        #[cfg(target_os = "macos")]
        assert_eq!(platform, Platform::MacOS);

        #[cfg(target_os = "linux")]
        assert_eq!(platform, Platform::Linux);
    }

    #[test]
    fn test_platform_name() {
        assert_eq!(Platform::MacOS.name(), "macOS");
        assert_eq!(Platform::Linux.name(), "Linux");
        assert_eq!(Platform::Unsupported.name(), "Unsupported");
    }

    #[test]
    fn test_sandbox_status_detect() {
        let status = SandboxStatus::detect();

        // On macOS, sandbox-exec should always be available
        #[cfg(target_os = "macos")]
        assert!(status.is_available());

        // On Linux, it depends on whether bwrap/socat are installed
        // We just check it doesn't panic
        let _ = status.is_available();
    }

    #[test]
    fn test_sandbox_status_display() {
        let available = SandboxStatus::Available {
            platform: Platform::MacOS,
        };
        assert!(available.to_string().contains("available"));

        let missing = SandboxStatus::MissingDependency {
            platform: Platform::Linux,
            missing: vec!["bubblewrap".to_string()],
            install_hint: "Install bubblewrap".to_string(),
        };
        assert!(missing.to_string().contains("unavailable"));
        assert!(missing.to_string().contains("bubblewrap"));
    }
}
