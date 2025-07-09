use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Parser)]
#[command(name = "model-downloader")]
#[command(about = "🦙🦀 Tauri Mistral Chat Example - AI Model Downloader")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available models
    List,
    /// Download a specific model
    Download {
        /// Model to download
        #[arg(value_enum)]
        model: ModelChoice,
        /// Force re-download if model exists
        #[arg(short, long)]
        force: bool,
        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
    },
    /// Download all available models
    DownloadAll {
        /// Force re-download if models exist
        #[arg(short, long)]
        force: bool,
        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
    },
    /// Show model information
    Info {
        /// Model to show info for
        #[arg(value_enum)]
        model: ModelChoice,
    },
}

// Available AI models for the mistral.rs Tauri demo
#[derive(Clone, ValueEnum, Debug, Hash, Eq, PartialEq)]
enum ModelChoice {
    /// TheBloke's Mistral 7B Instruct (GGUF format, ~4.4GB)
    MistralGguf,
    /// Llama 3.2 11B Vision Instruct (UQFF format, ~6-23GB depending on quantization)
    LlamaVision,
    /// Google Gemma 3n E2B Instruct UQFF (Multimodal: text, image, video, audio - 6B params, ~2-6GB)
    Gemma3nE2b,
    /// SmolLM3 3B (UQFF format, ~1-3GB depending on quantization)
    SmolLm3,
}

// Model metadata for downloads
struct ModelInfo {
    name: &'static str,
    description: &'static str,
    repo: &'static str,
    files: Vec<ModelFile>,
    directory: &'static str,
    format: &'static str,
    size_estimate: &'static str,
}

struct ModelFile {
    filename: &'static str,
    url: &'static str,
    description: &'static str,
    size: &'static str,
}

const BASE_DIR: &str = "src-tauri/models";

// Copy .env file for HuggingFace token access
async fn copy_env_file() -> Result<()> {
    let _ = fs::remove_file(".env").await;
    
    if Path::new("../.env").exists() {
        fs::copy("../.env", ".env").await?;
        println!("📄 Copied .env file from parent directory");
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    copy_env_file().await?;
    
    dotenvy::dotenv().ok();
    
    let cli = Cli::parse();
    let models = get_model_info();

    match cli.command {
        Commands::List => {
            print_header();
            list_models(&models);
        }
        Commands::Download { model, force, yes } => {
            print_header();
            download_model(&models, &model, force, yes).await?;
        }
        Commands::DownloadAll { force, yes } => {
            print_header();
            download_all_models(&models, force, yes).await?;
        }
        Commands::Info { model } => {
            print_header();
            show_model_info(&models, &model);
        }
    }

    Ok(())
}

fn print_header() {
    println!("🦀 Tauri Mistral Chat - AI Model Downloader");
    println!("═══════════════════════════════════════════");
    println!();
}

// Defines available AI models with download information
fn get_model_info() -> HashMap<ModelChoice, ModelInfo> {
    let mut models = HashMap::new();

    models.insert(
        ModelChoice::MistralGguf,
        ModelInfo {
            name: "Mistral 7B Instruct (GGUF)",
            description: "TheBloke's quantized GGUF format - Perfect for CPU inference",
            repo: "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
            directory: "mistral-gguf",
            format: "GGUF",
            size_estimate: "~4.4GB",
            files: vec![
                ModelFile {
                    filename: "mistral-7b-instruct-v0.1.Q4_K_M.gguf",
                    url: "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.1-GGUF/resolve/main/mistral-7b-instruct-v0.1.Q4_K_M.gguf",
                    description: "Q4_K_M quantization - balanced quality/size",
                    size: "4.37 GB",
                },
            ],
        },
    );

    models.insert(
        ModelChoice::LlamaVision,
        ModelInfo {
            name: "Llama 3.2 11B Vision Instruct",
            description: "EricB's UQFF format - Vision-capable model with multiple quantizations",
            repo: "EricB/Llama-3.2-11B-Vision-Instruct-UQFF",
            directory: "llama-vision",
            format: "UQFF",
            size_estimate: "12-17GB",
            files: vec![
                // Configuration files required for UQFF models
                ModelFile {
                    filename: "config.json",
                    url: "https://huggingface.co/EricB/Llama-3.2-11B-Vision-Instruct-UQFF/resolve/main/config.json",
                    description: "Model configuration",
                    size: "5.07 KB",
                },
                ModelFile {
                    filename: "tokenizer.json",
                    url: "https://huggingface.co/EricB/Llama-3.2-11B-Vision-Instruct-UQFF/resolve/main/tokenizer.json",
                    description: "Tokenizer configuration - REQUIRED",
                    size: "17.2 MB",
                },
                ModelFile {
                    filename: "tokenizer_config.json",
                    url: "https://huggingface.co/EricB/Llama-3.2-11B-Vision-Instruct-UQFF/resolve/main/tokenizer_config.json",
                    description: "Tokenizer configuration",
                    size: "55.8 KB",
                },
                ModelFile {
                    filename: "preprocessor_config.json",
                    url: "https://huggingface.co/EricB/Llama-3.2-11B-Vision-Instruct-UQFF/resolve/main/preprocessor_config.json",
                    description: "Preprocessor configuration",
                    size: "437 Bytes",
                },
                ModelFile {
                    filename: "generation_config.json",
                    url: "https://huggingface.co/EricB/Llama-3.2-11B-Vision-Instruct-UQFF/resolve/main/generation_config.json",
                    description: "Generation configuration",
                    size: "215 Bytes",
                },
                // Model weights required for inference
                ModelFile {
                    filename: "residual.safetensors",
                    url: "https://huggingface.co/EricB/Llama-3.2-11B-Vision-Instruct-UQFF/resolve/main/residual.safetensors",
                    description: "Residual model weights - REQUIRED",
                    size: "5.81 GB",
                },
                // Quantized model files - multiple options for different quality/size tradeoffs
                ModelFile {
                    filename: "llama3.2-vision-instruct-q4k.uqff",
                    url: "https://huggingface.co/EricB/Llama-3.2-11B-Vision-Instruct-UQFF/resolve/main/llama3.2-vision-instruct-q4k.uqff",
                    description: "Q4K quantization - good balance",
                    size: "4.37 GB",
                },
                ModelFile {
                    filename: "llama3.2-vision-instruct-q5k.uqff",
                    url: "https://huggingface.co/EricB/Llama-3.2-11B-Vision-Instruct-UQFF/resolve/main/llama3.2-vision-instruct-q5k.uqff",
                    description: "Q5K quantization - better quality",
                    size: "5.34 GB",
                },
                ModelFile {
                    filename: "llama3.2-vision-instruct-q8_0.uqff",
                    url: "https://huggingface.co/EricB/Llama-3.2-11B-Vision-Instruct-UQFF/resolve/main/llama3.2-vision-instruct-q8_0.uqff",
                    description: "Q8_0 quantization - highest quality",
                    size: "8.25 GB",
                },
            ],
        },
    );



    models.insert(
        ModelChoice::Gemma3nE2b,
        ModelInfo {
            name: "Google Gemma 3n E2B Instruct (UQFF)",
            description: "EricB's UQFF format - Multimodal model (text, image, video, audio) - 6B params",
            repo: "EricB/gemma-3n-E2B-it-UQFF",
            directory: "gemma-3n-e2b",
            format: "UQFF",
            size_estimate: "~8GB",
            files: vec![
                ModelFile {
                    filename: "config.json",
                    url: "https://huggingface.co/EricB/gemma-3n-E2B-it-UQFF/resolve/main/config.json",
                    description: "Model configuration",
                    size: "4 KB",
                },
                ModelFile {
                    filename: "tokenizer.json",
                    url: "https://huggingface.co/EricB/gemma-3n-E2B-it-UQFF/resolve/main/tokenizer.json",
                    description: "Tokenizer configuration",
                    size: "33.4 MB",
                },
                ModelFile {
                    filename: "gemma3n-e2b-it-q4k-0.uqff",
                    url: "https://huggingface.co/EricB/gemma-3n-E2B-it-UQFF/resolve/main/gemma3n-e2b-it-q4k-0.uqff",
                    description: "Q4K quantization - good balance of quality/size",
                    size: "1.74 GB",
                },
                ModelFile {
                    filename: "gemma3n-e2b-it-q8_0-0.uqff",
                    url: "https://huggingface.co/EricB/gemma-3n-E2B-it-UQFF/resolve/main/gemma3n-e2b-it-q8_0-0.uqff",
                    description: "Q8_0 quantization - higher quality",
                    size: "3.28 GB",
                },
                ModelFile {
                    filename: "residual.safetensors",
                    url: "https://huggingface.co/EricB/gemma-3n-E2B-it-UQFF/resolve/main/residual.safetensors",
                    description: "Residual model weights",
                    size: "5.77 GB",
                },
                ModelFile {
                    filename: "processor_config.json",
                    url: "https://huggingface.co/EricB/gemma-3n-E2B-it-UQFF/resolve/main/processor_config.json",
                    description: "Processor configuration",
                    size: "98 Bytes",
                },
                ModelFile {
                    filename: "preprocessor_config.json",
                    url: "https://huggingface.co/EricB/gemma-3n-E2B-it-UQFF/resolve/main/preprocessor_config.json",
                    description: "Preprocessor configuration",
                    size: "1.13 KB",
                },
                ModelFile {
                    filename: "tokenizer_config.json",
                    url: "https://huggingface.co/google/gemma-3n-E2B-it/resolve/main/tokenizer_config.json",
                    description: "Tokenizer configuration file",
                    size: "130 Bytes",
                },
            ],
        },
    );

    models.insert(
        ModelChoice::SmolLm3,
        ModelInfo {
            name: "SmolLM3 3B (UQFF)",
            description: "EricB's UQFF format - Small but powerful 3B parameter model with hybrid reasoning",
            repo: "EricB/SmolLM3-3B-UQFF",
            directory: "smollm3-3b",
            format: "UQFF",
            size_estimate: "~1-3GB",
            files: vec![
                ModelFile {
                    filename: "config.json",
                    url: "https://huggingface.co/HuggingFaceTB/SmolLM3-3B/resolve/main/config.json",
                    description: "Model configuration from base model",
                    size: "1.2 KB",
                },
                ModelFile {
                    filename: "tokenizer.json",
                    url: "https://huggingface.co/HuggingFaceTB/SmolLM3-3B/resolve/main/tokenizer.json",
                    description: "Tokenizer configuration from base model",
                    size: "17.5 MB",
                },
                ModelFile {
                    filename: "tokenizer_config.json",
                    url: "https://huggingface.co/HuggingFaceTB/SmolLM3-3B/resolve/main/tokenizer_config.json",
                    description: "Tokenizer configuration file from base model",
                    size: "2.4 KB",
                },
                ModelFile {
                    filename: "smollm33b-q4k-0.uqff",
                    url: "https://huggingface.co/EricB/SmolLM3-3B-UQFF/resolve/main/smollm33b-q4k-0.uqff",
                    description: "Q4K quantization - recommended balance",
                    size: "1.8 GB",
                },
                ModelFile {
                    filename: "smollm33b-q8_0-0.uqff",
                    url: "https://huggingface.co/EricB/SmolLM3-3B-UQFF/resolve/main/smollm33b-q8_0-0.uqff",
                    description: "Q8_0 quantization - higher quality",
                    size: "3.2 GB",
                },
                ModelFile {
                    filename: "smollm33b-afq4-0.uqff",
                    url: "https://huggingface.co/EricB/SmolLM3-3B-UQFF/resolve/main/smollm33b-afq4-0.uqff",
                    description: "AFQ4 quantization - adaptive format",
                    size: "1.9 GB",
                },
                ModelFile {
                    filename: "smollm33b-f8e4m3-0.uqff",
                    url: "https://huggingface.co/EricB/SmolLM3-3B-UQFF/resolve/main/smollm33b-f8e4m3-0.uqff",
                    description: "F8E4M3 quantization - experimental format",
                    size: "3.0 GB",
                },
            ],
        },
    );

    models
}

fn list_models(models: &HashMap<ModelChoice, ModelInfo>) {
    println!("📋 Available Models:");
    println!();

    for (choice, info) in models {
        let status_icon = if model_exists(info) { "✅" } else { "⬜" };
        println!("  {} {:?}", status_icon, choice);
        println!("     📝 {}", info.name);
        println!("     📄 {}", info.description);
        println!("     📊 Format: {} | Size: {}", info.format, info.size_estimate);
        println!("     🔗 {}", info.repo);
        println!();
    }

    println!("💡 Usage examples:");
    println!("   cargo run --example download_models download mistral-gguf");
    println!("   cargo run --example download_models download gemma3n-e2b");
    println!("   cargo run --example download_models info llama-vision");
    println!("   cargo run --example download_models download-all");
}

fn show_model_info(models: &HashMap<ModelChoice, ModelInfo>, choice: &ModelChoice) {
    if let Some(info) = models.get(choice) {
        println!("📋 Model Information: {:?}", choice);
        println!();
        println!("  📝 Name: {}", info.name);
        println!("  📄 Description: {}", info.description);
        println!("  🔗 Repository: {}", info.repo);
        println!("  📊 Format: {}", info.format);
        println!("  💾 Estimated Size: {}", info.size_estimate);
        println!("  📁 Local Directory: {}/{}", BASE_DIR, info.directory);
        println!();
        
        let status = if model_exists(info) { "✅ Downloaded" } else { "⬜ Not Downloaded" };
        println!("  Status: {}", status);
        println!();
        
        println!("  📦 Files to download:");
        for file in &info.files {
            println!("     • {} ({})", file.filename, file.size);
            println!("       {}", file.description);
        }
    }
}

// Check if all required model files are already downloaded
fn model_exists(info: &ModelInfo) -> bool {
    let model_dir = Path::new(BASE_DIR).join(info.directory);
    if !model_dir.exists() {
        return false;
    }
    
    info.files.iter().all(|file| {
        model_dir.join(file.filename).exists()
    })
}

// Download individual model with all required files
async fn download_model(
    models: &HashMap<ModelChoice, ModelInfo>,
    choice: &ModelChoice,
    force: bool,
    skip_confirmation: bool,
) -> Result<()> {
    if let Some(info) = models.get(choice) {
        println!("🎯 Selected Model: {}", info.name);
        println!("📄 {}", info.description);
        println!("📊 Total estimated size: {}", info.size_estimate);
        println!();

        let model_dir = Path::new(BASE_DIR).join(info.directory);
        
        // Check if model already exists
        if !force && model_exists(info) {
            println!("✅ Model already exists at: {:?}", model_dir);
            
            if !skip_confirmation {
                println!("🔄 Re-download? (y/N): ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("🚀 Using existing model.");
                    return Ok(());
                }
            } else {
                println!("🚀 Using existing model (use --force to re-download).");
                return Ok(());
            }
        }

        // Confirm download with user
        if !skip_confirmation {
            println!("⚠️  This will download {} files totaling approximately {}.", info.files.len(), info.size_estimate);
            println!("📁 Files will be saved to: {:?}", model_dir);
            println!();
            println!("🤔 Do you want to proceed? (y/N): ");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if !input.trim().to_lowercase().starts_with('y') {
                println!("❌ Download cancelled.");
                return Ok(());
            }
        }

        fs::create_dir_all(&model_dir).await?;
        
        // Download all required files
        println!("📥 Starting download...");
        println!();
        
        for (i, file) in info.files.iter().enumerate() {
            println!("📦 Downloading file {} of {}: {}", i + 1, info.files.len(), file.filename);
            println!("📝 {}", file.description);
            
            let file_path = model_dir.join(file.filename);
            download_file(file.url, &file_path).await?;
            
            println!("✅ Downloaded: {}", file.filename);
            println!();
        }
        
        println!("🎉 Model download complete!");
        println!("📁 Location: {:?}", model_dir);
        println!("🚀 You can now use this model in your Tauri app!");
        
    } else {
        println!("❌ Model not found: {:?}", choice);
    }
    
    Ok(())
}

// Download all available models for the demo
async fn download_all_models(
    models: &HashMap<ModelChoice, ModelInfo>,
    force: bool,
    skip_confirmation: bool,
) -> Result<()> {
    println!("🎯 Download All Models");
    println!();
    
    let total_models = models.len();
    let existing_count = models.values().filter(|info| model_exists(info)).count();
    
    println!("📊 Summary:");
    println!("   Total models: {}", total_models);
    println!("   Already downloaded: {}", existing_count);
    println!("   To download: {}", total_models - existing_count);
    
    // Calculate estimated total download size
    let total_size: f64 = models.values()
        .filter(|info| force || !model_exists(info))
        .map(|info| {
            match info.size_estimate {
                "~4.4GB" => 4.4,
                "~13GB" => 13.0,
                "12-17GB" => 14.5, // estimate middle range for Llama Vision
                "~8GB" => 8.0,
                "~1-3GB" => 2.0, // estimate middle range for SmolLM3
                _ => 5.0, // default estimate
            }
        })
        .sum();
    
    println!("   Estimated download size: ~{:.1}GB", total_size);
    println!();

    if !skip_confirmation {
        println!("⚠️  This is a large download that may take significant time and bandwidth.");
        println!("🤔 Do you want to proceed with downloading all models? (y/N): ");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("❌ Download cancelled.");
            return Ok(());
        }
    }

    // Download each model sequentially
    let choices = vec![ModelChoice::MistralGguf, ModelChoice::LlamaVision, ModelChoice::Gemma3nE2b, ModelChoice::SmolLm3];
    
    for (i, choice) in choices.iter().enumerate() {
        println!("🚀 Downloading model {} of {}", i + 1, total_models);
        download_model(models, choice, force, true).await?;
        println!();
    }
    
    println!("🎉 All models downloaded successfully!");
    println!("🚀 Your Tauri app is now ready with all available AI models!");
    
    Ok(())
}

// Downloads individual file with progress tracking
async fn download_file(url: &str, file_path: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Failed to download file: HTTP {}", response.status());
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut file = fs::File::create(file_path).await?;
    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();
    
    use futures::StreamExt;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        
        // Show progress every 100MB or at completion
        if downloaded % (100 * 1024 * 1024) == 0 || downloaded == total_size {
            let progress = if total_size > 0 {
                (downloaded as f64 / total_size as f64 * 100.0) as u32
            } else {
                0
            };
            println!("📈 Progress: {:.1} MB ({}%)", 
                downloaded as f64 / (1024.0 * 1024.0), progress);
        }
    }

    file.flush().await?;
    Ok(())
} 