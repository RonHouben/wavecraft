use anyhow::Result;
use clap::Parser;
use std::path::Path;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build automation for my-plugin", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Bundle the plugin for distribution
    Bundle {
        /// Install to system plugin directory after bundling
        #[arg(long)]
        install: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bundle { install } => {
            let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
            
            println!("Building plugin...");
            let mut bundle = nih_plug_xtask::BundleOptions::default();
            bundle.package = Some("my-plugin");
            
            if install {
                println!("Installing to system plugin directory...");
            }
            
            nih_plug_xtask::bundle(bundle)?;
            
            println!("✓ Plugin bundled successfully");
            Ok(())
        }
    }
}
