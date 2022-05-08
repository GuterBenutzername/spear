use clap::{Parser, Subcommand};
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
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { package } => {
        }
    }
}
