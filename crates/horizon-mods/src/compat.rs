//! Mod loader detection and mutual exclusion validation.
//!
//! Enforces the hard rule: Fabric and Forge mods CANNOT be used together in the same profile.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// The loader a mod file was built for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoaderType {
    Fabric,
    Forge,
    Unknown,
}

impl ModLoaderType {
    pub fn name(self) -> &'static str {
        match self {
            ModLoaderType::Fabric => "Fabric",
            ModLoaderType::Forge => "Forge",
            ModLoaderType::Unknown => "Unknown",
        }
    }
}

/// Inspects a jar or zip file to determine if it is a Fabric or Forge mod.
pub fn inspect_mod(path: &Path) -> Result<ModLoaderType, String> {
    if path.is_dir() {
        if path.join("fabric.mod.json").is_file() || path.join("quilt.mod.json").is_file() {
            return Ok(ModLoaderType::Fabric);
        }
        if path.join("META-INF/mods.toml").is_file()
            || path.join("mcmod.info").is_file()
            || path.join("META-INF/neoforge.mods.toml").is_file()
        {
            return Ok(ModLoaderType::Forge);
        }
        return Ok(ModLoaderType::Unknown);
    }

    let file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("{}: {e}", path.display()))?;

    let mut is_fabric = false;
    let mut is_forge = false;

    for i in 0..zip.len() {
        if let Ok(entry) = zip.by_index(i) {
            let name = entry.name();
            if name == "fabric.mod.json" || name == "quilt.mod.json" {
                is_fabric = true;
            } else if name == "META-INF/mods.toml"
                || name == "mcmod.info"
                || name == "META-INF/neoforge.mods.toml"
            {
                is_forge = true;
            } else if name.starts_with("net/fabricmc/") {
                is_fabric = true;
            } else if name.starts_with("net/minecraftforge/") {
                is_forge = true;
            }
        }
    }

    if is_fabric && is_forge {
        // Multi-loader jar or hybrid (forbidden to mix): mark as conflict
        return Ok(ModLoaderType::Unknown);
    }
    if is_fabric {
        Ok(ModLoaderType::Fabric)
    } else if is_forge {
        Ok(ModLoaderType::Forge)
    } else {
        Ok(ModLoaderType::Unknown)
    }
}

/// A census of the mods found in a profile's mods directory.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileModsReport {
    pub fabric_mods: Vec<String>,
    pub forge_mods: Vec<String>,
    pub unknown_mods: Vec<String>,
}

impl ProfileModsReport {
    /// True if both Fabric and Forge mods are present in the same profile directory.
    pub fn has_fabric_and_forge_conflict(&self) -> bool {
        !self.fabric_mods.is_empty() && !self.forge_mods.is_empty()
    }

    /// Validates that Fabric and Forge mods are NOT used together, and that the mods
    /// present in this profile match any configured expected loader.
    pub fn validate(&self, expected_loader: Option<&str>) -> Result<(), String> {
        if self.has_fabric_and_forge_conflict() {
            return Err(format!(
                "MUTUAL EXCLUSION CONFLICT: You CANNOT use Fabric and Forge mods together in the same profile!\n\
                Found {} Fabric mod(s): {}\n\
                Found {} Forge mod(s): {}\n\
                Fabric and Forge mods are incompatible. Please separate them into distinct profiles (e.g. one for 'vanilla', one for 'fabric', one for 'forge').",
                self.fabric_mods.len(),
                self.fabric_mods.join(", "),
                self.forge_mods.len(),
                self.forge_mods.join(", ")
            ));
        }

        if let Some(loader) = expected_loader {
            let l = loader.to_ascii_lowercase();
            if (l == "fabric" || l == "quilt") && !self.forge_mods.is_empty() {
                return Err(format!(
                    "Profile is configured for Fabric, but Forge mod(s) were found: {}. Fabric and Forge mods cannot be used together in the same profile.",
                    self.forge_mods.join(", ")
                ));
            }
            if l == "forge" && !self.fabric_mods.is_empty() {
                return Err(format!(
                    "Profile is configured for Forge, but Fabric mod(s) were found: {}. Fabric and Forge mods cannot be used together in the same profile.",
                    self.fabric_mods.join(", ")
                ));
            }
        }

        Ok(())
    }
}

/// Scans a profile's mods directory for Fabric and Forge mods.
pub fn scan_mods_dir(dir: &Path) -> ProfileModsReport {
    let mut report = ProfileModsReport::default();
    let Ok(entries) = fs::read_dir(dir) else {
        return report;
    };

    let mut jars: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext == "jar" || ext == "zip")
                || p.is_dir()
        })
        .collect();
    jars.sort();

    for jar in jars {
        let name = jar
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        match inspect_mod(&jar) {
            Ok(ModLoaderType::Fabric) => report.fabric_mods.push(name),
            Ok(ModLoaderType::Forge) => report.forge_mods.push(name),
            _ => report.unknown_mods.push(name),
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_test_jar(path: &Path, entry_name: &str, content: &[u8]) {
        let file = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file(entry_name, options).unwrap();
        zip.write_all(content).unwrap();
        zip.finish().unwrap();
    }

    #[test]
    fn detects_fabric_and_forge_mods() {
        let dir = std::env::temp_dir().join(format!("test-compat-mods-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let fabric_jar = dir.join("fabric_mod.jar");
        make_test_jar(&fabric_jar, "fabric.mod.json", b"{\"id\":\"test_fabric\"}");

        let forge_jar = dir.join("forge_mod.jar");
        make_test_jar(&forge_jar, "META-INF/mods.toml", b"modLoader=\"javafml\"\n");

        assert_eq!(inspect_mod(&fabric_jar).unwrap(), ModLoaderType::Fabric);
        assert_eq!(inspect_mod(&forge_jar).unwrap(), ModLoaderType::Forge);

        let report = scan_mods_dir(&dir);
        assert_eq!(report.fabric_mods, vec!["fabric_mod.jar".to_string()]);
        assert_eq!(report.forge_mods, vec!["forge_mod.jar".to_string()]);
        assert!(report.has_fabric_and_forge_conflict());

        // Validate must return an error because Fabric and Forge mods are together!
        let err = report.validate(None).unwrap_err();
        assert!(err.contains("You CANNOT use Fabric and Forge mods together"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validates_loader_affinity() {
        let mut report = ProfileModsReport::default();
        report.fabric_mods.push("sodium.jar".into());

        // Fabric loader accepts fabric mods
        assert!(report.validate(Some("fabric")).is_ok());

        // Forge loader rejects fabric mods
        assert!(report.validate(Some("forge")).is_err());

        report.fabric_mods.clear();
        report.forge_mods.push("jei.jar".into());

        // Forge loader accepts forge mods
        assert!(report.validate(Some("forge")).is_ok());

        // Fabric loader rejects forge mods
        assert!(report.validate(Some("fabric")).is_err());
    }
}
