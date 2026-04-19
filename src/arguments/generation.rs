use crate::arguments::Arguments;
use clap::Command;
use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::Shell::{Bash, Fish, Zsh};
use std::fs::create_dir_all;
use std::path::PathBuf;
// use clap_mangen::Man;

fn target_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target")
}

// fn generate_man_pages(cmd: Command) {
//     let man_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/man");
//
//     create_dir_all(&man_dir).unwrap();
//
//     let man = Man::new(cmd);
//     let mut buffer: Vec<u8> = vec![];
//
//     man.render(&mut buffer).expect("Man page generation failed");
//     write(man_dir.join(NAME.to_owned() + ".1"), buffer).expect("Failed to write man page");
// }

fn generate_shell_completions(mut cmd: Command) {
    let comp_dir = target_dir().join("completions");

    let result_dir = create_dir_all(&comp_dir);
    if let Err(e) = result_dir {
        eprintln!("Failed to create completions directory: {e}");
    }

    for shell in [Bash, Fish, Zsh] {
        let result = generate_to(shell, &mut cmd, "prayer-times", &comp_dir);
        match result {
            Ok(_) => {
                println!(
                    "Shell completion for {} written to {}",
                    shell,
                    comp_dir.display()
                );
            }
            Err(e) => {
                eprintln!("Failed to generate shell completion: {e}");
            }
        }
    }
    println!();
}

pub fn generate() {
    let cmd = Arguments::command();

    // generate_man_pages(cmd.clone());
    generate_shell_completions(cmd);
}
