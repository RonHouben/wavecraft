use anyhow::Result;
use clap::Parser;

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
        /// Build in release mode (default)
        #[arg(long)]
        release: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bundle { release: _ } => {
            println!("Building my-plugin plugin...");
            
            // Build arguments for nih_plug_xtask
            let mut args = vec!["bundle".to_string(), "my-plugin".to_string()];
            
            // Always use release mode for bundles
            args.push("--release".to_string());
            
            // Call nih_plug_xtask with the bundle command
            // This will compile and create VST3/CLAP bundles
            if let Err(e) = nih_plug_xtask::main_with_args("my_plugin", args) {
                anyhow::bail!("Bundle command failed: {}", e);
            }
            
            println!("✓ Plugin bundled successfully");
            println!("  Find bundles in: target/bundled/");
            Ok(())
        }
    }
}
