include!("src/opt.rs");

fn main() {
    if let Ok(path) = std::env::var("COMPLETION_PATH") {
        let out_dir = PathBuf::from(path);

        gen_completion(Shell::Bash, &out_dir).unwrap();
        gen_completion(Shell::Elvish, &out_dir).unwrap();
        gen_completion(Shell::Fish, &out_dir).unwrap();
        gen_completion(Shell::PowerShell, &out_dir).unwrap();
        gen_completion(Shell::Zsh, &out_dir).unwrap();
    }
}
