//! Wurst Client (https://www.wurstclient.net, GPLv3) for Java Edition 26.3: one command makes a
//! Fabric instance with the bundled Wurst jar and Fabric API (from Modrinth, hash-checked).
//! Wurst changes the game for the player running it: use it where its rules allow.
use crate::{backend, instances, loaders};
use sha2::{Digest, Sha256, Sha512};
use std::fs;

pub const MINECRAFT: &str = "26.3";
/// The instance this module makes, and the only one it ever removes.
pub const INSTANCE: &str = "Wurst Client 26.3";
pub const JAR_NAME: &str = "Wurst-Client-v7.57-MC26.3.jar";
const JAR: &[u8] = include_bytes!("../assets/wurst/Wurst-Client-v7.57-MC26.3.jar");
const JAR_SHA256: &str = "b420f33e8402dfee3383654d645bd043985ac71340c58627eae66e1de287bdc4";

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The bundled jar, checked against the hash it was added with.
pub fn jar() -> Result<&'static [u8], String> {
    if hex(&Sha256::digest(JAR)) != JAR_SHA256 {
        return Err("The bundled Wurst Client jar is damaged.".into());
    }
    Ok(JAR)
}

/// Fabric API's newest build for `MINECRAFT` (file name, bytes), checked against Modrinth's SHA-512.
fn fabric_api() -> Result<(String, Vec<u8>), String> {
    let url = format!(
        "https://api.modrinth.com/v2/project/fabric-api/version?game_versions=%5B%22{MINECRAFT}%22%5D&loaders=%5B%22fabric%22%5D"
    );
    let versions: serde_json::Value =
        serde_json::from_slice(&backend::get_bytes(&url)?).map_err(|e| e.to_string())?;
    let file = versions
        .as_array()
        .and_then(|v| {
            v.iter()
                .find(|v| v["version_type"] == "release")
                .or(v.first())
        })
        .and_then(|v| v["files"].as_array())
        .and_then(|f| f.iter().find(|f| f["primary"] == true).or(f.first()))
        .ok_or("Fabric API has no build for Minecraft 26.3 yet.")?;
    let name = file["filename"]
        .as_str()
        .ok_or("Fabric API's file has no name.")?;
    // Never a path: the name becomes a file in the mods folder.
    if name.contains(['/', '\\']) || !name.ends_with(".jar") {
        return Err(format!(
            "Fabric API's file name {name:?} is not a plain jar name."
        ));
    }
    let download = file["url"]
        .as_str()
        .ok_or("Fabric API's file has no link.")?;
    let bytes = backend::get_bytes(download)?;
    let want = file["hashes"]["sha512"]
        .as_str()
        .ok_or("Fabric API's file has no hash.")?;
    if hex(&Sha512::digest(&bytes)) != want.to_ascii_lowercase() {
        return Err("Fabric API's download did not match its published hash.".into());
    }
    Ok((name.to_string(), bytes))
}

/// Makes (or completes) the instance `name` with Fabric, Fabric API and Wurst for 26.3.
pub fn install(name: &str) -> Result<instances::Instance, String> {
    let wurst = jar()?;
    // Everything that can fail on the network comes before the instance exists.
    let loader = loaders::choose("fabric", MINECRAFT, None)?;
    let (api_name, api) = fabric_api()?;
    let instance = match instances::find(name) {
        Ok(existing) => existing,
        Err(_) => instances::create_java(name, MINECRAFT)?,
    };
    let instance = instances::edit(&instance.id, |m| {
        m.java.get_or_insert_with(Default::default).loader = Some(loader)
    })?;
    let mods = instance.home.join("mods");
    fs::create_dir_all(&mods).map_err(|e| format!("{}: {e}", mods.display()))?;
    // Older builds of either are replaced, not kept beside the new one.
    for entry in fs::read_dir(&mods).map_err(|e| e.to_string())?.flatten() {
        let file = entry.file_name().to_string_lossy().to_lowercase();
        if file.ends_with(".jar")
            && (file.starts_with("wurst-client") || file.starts_with("fabric-api"))
        {
            let _ = fs::remove_file(entry.path());
        }
    }
    fs::write(mods.join(JAR_NAME), wurst).map_err(|e| e.to_string())?;
    fs::write(mods.join(api_name), api).map_err(|e| e.to_string())?;
    Ok(instance)
}

/// Whether Wurst's instance is here with the Wurst jar in its mods folder.
pub fn installed() -> bool {
    instances::find(INSTANCE).is_ok_and(|i| i.home.join("mods").join(JAR_NAME).exists())
}

/// Removes Wurst's instance (its mods, worlds and settings with it): only the one `INSTANCE` names.
pub fn uninstall() -> Result<(), String> {
    match instances::find(INSTANCE) {
        Ok(_) => instances::delete(INSTANCE).map(|_| ()),
        Err(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_bundled_jar_matches_its_hash() {
        assert!(super::jar().is_ok());
    }
}
