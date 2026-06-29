use clap::Parser;
use pdf_struct_chunker::{Profile, chunk_pdf};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pdf-struct-chunker")]
#[command(about = "LLM-free, layout-aware PDF chunking for RAG pipelines")]
#[command(version)]
struct Cli {
    /// Path to the input PDF file
    #[arg(short, long)]
    input: PathBuf,

    /// Optional path to a JSON profile (patterns, min/max chunk chars)
    #[arg(short, long)]
    profile: Option<PathBuf>,

    /// Output path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format: jsonl (default) or json
    #[arg(long, default_value = "jsonl")]
    format: String,

    /// Pretty-print JSON output
    #[arg(long)]
    pretty: bool,

    /// Print summary statistics instead of chunks
    #[arg(long)]
    stats: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let bytes = fs::read(&cli.input)?;

    let profile = if let Some(p) = &cli.profile {
        let content = fs::read_to_string(p)?;
        let prof: Profile = serde_json::from_str(&content)?;
        Some(prof)
    } else {
        None
    };

    let chunks = chunk_pdf(&bytes, profile.as_ref())?;

    if cli.stats {
        let mut min_size = usize::MAX;
        let mut max_size = 0;
        let mut total_size = 0;
        let mut max_page = 0;

        for chunk in &chunks {
            let len = chunk.text.len();
            if len < min_size {
                min_size = len;
            }
            if len > max_size {
                max_size = len;
            }
            total_size += len;
            if let Some(p) = chunk.metadata.page {
                if p > max_page {
                    max_page = p;
                }
            }
        }

        if chunks.is_empty() {
            min_size = 0;
        }

        let avg_size = if chunks.is_empty() {
            0
        } else {
            total_size / chunks.len()
        };

        println!("pdf-struct-chunker v{}", env!("CARGO_PKG_VERSION"));
        println!("Input:    {}", cli.input.display());
        if let Some(p) = &cli.profile {
            println!("Profile:  {}", p.display());
        }
        println!("Chunks:   {}", chunks.len());
        println!("Avg size: {} chars", avg_size);
        println!("Min size: {} chars", min_size);
        println!("Max size: {} chars", max_size);
        println!("Pages:    {}", max_page);

        return Ok(());
    }

    let out_str = if cli.format.to_lowercase() == "json" {
        if cli.pretty {
            serde_json::to_string_pretty(&chunks)?
        } else {
            serde_json::to_string(&chunks)?
        }
    } else {
        // Default: JSONL
        let mut lines = Vec::new();
        for chunk in &chunks {
            if cli.pretty {
                // Warning: pretty printing jsonl creates multi-line objects, which is technically not jsonl,
                // but we follow user instruction if they pass --pretty with jsonl.
                lines.push(serde_json::to_string_pretty(chunk)?);
            } else {
                lines.push(serde_json::to_string(chunk)?);
            }
        }
        lines.join("\n")
    };

    if let Some(out_path) = cli.output {
        fs::write(out_path, out_str)?;
    } else {
        println!("{}", out_str);
    }

    Ok(())
}
