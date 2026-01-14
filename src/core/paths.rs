//! Ubuntu-specific cleanup paths

use std::path::PathBuf;

/// All cleanup target paths for Ubuntu systems
#[derive(Debug, Clone)]
pub struct CleanupPaths {
    // System caches (require sudo)
    pub apt_cache: PathBuf,
    pub apt_lists: PathBuf,
    pub journal_logs: PathBuf,
    pub system_logs: PathBuf,
    pub tmp: PathBuf,
    pub var_tmp: PathBuf,

    // User caches (no sudo needed)
    pub user_cache: PathBuf,
    pub thumbnails: PathBuf,
    pub trash: PathBuf,
    pub pip_cache: PathBuf,
    pub npm_cache: PathBuf,
    pub yarn_cache: PathBuf,

    // Browser caches
    pub firefox_cache: PathBuf,
    pub chrome_cache: PathBuf,
    pub chromium_cache: PathBuf,

    // Package manager caches
    pub snap_cache: PathBuf,
    pub flatpak_cache: PathBuf,
}

impl CleanupPaths {
    /// Create paths for the current user
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        Self {
            // System paths
            apt_cache: PathBuf::from("/var/cache/apt/archives"),
            apt_lists: PathBuf::from("/var/lib/apt/lists"),
            journal_logs: PathBuf::from("/var/log/journal"),
            system_logs: PathBuf::from("/var/log"),
            tmp: PathBuf::from("/tmp"),
            var_tmp: PathBuf::from("/var/tmp"),

            // User cache paths
            user_cache: home.join(".cache"),
            thumbnails: home.join(".cache/thumbnails"),
            trash: home.join(".local/share/Trash"),
            pip_cache: home.join(".cache/pip"),
            npm_cache: home.join(".npm/_cacache"),
            yarn_cache: home.join(".cache/yarn"),

            // Browser caches
            firefox_cache: home.join(".cache/mozilla/firefox"),
            chrome_cache: home.join(".cache/google-chrome"),
            chromium_cache: home.join(".cache/chromium"),

            // Package manager caches
            snap_cache: home.join("snap"),
            flatpak_cache: home.join(".var/app"),
        }
    }

    /// Get all user-level cache paths (no sudo required)
    pub fn user_caches(&self) -> Vec<(&str, &PathBuf)> {
        vec![
            ("User Cache", &self.user_cache),
            ("Thumbnails", &self.thumbnails),
            ("Trash", &self.trash),
            ("Pip Cache", &self.pip_cache),
            ("NPM Cache", &self.npm_cache),
            ("Yarn Cache", &self.yarn_cache),
            ("Firefox Cache", &self.firefox_cache),
            ("Chrome Cache", &self.chrome_cache),
            ("Chromium Cache", &self.chromium_cache),
        ]
    }

    /// Get all system-level cache paths (require sudo)
    pub fn system_caches(&self) -> Vec<(&str, &PathBuf)> {
        vec![
            ("APT Cache", &self.apt_cache),
            ("APT Lists", &self.apt_lists),
            ("Journal Logs", &self.journal_logs),
            ("System Logs", &self.system_logs),
            ("Temp Files", &self.tmp),
            ("Var Temp", &self.var_tmp),
        ]
    }
}

impl Default for CleanupPaths {
    fn default() -> Self {
        Self::new()
    }
}

/// Development artifact patterns
#[derive(Debug, Clone)]
pub struct DevArtifacts {
    pub patterns: Vec<ArtifactPattern>,
}

#[derive(Debug, Clone)]
pub struct ArtifactPattern {
    pub name: &'static str,
    pub dir_name: &'static str,
    pub marker_files: Vec<&'static str>,
}

impl DevArtifacts {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                ArtifactPattern {
                    name: "Node.js",
                    dir_name: "node_modules",
                    marker_files: vec!["package.json"],
                },
                ArtifactPattern {
                    name: "Rust",
                    dir_name: "target",
                    marker_files: vec!["Cargo.toml"],
                },
                ArtifactPattern {
                    name: "Python venv",
                    dir_name: "venv",
                    marker_files: vec!["requirements.txt", "setup.py", "pyproject.toml"],
                },
                ArtifactPattern {
                    name: "Python .venv",
                    dir_name: ".venv",
                    marker_files: vec!["requirements.txt", "setup.py", "pyproject.toml"],
                },
                ArtifactPattern {
                    name: "Python cache",
                    dir_name: "__pycache__",
                    marker_files: vec![],
                },
                ArtifactPattern {
                    name: "Gradle",
                    dir_name: "build",
                    marker_files: vec!["build.gradle", "build.gradle.kts"],
                },
                ArtifactPattern {
                    name: "Maven",
                    dir_name: "target",
                    marker_files: vec!["pom.xml"],
                },
                ArtifactPattern {
                    name: "Next.js",
                    dir_name: ".next",
                    marker_files: vec!["next.config.js", "next.config.mjs"],
                },
                ArtifactPattern {
                    name: "Nuxt",
                    dir_name: ".nuxt",
                    marker_files: vec!["nuxt.config.js", "nuxt.config.ts"],
                },
            ],
        }
    }
}

impl Default for DevArtifacts {
    fn default() -> Self {
        Self::new()
    }
}
