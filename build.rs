include!("src/opt.rs");

use std::env;
extern crate bindgen;

fn main() {
    if let Ok(path) = std::env::var("COMPLETION_PATH") {
        let out_dir = PathBuf::from(path);

        gen_completion(Shell::Bash, &out_dir).unwrap();
        gen_completion(Shell::Elvish, &out_dir).unwrap();
        gen_completion(Shell::Fish, &out_dir).unwrap();
        gen_completion(Shell::PowerShell, &out_dir).unwrap();
        gen_completion(Shell::Zsh, &out_dir).unwrap();
    }

    if cfg!(target_os = "macos") {
        let bindings = bindgen::Builder::default()
            .header("src/sysctl_wrapper.h") // Change this to your header file
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
