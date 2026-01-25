/*
 * =============================================================================
 * WinHider Build Script
 * =============================================================================
 *
 * Filename: build.rs
 * Description: Custom build script executed by Cargo before compiling the main
 * application. It handles platform-specific resource embedding for
 * Windows targets.
 *
 * Key Operations:
 * 1. Icon Embedding: Bakes the application icon (.ico) directly into the executable
 * so it appears correctly in File Explorer and the Taskbar.
 * 2. UAC Enforcement: Injects an XML manifest to strictly enforce "Run as
 * Administrator" privileges. This ensures the app triggers
 * a UAC prompt immediately upon startup.
 *
 * Dependencies:
 * - winres: Used to compile Windows resource files (.rc) and link them.
 * =============================================================================
 */

 
fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        
        // 1. Set the Icon (Keep your existing path)
        res.set_icon("../Misc/whicon-small.ico");

        // 2. Add the Manifest to force UAC (Run as Admin)
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#);

        res.compile().unwrap();
    }
}