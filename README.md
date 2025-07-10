# 🦙🦀 Tauri-Served Local LLMs with Mistral.rs

This repo was presented at the [Desert Rust](https://azdevs.github.io/desert-rustaceans) meetup on Rust × AI.

## Quick Start

1. **Install dependencies**

   ```bash
   pnpm install
   ```

2. **Run the app**
   ```bash
   pnpm tauri dev
   ```

## Download Model

> [!WARNING]  
> LLM Models are large! Check your disk space before downloading models arbitrarily! The entire model set on my machine is about **~40GB** (also, I am being lazy and letting Rust copy the entire models in duplicate into the `target` directory so it's actually **80GB**). Best to download one model at a time and evaluate.

In order to chat with AI models locally, you need to download them first:

1. **Get a Hugging Face token**: Create a [Hugging Face account](https://huggingface.co/) and [get an API key](https://huggingface.co/settings/tokens)

2. **Set up environment**: Create a `.env` file in the project root:

   ```bash
   HF_TOKEN=your_hugging_face_token_here
   ```

3. **Download the model**:

   ```bash
   cd src-tauri
   cargo run --example download-models list
   cargo run --example download_models download llama-vision --force --yes
   ```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

# A Newbie's Guide to Mistral.rs

## Hidden Gems in the Documentation

The [mistral.rs Rust Docs](https://ericlbuehler.github.io/mistral.rs/mistralrs/) can be a little hard to navigate and there are a lot of concepts. Here are a few things I think would have helped if they were more prominent:

### ❗ Chat Templates (IMPORTANT)

You will need to specify [a chat template](https://github.com/EricLBuehler/mistral.rs/tree/master/chat_templates) ([e.g. `mistral.json`](https://github.com/danielbank/tauri-mistral-chat/blob/main/src-tauri/examples/hello_world.rs#L187-L192)) with your model builder:

```rs
   builder = builder.with_chat_template(template_path);
```

These templates are readily available in the mistral.rs repo, but you have to look for them: https://github.com/EricLBuehler/mistral.rs/tree/master/chat_templates

As an aside, you can use remote tokenizer as backup: `.with_tok_model_id("mistralai/Mistral-7B-Instruct-v0.1")`

### Rust Examples!

The [Rust examples](https://github.com/EricLBuehler/mistral.rs/tree/master/mistralrs/examples) are all here and are a good starting point for simple programs that demonstrate the models

## ❗ Hugging Face Model Directories (IMPORTANT)

This is also important and I can't explain it very well because I still don't know the idiomatic patterns myself. However as much as I can say it, it's that it matters very much that you have all the files necessary for running the model and structured in your file system in an expected way (for instance, the name of the model folder has to match with the URL you downloaded it from on Hugging Face).

**UQFF Vision Models Require Multiple Files (Not Just .uqff!):**

- **tokenizer.json** (17MB) - **CRITICAL** - Translates text into data.
- **config.json** - Model configuration
- **tokenizer_config.json** - Tokenizer settings
- **preprocessor_config.json** - Image preprocessing config
- **generation_config.json** - Generation parameters
- **residual.safetensors** (5.8GB) - **REQUIRED** - Additional model weights
- **Multiple .uqff files** - Choose quantization level (Q4K/Q5K/Q8_0)

**Key Insight:** UQFF models aren't just single files - they're ecosystems of configuration and weight files!

## ❗ Use the Right Buider Type (IMPORTANT)

It's important to use the right model builder type for the model you are attempting to load.

**Different Builders for Different Model Types:**

```rust
// GGUF Models (simpler, self-contained)
GgufModelBuilder::new(path, vec!["model.gguf"])

// UQFF Vision Models (complex, multi-file)
UqffVisionModelBuilder::new(path, uqff_files)
    .into_inner()
    .with_isq(IsqType::Q5_0)  // Better than Q4K

// UQFF Text Models
UqffTextModelBuilder::new(path, uqff_files)

// MatFormer Vision Models
VisionModelBuilder::new(path).with_isq(IsqType::Q4K)

// Remote Models (when local fails)
TextModelBuilder::new("HuggingFaceTB/SmolLM3-3B")
```

## 🎚️ Quantization Quality Ladder

**Quality vs Size Trade-offs:**

- **Q4K** - Good balance (smaller)
- **Q5_0** - Better quality (recommended from example code)
- **Q8_0** - Highest quality (largest)

**Lesson:** Q5_0 often provides the best balance for UQFF models.

## 🚀 Local vs Remote Model Loading

**Sometimes When Local Fails, Remote Still Works:**

- ✅ SmolLM3: Use remote `TextModelBuilder` even with local UQFF files
- ✅ Remote models handle tokenizer/config automatically
- ❌ Local UQFF requires manual file management

# Universal Quantized File Format (UQFF) and GGML Universal File (GGUF) Format

## What is UQFF?

Think of UQFF as a new way to package AI models so they run faster and use less computer memory. It's like having a ZIP file specifically designed for AI models. Specifically, it uses a technique called "quantization" to compress AI models to make them smaller and faster - kind of like how you might compress a video file to make it smaller.

### What Makes UQFF Special

- **One File, Multiple Options** - Instead of having separate files for different compression levels, UQFF lets you pack multiple compression types into one file. It's like having a ZIP file that contains both the HD version and the compressed version of a movie.

- **No More Waiting** - Previously, if you wanted to use a compressed AI model, you had to wait for your computer to compress it first (which could take a while). With UQFF, someone already did the compression work for you - you just download and use it.

- **Works with Many Types** - It supports different compression methods (they have nerdy names like Q4_0, Q8_1, etc.) but basically just think of them as different quality/speed settings.

## What is GGUF?

GGUF stands for "GGML Universal File" (or sometimes "Generic GPT Unified Format") - it's a way to store AI models that makes them run faster and use less memory on regular computers like yours. It's essentially a special compression method that squishes models down so they can run on your laptop or desktop computer instead of needing a supercomputer.

### What GGUF Does

- Compresses big AI models so they can run on CPUs or low-power devices
- Enables running complex models on everyday hardware like CPUs
- Optimized for quick loading and saving of models, making it highly efficient for inference purposes

### Advantages

- One file format, one compression method
- Very popular and widely supported
- Works great, but limited to just GGUF-style compression

# Model Chart

I found the following chart interesting as it shows the size vs win % trade-off of some current LLMs as of 2025.

![A Chart of Small LLMs rated by their win% vs size](https://github.com/danielbank/tauri-mistral-chat/blob/main/model_chart.png)
