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

   ```

Now the AI chat feature will work in the app!

## Materials:

- [Mistral.rs Rust Examples](https://github.com/EricLBuehler/mistral.rs/tree/master/mistralrs/examples) - A little hard to find from the docs
- [mistralrs docs](https://ericlbuehler.github.io/mistral.rs/mistralrs/)

## Universal Quantized File Format (UQFF) and GGML Universal File (GGUF) Format

### What is UQFF?

Think of UQFF as a new way to package AI models so they run faster and use less computer memory. It's like having a ZIP file specifically designed for AI models. Specifically, it uses a technique called "quantization" to compress AI models to make them smaller and faster - kind of like how you might compress a video file to make it smaller.

#### What Makes UQFF Special

- **One File, Multiple Options** - Instead of having separate files for different compression levels, UQFF lets you pack multiple compression types into one file. It's like having a ZIP file that contains both the HD version and the compressed version of a movie.

- **No More Waiting** - Previously, if you wanted to use a compressed AI model, you had to wait for your computer to compress it first (which could take a while). With UQFF, someone already did the compression work for you - you just download and use it.

- **Works with Many Types** - It supports different compression methods (they have nerdy names like Q4_0, Q8_1, etc.) but basically just think of them as different quality/speed settings.

### What is GGUF?

GGUF stands for "GGML Universal File" (or sometimes "Generic GPT Unified Format") - it's a way to store AI models that makes them run faster and use less memory on regular computers like yours. It's essentially a special compression method that squishes models down so they can run on your laptop or desktop computer instead of needing a supercomputer.

#### What GGUF Does

- Compresses big AI models so they can run on CPUs or low-power devices
- Enables running complex models on everyday hardware like CPUs
- Optimized for quick loading and saving of models, making it highly efficient for inference purposes

#### Advantages

- One file format, one compression method
- Very popular and widely supported
- Works great, but limited to just GGUF-style compression

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
