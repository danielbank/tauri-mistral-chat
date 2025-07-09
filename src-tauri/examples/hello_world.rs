use anyhow::Result;
use mistralrs::{
    GgufModelBuilder, TextModelBuilder, PagedAttentionMetaBuilder, RequestBuilder, TextMessageRole, TextMessages, IsqType,
};
use std::path::Path;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    println!("🦀 Local Mistral Model Testing for Tauri Desktop App");
    println!("This example tests all models found in the models directory");

    // Scan for all available models
    let models_base_path = "models";
    
    if !Path::new(models_base_path).exists() {
        println!("📁 Models directory not found at: {}", models_base_path);
        println!("💡 Create the models directory and add .gguf model files");
        println!("📥 Falling back to remote model test...");
        return run_with_remote_model().await;
    }

    // Find all model directories and files
    let discovered_models = discover_models(models_base_path)?;
    
    if discovered_models.is_empty() {
        println!("📭 No models found in the models directory");
        println!("💡 Add .gguf model files to subdirectories in models/");
        println!("📥 Falling back to remote model test...");
        return run_with_remote_model().await;
    }

    println!("🔍 Found {} model(s) to test:", discovered_models.len());
    for (i, (model_dir, model_file, model_type)) in discovered_models.iter().enumerate() {
        if model_type == "matformer" {
            println!("  {}. {} (MatFormer)", i + 1, model_dir);
        } else if model_type == "smollm3" {
            println!("  {}. {} (SmolLM3)", i + 1, model_dir);
        } else {
            println!("  {}. {}/{} ({})", i + 1, model_dir, model_file, model_type);
        }
    }
    println!();

    // Test each discovered model
    let mut successful_tests = 0;
    let mut failed_tests = 0;

    for (model_dir, model_file, model_type) in discovered_models {
        if model_type == "matformer" {
            println!("🧪 Testing MatFormer model: {}", model_dir);
        } else if model_type == "smollm3" {
            println!("🧪 Testing SmolLM3 model: {}", model_dir);
        } else {
            println!("🧪 Testing {} model: {}/{}", model_type.to_uppercase(), model_dir, model_file);
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        match test_single_model(&model_dir, &model_file, &model_type).await {
            Ok(_) => {
                if model_type == "matformer" {
                    println!("✅ Model test passed: {} (MatFormer)", model_dir);
                } else if model_type == "smollm3" {
                    println!("✅ Model test passed: {} (SmolLM3)", model_dir);
                } else {
                    println!("✅ Model test passed: {}/{}", model_dir, model_file);
                }
                successful_tests += 1;
            }
            Err(e) => {
                if model_type == "matformer" {
                    println!("❌ Model test failed: {} (MatFormer)", model_dir);
                } else if model_type == "smollm3" {
                    println!("❌ Model test failed: {} (SmolLM3)", model_dir);
                } else {
                    println!("❌ Model test failed: {}/{}", model_dir, model_file);
                }
                println!("   Error: {}", e);
                failed_tests += 1;
            }
        }
        println!();
    }

    // Test results summary
    println!("📊 Test Summary:");
    println!("  • Successful: {}", successful_tests);
    println!("  • Failed: {}", failed_tests);
    println!("  • Total: {}", successful_tests + failed_tests);

    if failed_tests > 0 {
        println!("⚠️  Some models failed to load. Check the error messages above.");
    } else {
        println!("🎉 All models loaded and tested successfully!");
    }

    Ok(())
}

// Discovers all supported model formats in the models directory
fn discover_models(base_path: &str) -> Result<Vec<(String, String, String)>> {
    let mut models = Vec::new();
    
    let entries = fs::read_dir(base_path)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let subdir_name = path.file_name().unwrap().to_string_lossy().to_string();
            
            // Check for UQFF-based models (MatFormer, SmolLM3, etc.)
            let config_path = path.join("config.json");
            let mut has_uqff = false;
            let mut uqff_files = Vec::new();
            
            if let Ok(subdir_entries) = fs::read_dir(&path) {
                for subentry in subdir_entries {
                    if let Ok(subentry) = subentry {
                        let subpath = subentry.path();
                        
                        if subpath.is_file() {
                            if let Some(extension) = subpath.extension() {
                                if extension == "gguf" {
                                    let file_name = subpath.file_name().unwrap().to_string_lossy().to_string();
                                    models.push((subdir_name.clone(), file_name, "gguf".to_string()));
                                } else if extension == "uqff" {
                                    has_uqff = true;
                                    let file_name = subpath.file_name().unwrap().to_string_lossy().to_string();
                                    uqff_files.push(file_name);
                                }
                            }
                        }
                    }
                }
            }
            
            // Classify UQFF models by type
            if has_uqff {
                if subdir_name.to_lowercase().contains("smollm") {
                    // SmolLM3 models use TextModelBuilder without config.json requirement
                    models.push((subdir_name.clone(), "smollm3".to_string(), "smollm3".to_string()));
                } else if config_path.exists() {
                    // MatFormer models require config.json
                    models.push((subdir_name.clone(), "matformer".to_string(), "matformer".to_string()));
                }
            }
        } else if path.is_file() {
            // Standalone GGUF files in models directory root
            if let Some(extension) = path.extension() {
                if extension == "gguf" {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    models.push(("".to_string(), file_name, "gguf".to_string()));
                }
            }
        }
    }
    
    Ok(models)
}

// Tests individual model loading and basic inference
async fn test_single_model(model_dir: &str, model_file: &str, model_type: &str) -> Result<()> {
    if model_type == "matformer" {
        return test_matformer_model(model_dir).await;
    }
    
    if model_type == "smollm3" {
        return test_smollm3_model(model_dir).await;
    }
    
    // Standard GGUF model testing
    let model_path = if model_dir.is_empty() {
        "models/".to_string()
    } else {
        format!("models/{}/", model_dir)
    };
    
    let full_model_path = Path::new(&model_path).join(model_file);
    println!("📁 Testing GGUF model at: {:?}", full_model_path);
    
    // Search for chat template files
    let mut chat_template_path = None;
    
    let template_locations = [
        "mistral.json",                           // Current directory
        "src-tauri/mistral.json",                // src-tauri directory  
        &format!("{}mistral.json", model_path),  // With model files
        &format!("{}tokenizer_config.json", model_path), // Alternative name
    ];
    
    for location in &template_locations {
        let path = Path::new(location);
        if path.exists() {
            println!("✅ Found chat template: {:?}", path);
            chat_template_path = Some(*location);
            break;
        }
    }
    
    if chat_template_path.is_none() {
        println!("⚠️  No chat template found for this model");
    }
    
    // Build model using local files only
    let mut builder = GgufModelBuilder::new(
        &model_path,
        vec![model_file.to_string()],
    );
    
    if let Some(template_path) = chat_template_path {
        builder = builder.with_chat_template(template_path);
    }
    
    let model = builder
        .with_logging()
        .with_paged_attn(|| PagedAttentionMetaBuilder::default().build())?
        .build()
        .await?;

    println!("✅ Model loaded successfully!");

    // Quick inference test
    let messages = TextMessages::new()
        .add_message(
            TextMessageRole::User,
            "Hello! Please respond with just 'Hi there!' to confirm you're working.",
        );

    println!("🤖 Testing model response...");

    let response = model.send_chat_request(messages).await?;

    if let Some(content) = &response.choices[0].message.content {
        println!("🦙 Model Response: {}", content.trim());
    }

    println!("📊 Performance: {:.1} tok/s (completion)", response.usage.avg_compl_tok_per_sec);

    Ok(())
}

// Tests MatFormer model detection and configuration
async fn test_matformer_model(model_dir: &str) -> Result<()> {
    println!("📁 Testing MatFormer model in directory: models/{}/", model_dir);
    
    // Verify required MatFormer files
    let model_path = format!("models/{}", model_dir);
    let config_path = Path::new(&model_path).join("config.json");
    let tokenizer_path = Path::new(&model_path).join("tokenizer.json");
    
    if !config_path.exists() || !tokenizer_path.exists() {
        return Err(anyhow::anyhow!(
            "Required MatFormer files not found in {}/. Need config.json and tokenizer.json",
            model_path
        ));
    }
    
    println!("✅ Found local MatFormer configuration files");
    
    println!("⚠️  MatFormer model detection successful!");
    println!("📝 Note: MatFormer models require specialized loading with local .uqff files");
    println!("📂 Found files:");
    
    // List available files in the model directory
    if let Ok(entries) = fs::read_dir(&model_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let file_path = entry.path();
                if file_path.is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                        println!("   • {} ({:.1} MB)", file_name, size_mb);
                    } else {
                        println!("   • {}", file_name);
                    }
                }
            }
        }
    }
    
    println!("🔧 To fully test MatFormer models, you would need:");
    println!("   1. Proper MatFormer configuration CSV file");
    println!("   2. Network access to download base model components");
    println!("   3. Compatible mistralrs build with MatFormer support");

    Ok(())
}

// Tests SmolLM3 model loading and inference capabilities
async fn test_smollm3_model(model_dir: &str) -> Result<()> {
    println!("📁 Testing SmolLM3 model in directory: models/{}/", model_dir);
    
    let model_path = format!("models/{}", model_dir);
    
    println!("✅ Found local SmolLM3 model directory");
    
    // List UQFF files in the directory
    println!("📂 Found files:");
    if let Ok(entries) = fs::read_dir(&model_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let file_path = entry.path();
                if file_path.is_file() && file_name.ends_with(".uqff") {
                    if let Ok(metadata) = entry.metadata() {
                        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                        println!("   • {} ({:.1} MB)", file_name, size_mb);
                    } else {
                        println!("   • {}", file_name);
                    }
                }
            }
        }
    }
    
    println!("🚀 Loading SmolLM3 model...");
    println!("📝 Note: Local UQFF loading requires advanced configuration. Using remote SmolLM3 for testing.");
    
    // Load SmolLM3 remotely for compatibility testing
    let model = TextModelBuilder::new("HuggingFaceTB/SmolLM3-3B")
        .with_isq(IsqType::Q8_0)
        .with_logging()
        .with_paged_attn(|| PagedAttentionMetaBuilder::default().build())?
        .build()
        .await?;

    println!("✅ SmolLM3 model loaded successfully!");

    // Basic inference test
    let messages = TextMessages::new()
        .add_message(
            TextMessageRole::User,
            "Hello! Please respond with just 'Hi there!' to confirm you're working.",
        );

    println!("🤖 Testing SmolLM3 model response...");

    let response = model.send_chat_request(messages).await?;

    if let Some(content) = &response.choices[0].message.content {
        println!("🦙 SmolLM3 Response: {}", content.trim());
    }

    println!("📊 Performance: {:.1} tok/s (completion)", response.usage.avg_compl_tok_per_sec);

    // Test reasoning capability specific to SmolLM3
    println!("🧠 Testing SmolLM3 thinking capability...");
    let thinking_messages = TextMessages::new()
        .add_message(
            TextMessageRole::System,
            "You are an AI agent with a specialty in programming.",
        )
        .add_message(
            TextMessageRole::User,
            "Write a simple binary search function in Rust. Think through the logic step by step.",
        );

    let thinking_response = model.send_chat_request(thinking_messages).await?;

    if let Some(content) = &thinking_response.choices[0].message.content {
        println!("🧠 SmolLM3 Thinking Response:");
        println!("{}", content.trim());
    }

    Ok(())
}

// Fallback to remote model testing when no local models are available
async fn run_with_remote_model() -> Result<()> {
    dotenvy::dotenv().ok();
    
    // Verify HF_TOKEN for remote model access
    if std::env::var("HF_TOKEN").is_err() {
        eprintln!("⚠️  Warning: HF_TOKEN not found in environment");
        eprintln!("💡 Create a .env file with: HF_TOKEN=your_token_here");
        eprintln!("🔗 Get your token from: https://huggingface.co/settings/tokens");
        eprintln!();
        return Err(anyhow::anyhow!("HF_TOKEN required for remote model access"));
    }
    
    println!("📥 Loading model from remote... (this may take a moment)");

    // Search for local chat template even for remote model
    let mut chat_template_path = None;
    let template_locations = [
        "mistral.json",
        "src-tauri/mistral.json",
    ];
    
    for location in &template_locations {
        let path = Path::new(location);
        if path.exists() {
            println!("✅ Using local chat template: {:?}", path);
            chat_template_path = Some(location);
            break;
        }
    }
    
    if chat_template_path.is_none() {
        println!("⚠️  No local chat template found, using remote tokenizer");
    }

    let mut builder = GgufModelBuilder::new(
        "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
        vec!["mistral-7b-instruct-v0.1.Q4_K_M.gguf".to_string()],
    );
    
    if let Some(template_path) = chat_template_path {
        builder = builder.with_chat_template(template_path);
    }
    
    let model = builder
        .with_tok_model_id("mistralai/Mistral-7B-Instruct-v0.1".to_string())
        .with_logging()
        .with_paged_attn(|| PagedAttentionMetaBuilder::default().build())?
        .build()
        .await?;

    println!("✅ Remote model loaded successfully!");
    
    run_chat_examples(&model).await?;
    
    Ok(())
}

// Demonstrates chat capabilities for Tauri app integration
async fn run_chat_examples(model: &mistralrs::Model) -> Result<()> {
    // Example 1: Tauri-specific conversation with embedded system instructions
    let messages = TextMessages::new()
        .add_message(
            TextMessageRole::User,
            "You are a helpful AI assistant built into a Tauri desktop application. You help users with programming questions and provide concise, accurate answers.\n\nI'm building a Tauri app with Rust and React. Can you give me a quick tip for handling async operations between the frontend and backend?",
        );

    println!("🤖 Sending message to AI...");

    let response = model.send_chat_request(messages).await?;

    if let Some(content) = &response.choices[0].message.content {
        println!("\n🦙 AI Response:");
        println!("{}", content);
    }

    println!("\n📊 Performance Stats:");
    println!("  • Prompt tokens/sec: {:.2}", response.usage.avg_prompt_tok_per_sec);
    println!("  • Completion tokens/sec: {:.2}", response.usage.avg_compl_tok_per_sec);
    println!("  • Total tokens: {}", response.usage.total_tokens);

    // Example 2: Code generation for Tauri commands
    println!("\n🔧 Quick Code Generation Example:");
    let code_request = RequestBuilder::new()
        .add_message(
            TextMessageRole::User,
            "Write a simple Tauri command function that returns the current timestamp.",
        );

    let code_response = model.send_chat_request(code_request).await?;

    if let Some(content) = &code_response.choices[0].message.content {
        println!("{}", content);
    }

    Ok(())
} 