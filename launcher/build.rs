//! Embeds Horizon Offline's icon and application identity in the Windows executable.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // Resolve imports before main from System32, never a nearby injector DLL (#13108).
        println!("cargo:rustc-link-arg-bins=/DEPENDENTLOADFLAG:0x800");
        // Unoptimised builds overflow Windows' default 1 MiB main-thread stack while the window
        // builds its first frame (0xC00000FD at start); release builds keep the default.
        if std::env::var("PROFILE").as_deref() == Ok("debug") {
            println!("cargo:rustc-link-arg-bins=/STACK:16777216");
        }
    }
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
