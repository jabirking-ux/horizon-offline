//! Embeds Horizon Offline's icon and application identity in the Windows executable.
fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let ico = format!("{dir}/../packaging/icons/horizon.ico");
    println!("cargo:rerun-if-changed={ico}");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon(&ico);
        resource.set("ProductName", "Horizon Offline");
        resource.set("FileDescription", "Horizon Offline Launcher");
        resource.set("InternalName", "horizon-offline-launcher");
        resource.set("OriginalFilename", "horizon-launcher.exe");
        resource.set("AppId", "dev.playhorizon.offline.launcher");
        if let Err(e) = resource.compile() {
            panic!("embedding {ico}: {e}");
        }
    }
}
