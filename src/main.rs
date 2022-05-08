#![warn(clippy::pedantic)]
#![allow(dead_code)]
mod handle_source;
use clap::{Parser, Subcommand};
use glob::glob;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::fs;
use std::process::Command;
use std::{collections::HashMap, path::Path};

#[derive(Parser)]
#[clap(author = "Adam Y. Cole II", version = "0.1.0", about = "the Simpler PackagE mAnageR", long_about = None)]
struct Cli {
    #[clap(short, long)]
    verbose: bool,

    #[clap(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Install a single package via a TOML package file.
    Install { package: String },
}
#[derive(Deserialize, Debug)]
struct Package {
    package_info: PackageInfo,
    configure: Configure,
    build: Build,
    install: Install,
}
#[derive(Deserialize, Debug)]
struct PackageInfo {
    name: String,
    version: String,
    from: String,
    deps: Option<HashMap<String, String>>,
    build_dir: bool,
    compression_method: String,
    after_compression: String,
}
#[derive(Deserialize, Debug)]
struct Configure {
    how_to: String,
}
#[derive(Deserialize, Debug)]
struct Build {
    how_to: String,
}
#[derive(Deserialize, Debug)]
struct Install {
    how_to: String,
    need_sudo: bool,
}
#[tokio::main]
async fn main() {
    better_panic::install();
    for dir in glob("spear_build_*").unwrap().filter_map(Result::ok) {
        fs::remove_dir_all(dir).expect("Couldn't remove old temp directories!");
    }
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { package } => {
            //Figure out what we are installing.
            let install_file = Path::new(package);
            let file_as_string =
                fs::read_to_string(install_file).expect("Couldn't read file; does it exist?");
            let to_install: Package =
                toml::from_str(&file_as_string).expect("Couldn't parse TOML; is it valid?");
            println!(
                "Installing {}, version {}!",
                to_install.package_info.name, to_install.package_info.version
            );

            //Get our source files into a temporary folder
            let mut sp = Spinner::with_timer(
                Spinners::BouncingBar,
                format!(
                    "Downloading {} from {}... ",
                    to_install.package_info.name, to_install.package_info.from
                ),
            );
            let tarball = handle_source::download_source_tarball(
                &to_install.package_info.from,
                &to_install.package_info.name,
            )
            .await
            .expect("Could not download tarball; is the url incorrect?");
            sp.stop_with_message("Done! \n".into());
            let mut sp = Spinner::with_timer(
                Spinners::BouncingBar,
                format!("Extracting {}...", &tarball[1]),
            );
            handle_source::extract_source_tarball(
                &to_install.package_info.compression_method,
                &tarball[1],
                &tarball[0],
            ).expect("Couldn't decompress tarball; was the package file misconfigured?");
            sp.stop_with_message("Done! \n".into());

            // Use TOML file to configure, make, and install code
            if to_install.package_info.build_dir {
                fs::create_dir(format!(
                    "{}/{}/build",
                    &tarball[0], to_install.package_info.after_compression
                ))
                .unwrap(); // no reason to fail here
            }

            let mut sp = Spinner::with_timer(
                Spinners::BouncingBar,
                format!(
                    "Configuring {}...",
                    &to_install.package_info.name
                ),
            );
            let mut split_command = to_install
                .configure
                .how_to
                .split(' ')
                .collect::<Vec<&str>>();
            let program = split_command[0];
            split_command.remove(0);
            if to_install.package_info.build_dir {
                Command::new(program).args(split_command).current_dir(format!(
                    "{}/{}/build",
                    &tarball[0], to_install.package_info.after_compression
                )).output().expect("Couldn't start configure script; was the package file misconfigured?");
            
            }
            sp.stop_with_message("Done! \n".into());

            let mut sp = Spinner::with_timer(
                Spinners::BouncingBar,
                format!(
                    "Compiling/building {}, this will take a long time :P",
                    to_install.package_info.name
                ),
            );
            let mut split_command = to_install
                .build
                .how_to
                .split(' ')
                .collect::<Vec<&str>>();
            let program = split_command[0];
            split_command.remove(0);
            if to_install.package_info.build_dir {
                Command::new(program).args(split_command).current_dir(format!(
                    "{}/{}/build",
                    &tarball[0], to_install.package_info.after_compression
                )).output().expect("Couldn't start build script; was the package file misconfigured?");
            
            }
            sp.stop_with_message("Done! \n".into());

        }
    }
}
