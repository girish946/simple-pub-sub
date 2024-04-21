use clap::CommandFactory;

include!("src/cli.rs");
fn main() -> Result<(), std::io::Error> {
    let out_dir_env = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    println!("OUT_DIR: {}", out_dir_env);
    let target_dir = std::path::PathBuf::from(&out_dir_env)
        .ancestors() // Iterate up through parent directories
        .nth(3) // This goes up three levels from OUT_DIR: out -> build -> debug/release -> target
        .map(|p| p.to_path_buf())
        .expect("Failed to determine target directory");

    println!("target_dir: {:?}", target_dir);

    let out_dir = std::path::PathBuf::from(target_dir);

    println!("out_dir: {:?}", out_dir);

    let mut cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    match std::fs::write(out_dir.join("simple-pub-sub.1"), buffer) {
        Ok(_) => {
            println!("file written");
        }
        Err(e) => {
            println!("error writing file: {}", e.to_string());
        }
    };

    let bash_completion_file = out_dir.join("simple-pub-sub.bash");

    let mut file = std::fs::File::create(bash_completion_file)?;

    let bin_name = "simple-pub-sub";
    clap_complete::generate(clap_complete::shells::Bash, &mut cmd, bin_name, &mut file);
    Ok(())
}
