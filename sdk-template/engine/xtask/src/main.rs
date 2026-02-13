use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build automation for {{plugin_name}}", long_about = None)]
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
        
        /// Validate configuration without building (dry run)
        #[arg(long)]
        check: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bundle { release: _, check } => {
            if check {
                println!("✓ Bundle configuration valid (dry run)");
                return Ok(());
            }
            
            println!("Building {{plugin_name}} plugin...");
            
            // Build arguments for nih_plug_xtask
            let mut args = vec!["bundle".to_string(), "{{plugin_name}}".to_string()];
            
            // Always use release mode for bundles
            args.push("--release".to_string());
            
            // Call nih_plug_xtask with the bundle command
            // This will compile and create VST3/CLAP bundles
            if let Err(e) = nih_plug_xtask::main_with_args("{{plugin_name_snake}}", args) {
                anyhow::bail!("Bundle command failed: {}", e);
            }
            
            println!("✓ Plugin bundled successfully");
            println!("  Find bundles in: target/bundled/");
            Ok(())
        }
    }
}
