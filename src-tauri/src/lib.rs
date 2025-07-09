use mistralrs::{
    TextMessageRole, TextMessages, VisionMessages, GgufModelBuilder, VisionModelBuilder, TextModelBuilder, UqffVisionModelBuilder, UqffTextModelBuilder, IsqType,
};
use std::sync::Arc;
use tauri::{path::BaseDirectory, Manager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::sync::OnceLock;
use anyhow::Result as AnyhowResult;

// Global model instances to avoid reloading models on each request
static MODEL_INSTANCES: OnceLock<Arc<tokio::sync::Mutex<HashMap<String, Arc<mistralrs::Model>>>>> = OnceLock::new();

// Comprehensive error handling for mistral.rs model operations
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("Model loading failed: {0}")]
    LoadingError(#[from] anyhow::Error),
    #[error("Model not found: {0}")]
    NotFound(String),
    #[error("Invalid configuration: {0}")]
    Configuration(String),
    #[error("Vision model requires image input")]
    MissingImage,
    #[error("Image processing failed: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

type ModelResult<T> = Result<T, ModelError>;

// Model metadata for the demo - supports multiple local model formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub model_type: String, // "local-gguf", "local-matformer", "remote-gguf", "remote-vision"
    pub size_estimate: Option<String>,
    pub is_available: bool,
    pub repo: Option<String>,
    pub files: Vec<String>,
    pub is_vision: bool, // Whether this model supports vision/image inputs
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Core demo function: discovers available local AI models in multiple formats
#[tauri::command]
async fn discover_models(app: tauri::AppHandle) -> Result<Vec<ModelInfo>, String> {
    println!("Discovering available models...");
    let mut models = Vec::new();
    
    // Search multiple possible locations for models directory
    let possible_paths = [
        "models",                    // When running from src-tauri directory (most common)
        "../models",                // When running from target directory  
        "src-tauri/models",         // When running from project root
    ];
    
    let mut models_base_path = None;
    for path in &possible_paths {
        if Path::new(path).exists() {
            models_base_path = Some(*path);
            println!("Found models directory at: {}", path);
            break;
        }
    }
    
    // Try to resolve using Tauri's path API as well
    if models_base_path.is_none() {
        if let Ok(app_dir) = app.path().app_data_dir() {
            let models_dir = app_dir.join("models");
            if models_dir.exists() {
                if let Some(path_str) = models_dir.to_str() {
                    let path_string = path_str.to_string();
                    println!("Found models directory at app data: {}", path_string);
                    // Note: We can't use this path easily since it would need to be static
                    // For now, we'll stick with the relative paths approach
                }
            }
        }
    }
    
    if let Some(base_path) = models_base_path {
        match discover_local_models(base_path) {
            Ok(local_models) => {
                for (model_dir, model_file, model_type) in local_models {
                    let model_id = if model_dir.is_empty() {
                        format!("local-{}", model_file.replace(".gguf", "").replace(".uqff", ""))
                    } else {
                        format!("local-{}", model_dir)
                    };
                    
                    // Generate user-friendly names and descriptions for different model types
                    let (name, description, is_vision) = if model_type == "matformer-vision" {
                        (
                            format!("{} (Vision)", model_dir),
                            "Local MatFormer vision model with .uqff files".to_string(),
                            true
                        )
                    } else if model_type == "matformer" {
                        (
                            format!("{} (MatFormer)", model_dir),
                            "Local MatFormer model with .uqff files".to_string(),
                            false
                        )
                    } else if model_type == "smollm3" {
                        (
                            format!("{} (SmolLM3)", model_dir),
                            "Local SmolLM3 3B model with UQFF files - hybrid reasoning".to_string(),
                            false
                        )
                    } else if model_type == "llama-uqff-vision" {
                        (
                            format!("{} (Vision)", model_dir),
                            "Local Llama vision model with .uqff files".to_string(),
                            true
                        )
                    } else if model_type == "llama-uqff" {
                        (
                            format!("{} (Llama)", model_dir),
                            "Local Llama model with .uqff files".to_string(),
                            false
                        )
                    } else if model_type == "gguf-vision" {
                        (
                            format!("{} (Vision)", if model_dir.is_empty() { model_file.replace(".gguf", "") } else { model_dir }),
                            "Local GGUF vision model file".to_string(),
                            true
                        )
                    } else {
                        (
                            format!("{}/{}", model_dir, model_file),
                            "Local GGUF model file".to_string(),
                            false
                        )
                    };
                    
                    models.push(ModelInfo {
                        id: model_id,
                        name,
                        description,
                        model_type: format!("local-{}", model_type),
                        size_estimate: None,
                        is_available: true,
                        repo: None,
                        files: vec![model_file.clone()],
                        is_vision,
                    });
                }
            }
            Err(e) => {
                println!("Warning: Failed to discover local models: {}", e);
            }
        }
    } else {
        println!("No models directory found. Checked paths: {:?}", possible_paths);
        println!("Current working directory: {:?}", std::env::current_dir());
    }
    
    println!("Found {} models", models.len());
    Ok(models)
}

// Helper function to find UQFF files in model directories
fn get_uqff_files(model_path: &str) -> Result<Vec<std::path::PathBuf>, String> {
    let mut uqff_files = Vec::new();
    
    match fs::read_dir(model_path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(extension) = path.extension() {
                                if extension == "uqff" {
                                    if let Some(file_name) = path.file_name() {
                                        uqff_files.push(std::path::PathBuf::from(file_name));
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => return Err(format!("Failed to read directory entry: {}", e)),
                }
            }
        }
        Err(e) => return Err(format!("Failed to read directory {}: {}", model_path, e)),
    }
    
    if uqff_files.is_empty() {
        return Err(format!("No UQFF files found in directory: {}", model_path));
    }
    
    // Sort for consistency
    uqff_files.sort();
    
    Ok(uqff_files)
}

// Scans local filesystem for different model formats (GGUF, MatFormer, UQFF)
fn discover_local_models(base_path: &str) -> Result<Vec<(String, String, String)>, Box<dyn std::error::Error>> {
    let mut models = Vec::new();
    
    let entries = fs::read_dir(base_path)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let subdir_name = path.file_name().unwrap().to_string_lossy().to_string();
            
            // Check for MatFormer models (require config.json and .uqff files)
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
                                    
                                    // Detect vision models by directory or filename patterns
                                    let is_vision_gguf = subdir_name.to_lowercase().contains("vision") || 
                                                        subdir_name.to_lowercase().contains("llama") ||
                                                        file_name.to_lowercase().contains("vision") ||
                                                        file_name.to_lowercase().contains("llama");
                                    
                                    let model_type = if is_vision_gguf { "gguf-vision" } else { "gguf" };
                                    models.push((subdir_name.clone(), file_name, model_type.to_string()));
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
            
            // Process UQFF-based models (SmolLM3, Llama, MatFormer)
            if has_uqff {
                if subdir_name.to_lowercase().contains("smollm") {
                    // SmolLM3 models use TextModelBuilder and don't need config.json
                    models.push((subdir_name.clone(), "smollm3".to_string(), "smollm3".to_string()));
                } else if subdir_name.to_lowercase().contains("llama") {
                    // Llama UQFF models (including vision models) - don't require config.json
                    let is_vision_model = subdir_name.to_lowercase().contains("vision") ||
                                        uqff_files.iter().any(|f| f.to_lowercase().contains("vision"));
                    
                    let model_type = if is_vision_model { "llama-uqff-vision" } else { "llama-uqff" };
                    models.push((subdir_name.clone(), "llama-uqff".to_string(), model_type.to_string()));
                } else if config_path.exists() {
                    // MatFormer models that need config.json
                    let is_vision_model = subdir_name.to_lowercase().contains("vision") || 
                                        subdir_name.to_lowercase().contains("gemma-3n") ||
                                        subdir_name.to_lowercase().contains("llama");
                    
                    let model_type = if is_vision_model { "matformer-vision" } else { "matformer" };
                    
                    models.push((subdir_name.clone(), "matformer".to_string(), model_type.to_string()));
                }
            }
        } else if path.is_file() {
            // Handle standalone GGUF files in models directory root
            if let Some(extension) = path.extension() {
                if extension == "gguf" {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    
                    // Detect vision models by filename patterns
                    let is_vision_gguf = file_name.to_lowercase().contains("vision") ||
                                        file_name.to_lowercase().contains("llama");
                    
                    let model_type = if is_vision_gguf { "gguf-vision" } else { "gguf" };
                    models.push(("".to_string(), file_name, model_type.to_string()));
                }
            }
        }
    }
    
    Ok(models)
}

// Main chat interface - handles both text and vision models
#[tauri::command]
async fn ai_chat(message: String, model_id: String, image_data: Option<String>, app: tauri::AppHandle) -> Result<String, String> {
    println!("AI Chat called with message: {} using model: {}", message, model_id);
    
    dotenvy::dotenv().ok();
    
    // Initialize the model instances map if not already done
    let model_instances = MODEL_INSTANCES
        .get_or_init(|| Arc::new(tokio::sync::Mutex::new(HashMap::new())))
        .clone();
    
    let mut instances = model_instances.lock().await;
    
    // Use cached model if available, otherwise load new model
    let model = if let Some(existing_model) = instances.get(&model_id) {
        println!("Using cached model: {}", model_id);
        existing_model.clone()
    } else {
        println!("Loading new model: {}", model_id);
        
        let new_model = load_model_by_id(&model_id, &app).await?;
        let model_arc = Arc::new(new_model);
        
        // Cache the model for future requests
        instances.insert(model_id.clone(), model_arc.clone());
        model_arc
    };
    
    drop(instances);

    // Handle vision vs text models differently
    let response = if model_id.contains("vision") || model_id.contains("gemma-3n") || model_id.contains("llama") {
        // Vision model processing
        if let Some(image_base64) = image_data {
            use base64::Engine;
            let image_bytes = base64::engine::general_purpose::STANDARD.decode(&image_base64)
                .map_err(|e| format!("Failed to decode image: {}", e))?;
            
            let image = image::load_from_memory(&image_bytes)
                .map_err(|e| format!("Failed to load image: {}", e))?;
            
            // Create vision messages with image and text
            let messages = VisionMessages::new().add_image_message(
                TextMessageRole::User,
                &message,
                vec![image],
                &model,
            ).map_err(|e| format!("Failed to create vision message: {}", e))?;
            
            model
                .send_chat_request(messages)
                .await
                .map_err(|e| format!("Failed to send vision chat request: {}", e))?
        } else {
            return Err("Vision model requires an image input".to_string());
        }
    } else {
        // Text-only model processing
        let messages = TextMessages::new()
            .add_message(
                TextMessageRole::User,
                &format!("You are a helpful AI assistant. Keep your responses concise and friendly.\n\n{}", message)
            );

        model
            .send_chat_request(messages)
            .await
            .map_err(|e| format!("Failed to send text chat request: {}", e))?
    };

    // Extract response content
    let content = response.choices[0]
        .message
        .content
        .as_ref()
        .ok_or("No content in response")?
        .clone();

    println!("AI Response: {}", content);
    Ok(content)
}

// Routes model loading to appropriate builder based on model ID
async fn load_model_by_id(model_id: &str, app: &tauri::AppHandle) -> Result<mistralrs::Model, String> {
    if model_id == "mistral-7b-remote" {
        return load_remote_mistral_model(app).await;
    }
    
    if model_id == "smollm3-remote" {
        return load_remote_smollm3_model().await;
    }
    
    if model_id.starts_with("local-") {
        return load_local_model(model_id, app).await;
    }
    
    Err(format!("Unknown model ID: {}", model_id))
}

// Example remote model loading (requires HF_TOKEN)
async fn load_remote_mistral_model(app: &tauri::AppHandle) -> Result<mistralrs::Model, String> {
    println!("Loading remote Mistral 7B model...");
    
    if std::env::var("HF_TOKEN").is_err() {
        return Err("HF_TOKEN not found. Set HF_TOKEN in .env file for remote model access".to_string());
    }
    
    // Try to find local chat template
    let mut mistral_json_path = None;
    
    if let Ok(resource_path) = app.path().resolve("mistral.json", BaseDirectory::Resource) {
        if resource_path.exists() {
            mistral_json_path = Some(resource_path);
        }
    }
    
    if mistral_json_path.is_none() {
        let dev_path = std::path::Path::new("mistral.json");
        if dev_path.exists() {
            mistral_json_path = Some(dev_path.to_path_buf());
        }
    }
    
    if mistral_json_path.is_none() {
        let src_tauri_path = std::path::Path::new("src-tauri/mistral.json");
        if src_tauri_path.exists() {
            mistral_json_path = Some(src_tauri_path.to_path_buf());
        }
    }
    
    // Build the remote model with optional local chat template
    let model = if let Some(template_path) = mistral_json_path {
        println!("Using local chat template: {:?}", template_path);
        GgufModelBuilder::new(
            "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
            vec!["mistral-7b-instruct-v0.1.Q4_K_M.gguf".to_string()],
        )
        .with_chat_template(template_path.to_str().unwrap())
        .build()
        .await
    } else {
        println!("Using remote tokenizer");
        GgufModelBuilder::new(
            "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
            vec!["mistral-7b-instruct-v0.1.Q4_K_M.gguf".to_string()],
        )
        .with_tok_model_id("mistralai/Mistral-7B-Instruct-v0.1".to_string())
        .build()
        .await
    }
    .map_err(|e: anyhow::Error| format!("Failed to build remote model: {}", e))?;
    
    println!("Remote model loaded successfully!");
    Ok(model)
}

async fn load_remote_smollm3_model() -> Result<mistralrs::Model, String> {
    println!("Loading remote SmolLM3 3B model...");
    
    // Build the remote SmolLM3 model using TextModelBuilder
    let model = TextModelBuilder::new("HuggingFaceTB/SmolLM3-3B")
        .with_isq(IsqType::Q8_0)
        .with_logging()
        .build()
        .await
        .map_err(|e: anyhow::Error| format!("Failed to build remote SmolLM3 model: {}", e))?;
    
    println!("Remote SmolLM3 model loaded successfully!");
    Ok(model)
}

// Loads local models using appropriate mistral.rs builders for each format
async fn load_local_model(model_id: &str, _app: &tauri::AppHandle) -> Result<mistralrs::Model, String> {
    println!("Loading local model: {}", model_id);
    
    // Find the models directory using the same logic as discover_models
    let possible_paths = [
        "models",
        "../models",
        "src-tauri/models",
    ];
    
    let mut models_base_path = None;
    for path in &possible_paths {
        if Path::new(path).exists() {
            models_base_path = Some(*path);
            break;
        }
    }
    
    let base_path = models_base_path.ok_or("No models directory found")?;
    
    let discovered_models = discover_local_models(base_path)
        .map_err(|e| format!("Failed to discover local models: {}", e))?;
    
    // Find the matching model and load with appropriate builder
    for (model_dir, model_file, model_type) in discovered_models {
        let expected_id = if model_dir.is_empty() {
            format!("local-{}", model_file.replace(".gguf", "").replace(".uqff", ""))
        } else {
            format!("local-{}", model_dir)
        };
        
        if expected_id == model_id {
            if model_type == "matformer-vision" {
                // MatFormer vision model using VisionModelBuilder
                let model_path = format!("{}/{}", base_path, model_dir);
                
                println!("Loading MatFormer vision model from: {}", model_path);
                
                let model = VisionModelBuilder::new(&model_path)
                    .with_isq(IsqType::Q4K)
                    .with_logging()
                    .build()
                    .await
                    .map_err(|e: anyhow::Error| format!("Failed to build MatFormer vision model: {}", e))?;
                
                println!("MatFormer vision model loaded successfully!");
                return Ok(model);
            }
            
            if model_type == "gguf-vision" {
                // GGUF vision model using GgufModelBuilder
                let model_path = if model_dir.is_empty() {
                    format!("{}/", base_path)
                } else {
                    format!("{}/{}/", base_path, model_dir)
                };
                
                println!("Loading GGUF vision model from: {}{}", model_path, model_file);
                
                // Look for chat template files
                let mut chat_template_path = None;
                let template_locations = [
                    "mistral.json",
                    "src-tauri/mistral.json",
                    &format!("{}mistral.json", model_path),
                    &format!("{}tokenizer_config.json", model_path),
                ];
                
                for location in &template_locations {
                    let path = Path::new(location);
                    if path.exists() {
                        chat_template_path = Some(*location);
                        break;
                    }
                }
                
                let mut builder = GgufModelBuilder::new(
                    &model_path,
                    vec![model_file.to_string()],
                );
                
                if let Some(template_path) = chat_template_path {
                    builder = builder.with_chat_template(template_path);
                }
                
                let model = builder
                    .build()
                    .await
                    .map_err(|e: anyhow::Error| format!("Failed to build GGUF vision model: {}", e))?;
                
                println!("GGUF vision model loaded successfully!");
                return Ok(model);
            }
            
            if model_type == "smollm3" {
                // SmolLM3 model loaded remotely for better compatibility
                println!("Loading SmolLM3 model remotely (local UQFF files detected but using remote for compatibility)");
                
                let model = TextModelBuilder::new("HuggingFaceTB/SmolLM3-3B")
                    .with_isq(IsqType::Q8_0)
                    .with_logging()
                    .build()
                    .await
                    .map_err(|e: anyhow::Error| format!("Failed to build SmolLM3 model: {}", e))?;
                
                println!("SmolLM3 model loaded successfully!");
                return Ok(model);
            }
            
            if model_type == "llama-uqff-vision" {
                // Llama UQFF vision model using UqffVisionModelBuilder
                let model_path = format!("{}/{}", base_path, model_dir);
                
                let uqff_files = get_uqff_files(&model_path)
                    .map_err(|e| format!("Failed to get UQFF files: {}", e))?;
                
                println!("Loading Llama UQFF vision model from: {} with files: {:?}", model_path, uqff_files);
                
                let model = UqffVisionModelBuilder::new(&model_path, uqff_files)
                    .into_inner()
                    .with_isq(IsqType::Q5_0)
                    .with_logging()
                    .build()
                    .await
                    .map_err(|e: anyhow::Error| format!("Failed to build Llama UQFF vision model: {}", e))?;
                
                println!("Llama UQFF vision model loaded successfully!");
                return Ok(model);
            }
            
            if model_type == "llama-uqff" {
                // Llama UQFF text model using UqffTextModelBuilder
                let model_path = format!("{}/{}", base_path, model_dir);
                
                let uqff_files = get_uqff_files(&model_path)
                    .map_err(|e| format!("Failed to get UQFF files: {}", e))?;
                
                println!("Loading Llama UQFF text model from: {} with files: {:?}", model_path, uqff_files);
                
                let model = UqffTextModelBuilder::new(&model_path, uqff_files)
                    .into_inner()
                    .with_isq(IsqType::Q5_0)
                    .with_logging()
                    .build()
                    .await
                    .map_err(|e: anyhow::Error| format!("Failed to build Llama UQFF text model: {}", e))?;
                
                println!("Llama UQFF text model loaded successfully!");
                return Ok(model);
            }
            
            if model_type == "matformer" {
                return Err("MatFormer text models are not yet fully supported in this version".to_string());
            }
            
            // Standard GGUF model using GgufModelBuilder
            let model_path = if model_dir.is_empty() {
                format!("{}/", base_path)
            } else {
                format!("{}/{}/", base_path, model_dir)
            };
            
            println!("Loading GGUF model from: {}{}", model_path, model_file);
            
            // Look for chat template files
            let mut chat_template_path = None;
            let template_locations = [
                "mistral.json",
                "src-tauri/mistral.json",
                &format!("{}mistral.json", model_path),
                &format!("{}tokenizer_config.json", model_path),
            ];
            
            for location in &template_locations {
                let path = Path::new(location);
                if path.exists() {
                    chat_template_path = Some(*location);
                    break;
                }
            }
            
            let mut builder = GgufModelBuilder::new(
                &model_path,
                vec![model_file.to_string()],
            );
            
            if let Some(template_path) = chat_template_path {
                builder = builder.with_chat_template(template_path);
            }
            
            let model = builder
                .build()
                .await
                .map_err(|e: anyhow::Error| format!("Failed to build local model: {}", e))?;
            
            println!("Local model loaded successfully!");
            return Ok(model);
        }
    }
    
    Err(format!("Local model not found: {}", model_id))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, ai_chat, discover_models])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
