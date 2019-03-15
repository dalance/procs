use fs_extra::dir;
use std::fs;

fn main() -> std::io::Result<()> {
    let _ = fs::remove_dir_all("./src/columns/linux");
    let _ = fs::remove_dir_all("./src/columns/macos");
    let _ = fs::remove_dir_all("./src/columns/windows");
    let mut options = dir::CopyOptions::new();
    options.copy_inside = true;
    let _ = dir::copy("./src/columns/common", "./src/columns/linux", &options);
    let _ = dir::copy("./src/columns/common", "./src/columns/macos", &options);
    let _ = dir::copy("./src/columns/common", "./src/columns/windows", &options);
    Ok(())
}
