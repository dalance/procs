use std::fs;

fn main() -> std::io::Result<()> {
    if cfg!(target_os = "linux") {
        let _ = fs::copy("./src/columns/linux.rs", "./src/columns/common.rs");
    } else if cfg!(target_os = "macos") {
        let _ = fs::copy("./src/columns/macos.rs", "./src/columns/common.rs");
    } else if cfg!(target_os = "windows") {
        let _ = fs::copy("./src/columns/windows.rs", "./src/columns/common.rs");
    }
    Ok(())
}
