mod download;
use clap::{Parser, Subcommand};
use glob::glob;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::fs;
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
    how_to_install: ToInstall,
}
#[derive(Deserialize, Debug)]
struct PackageInfo {
    name: String,
    version: String,
    from: String,
    deps: Option<HashMap<String, String>>,
    sudo_req: bool,
    compression_method: String,
    after_compression: String,
}
#[derive(Deserialize, Debug)]
struct ToInstall {
    to_run: String,
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
            let install_file = Path::new(package);
            
            let install_file = Path::new("test_packages/binutils.toml");
            let file_as_string =
                fs::read_to_string(install_file).expect("Couldn't read file; does it exist?");
            let to_install: Package =
                toml::from_str(&file_as_string).expect("Couldn't parse TOML; is it valid?");
            println!(
                "Installing {}, version {}!",
                to_install.package_info.name, to_install.package_info.version
            );

            let mut sp = Spinner::with_timer(
                Spinners::BouncingBar,
                format!(
                    "Downloading {} from {}... ",
                    to_install.package_info.name, to_install.package_info.from
                )
                .into(),
            );
            let tarball = download::download_source_tarball(
                to_install.package_info.from,
                to_install.package_info.name,
            )
            .await
            .expect("Could not download tarball; is the url incorrect?");
            sp.stop_with_message("Done! \n".into());
            let mut sp = Spinner::with_timer(
                Spinners::BouncingBar,
                format!("Extracting {}...", &tarball[1]).into(),
            );
            download::extract_source_tarball(
                to_install.package_info.compression_method,
                &tarball[1],
                &tarball[0],
            )
            .await
            .expect("Couldn't decompress tarball; was the package file misconfigured?");
            sp.stop_with_message("Done! \n".into());
            
        }
    }
}
