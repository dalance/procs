use clap::CommandFactory;

include!("src/opt.rs");

fn main() {
    if let Ok(path) = std::env::var("COMPLETION_PATH") {
        let out_dir = PathBuf::from(path);

        gen_completion(Shell::Bash, &out_dir, &mut Opt::command()).unwrap();
        gen_completion(Shell::Elvish, &out_dir, &mut Opt::command()).unwrap();
        gen_completion(Shell::Fish, &out_dir, &mut Opt::command()).unwrap();
        gen_completion(Shell::PowerShell, &out_dir, &mut Opt::command()).unwrap();
        gen_completion(Shell::Zsh, &out_dir, &mut Opt::command()).unwrap();
    }

    // MAN_PATH=./man cargo build
    if let Ok(path) = std::env::var("MAN_PATH") {
        let out_dir = std::path::Path::new(&path);
        std::fs::create_dir_all(out_dir).unwrap();
        let cmd = Opt::command();
        let man = clap_mangen::Man::new(cmd);
        let mut buf = Vec::new();
        man.render(&mut buf).unwrap();
        std::fs::write(out_dir.join("procs.1"), buf).unwrap();
        println!(
            "man page is generated: {}",
            out_dir.join("procs.1").display()
        );
    }
}
