use std::{fs, path::{Path, PathBuf}, process::exit};
use clap::{command, Parser, Subcommand};

mod ser;
mod de;
mod constants;
mod error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
#[command(rename_all = "lower")]
enum Commands {
    /// Convert Construct Classic Hash Table to JSON file
    TableToJson {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    /// Convert JSON file to Construct Classic Hash Table
    JsonToTable {
        input: PathBuf,
        output: Option<PathBuf>,
    },
}

fn main() {
    match main1() {
        Ok(_) => {
            exit(0);
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}

fn main1() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::TableToJson { input, output } => {
            let output: PathBuf = output_path(output, &input, "json");
            let bytes = fs::read(&input)?;
            let value: serde_json::Value = de::from_bytes(&bytes)?;
            let s = serde_json::to_string_pretty(&value)?;
            fs::write(&output, &s)?;
            std::eprintln!("Successfully converted \"{}\" to \"{}\"", input.to_string_lossy(), output.to_string_lossy())
        },
        Commands::JsonToTable { input, output } => {
            let output: PathBuf = output_path(output, &input, "lvl");
            let s = fs::read_to_string(&input)?;
            let value: serde_json::Value = serde_json::from_str(&s)?;
            let bytes = ser::to_bytes(&value)?;
            fs::write(&output, &bytes)?;
            std::eprintln!("Successfully converted \"{}\" to \"{}\"", input.to_string_lossy(), output.to_string_lossy())
        },
    };
    Ok(())
}

fn output_path(path: Option<PathBuf>, input_path: &Path, default_ext: &str) -> PathBuf {
    match path {
        Some(p) => p,
        // if no output path provided, infer filename with renamed extension in current directory
        None => {
            let name = input_path.file_stem()
                .map_or("".to_owned(), |o| o.to_string_lossy().to_string());
            let name = format!("{name}.{default_ext}");
            PathBuf::from_iter([".", &name])
        },
    }
}