use clap::CommandFactory;
include!("src/cli.rs");
fn main() -> Result<(), std::io::Error> {
    let out_dir = std::path::PathBuf::from(
        std::env::var_os("MAN_PAGE_DIR")
            .ok_or(std::io::ErrorKind::NotFound)
            .unwrap(),
    );
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
