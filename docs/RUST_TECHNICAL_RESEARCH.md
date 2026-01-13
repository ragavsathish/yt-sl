# Rust YouTube Slide Extractor - Technical Research Document

## Overview

This document provides comprehensive research on Rust libraries and utilities for building a CLI-based YouTube video slide extractor application. The tool will:

- Download YouTube videos
- Extract frames at intervals
- Detect unique slides using perceptual hashing
- Apply OCR to extract text from slides
- Export to PowerPoint presentation

All components will be implemented in Rust for performance, safety, and ease of distribution.

---

## Table of Contents

1. [Video Processing & FFmpeg](#video-processing--ffmpeg)
2. [OCR (Text Recognition)](#ocr-text-recognition)
3. [Audio Transcription](#audio-transcription)
4. [YouTube Video Download](#youtube-video-download)
5. [Image Processing](#image-processing)
6. [PowerPoint/Presentation Generation](#powerpointpresentation-generation)
7. [CLI Framework](#cli-framework)
8. [HTTP & Web](#http--web)
9. [Additional Utilities](#additional-utilities)
10. [Recommended Stack](#recommended-stack)
11. [Architecture Recommendations](#architecture-recommendations)
12. [Missing/Underdeveloped Areas](#missingunderdeveloped-areas)

---

## Video Processing & FFmpeg

### Core FFmpeg Libraries

#### video-rs (383 ⭐)
- **Repository**: [oddity-ai/video-rs](https://github.com/oddity-ai/video-rs)
- **Description**: Video readers, writers, muxers, encoders and decoders for Rust based on FFmpeg libraries
- **Features**:
  - Modern and active maintenance
  - High-level API
  - Cross-platform support
  - Decoding, encoding, muxing, demuxing
- **Use Case**: Primary choice for video frame extraction
- **Pros**: Most modern, active development, good documentation
- **Cons**: May require FFmpeg system dependency

#### rsmpeg (848 ⭐)
- **Repository**: [larksuite/rsmpeg](https://github.com/larksuite/rsmpeg)
- **Description**: A Rust crate that exposes FFmpeg's power as much as possible
- **Features**:
  - Low-level control over FFmpeg
  - Comprehensive FFmpeg bindings
  - High performance
- **Use Case**: Advanced video manipulation
- **Pros**: Maximum FFmpeg feature exposure
- **Cons**: Steeper learning curve, more verbose

#### rust-ffmpeg (1.8k ⭐)
- **Repository**: [zmwangx/rust-ffmpeg](https://github.com/zmwangx/rust-ffmpeg)
- **Description**: Safe FFmpeg wrapper
- **Features**:
  - Safe Rust bindings to FFmpeg
  - Battle-tested and stable
  - Comprehensive documentation
- **Use Case**: Stable, production-ready FFmpeg integration
- **Pros**: Mature, well-tested, stable API
- **Cons**: Less feature-rich than rsmpeg

#### ffmpeg-sidecar (508 ⭐)
- **Repository**: [nathanbabcock/ffmpeg-sidecar](https://github.com/nathanbabcock/ffmpeg-sidecar)
- **Description**: Wrap a standalone FFmpeg binary in an intuitive Iterator interface
- **Features**:
  - No FFmpeg linking required
  - Easy to use iterator interface
  - Cross-platform binary management
- **Use Case**: Easiest FFmpeg integration
- **Pros**: Simplest to use, no compilation issues
- **Cons**: Requires external FFmpeg binary

### Other Video Libraries

#### onevpl-rs (5 ⭐)
- **Repository**: [FallingSnow/onevpl-rs](https://github.com/FallingSnow/onevpl-rs)
- **Description**: Rust wrapper around the Intel OneAPI Video Processing Library
- **Use Case**: Hardware-accelerated video processing on Intel GPUs
- **Pros**: Hardware acceleration
- **Cons**: Intel-specific, limited hardware support

#### rust-ffmpeg-sys (240 ⭐)
- **Repository**: [CCExtractor/rusty_ffmpeg](https://github.com/CCExtractor/rusty_ffmpeg)
- **Description**: FFI bindings for FFmpeg inner libraries
- **Use Case**: Low-level FFmpeg access
- **Pros**: Raw FFmpeg bindings
- **Cons**: Unsafe, requires careful memory management

---

## OCR (Text Recognition)

### Top OCR Libraries

#### ocrs (1.7k ⭐) - RECOMMENDED
- **Repository**: [robertknight/ocrs](https://github.com/robertknight/ocrs)
- **Description**: Rust library and CLI tool for OCR (extracting text from images)
- **Features**:
  - Machine learning powered
  - Multiple backends
  - CLI tool included
  - Active maintenance
- **Use Case**: Primary OCR choice
- **Pros**: Best maintained, comprehensive, includes CLI
- **Cons**: May have higher memory usage

#### rust-paddle-ocr (140 ⭐)
- **Repository**: [zibo-chen/rust-paddle-ocr](https://github.com/zibo-chen/rust-paddle-ocr)
- **Description**: High-performance OCR library powered by PaddleOCR v4/v5 with MNN backend
- **Features**:
  - Supports 10+ languages (Chinese, English, Japanese, Korean, Arabic)
  - MNN backend for fast inference
  - PaddleOCR v4/v5 support
- **Use Case**: Multi-language OCR requirements
- **Pros**: Excellent multi-language support, fast
- **Cons**: Larger dependency tree

#### deepseek-ocr.rs (2.1k ⭐)
- **Repository**: [TimmyOVO/deepseek-ocr.rs](https://github.com/TimmyOVO/deepseek-ocr.rs)
- **Description**: Rust multi-backend OCR/VLM engine
- **Features**:
  - Multiple backends (DeepSeek-OCR, PaddleOCR-VL, DotsOCR)
  - DSQ quantization
  - OpenAI-compatible server & CLI
- **Use Case**: Advanced OCR with vision models
- **Pros**: State-of-the-art models, quantization
- **Cons**: Complex setup, resource-intensive

#### ddddocr (272 ⭐)
- **Repository**: [86maid/ddddocr](https://github.com/86maid/ddddocr)
- **Description**: ddddocr rust version, simple OCR API server
- **Features**:
  - Simple OCR API server
  - No OpenCV dependency
  - Cross-platform
  - MCP support
- **Use Case**: Lightweight OCR deployment
- **Pros**: No OpenCV, easy deployment, cross-platform
- **Cons**: Less accurate than ML-based solutions

### AI/Data Extraction

#### extractous (1.7k ⭐)
- **Repository**: [yobix-ai/extractous](https://github.com/yobix-ai/extractous)
- **Description**: Fast and efficient unstructured data extraction
- **Features**:
  - Written in Rust with bindings for many languages
  - NLP/machine learning focused
  - PDF processing support
- **Use Case**: Advanced text extraction and NLP
- **Pros**: Multi-language bindings, ML-focused
- **Cons**: Overkill for simple OCR

---

## Audio Transcription

### Top Transcription Libraries

#### whisper-rs (638 ⭐) - RECOMMENDED
- **Repository**: [tazz4843/whisper-rs](https://codeberg.org/tazz4843/whisper-rs)
- **Description**: Rust bindings for OpenAI Whisper automatic speech recognition
- **Features**:
  - Multiple model sizes (tiny, base, small, medium, large)
  - GPU acceleration support (CUDA, Metal, Vulkan, OpenCL)
  - Multi-language support
  - Timestamp extraction
  - Active maintenance
- **Use Case**: Primary audio transcription choice
- **Pros**: Best maintained, GPU support, comprehensive Whisper bindings
- **Cons**: Requires model download, larger memory usage

#### candle-whisper (16.3k ⭐)
- **Repository**: [huggingface/candle](https://github.com/huggingface/candle)
- **Description**: Minimalist ML framework for Rust with Whisper support
- **Features**:
  - Part of Hugging Face's Candle ecosystem
  - Metal/CUDA acceleration
  - Pure Rust implementation
  - Modern and active
- **Use Case**: Alternative to whisper-rs with Candle ecosystem
- **Pros**: Pure Rust, part of larger ML framework, modern
- **Cons**: Less mature than whisper-rs, smaller community

#### burn-whisper (5.7k ⭐)
- **Repository**: [tracel-ai/burn](https://github.com/tracel-ai/burn)
- **Description**: Deep learning framework in Rust with Whisper support
- **Features**:
  - Training and inference
  - Multiple backend support
  - Cross-platform
- **Use Case**: ML-focused transcription with training needs
- **Pros**: Training support, comprehensive ML framework
- **Cons**: Overkill for simple inference, heavier dependency

### Audio Extraction Libraries

#### ffmpeg-sidecar (508 ⭐)
- **Repository**: [nathanbabcock/ffmpeg-sidecar](https://github.com/nathanbabcock/ffmpeg-sidecar)
- **Description**: Wrap a standalone FFmpeg binary in an intuitive Iterator interface
- **Use Case**: Audio extraction from video
- **Pros**: Simplest FFmpeg integration, cross-platform
- **Cons**: Requires external FFmpeg binary

#### symphonia (947 ⭐)
- **Repository**: [pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia)
- **Description**: Pure Rust audio decoding and media demuxing library
- **Features**:
  - Multiple format support (MP3, WAV, AAC, FLAC, OGG)
  - Pure Rust implementation
  - No external dependencies
- **Use Case**: Audio-only processing without FFmpeg
- **Pros**: Pure Rust, no external dependencies
- **Cons**: Limited format support compared to FFmpeg

### Recommended Transcription Stack

Add `whisper-rs` with platform-specific GPU acceleration features, along with `ffmpeg-sidecar` for audio extraction or `symphonia` for pure Rust audio processing.

### Transcription Requirements

#### Whisper Audio Format
Whisper requires audio in specific format:
- **Format**: WAV (PCM)
- **Sample Rate**: 16,000 Hz
- **Channels**: 1 (Mono)
- **Bit Depth**: 16-bit

#### Model Selection
- **tiny.en** (~39MB): Fastest, ~74% accuracy
- **base.en** (~74MB): Fast, ~86% accuracy
- **small.en** (~464MB): Recommended, ~90% accuracy (RECOMMENDED)
- **medium.en** (~1.5GB): Slower, ~93% accuracy
- **large-v3** (~3.0GB): Slowest, ~94% accuracy

#### GPU Acceleration
- **macOS (Apple Silicon)**: Use `metal` feature flag
- **NVIDIA GPUs**: Use `cuda` feature flag
- **Others**: Use `vulkan` or `opencl` feature flags

---

## YouTube Video Download

### YouTube Downloaders

#### yaydl (309 ⭐) - RECOMMENDED
- **Repository**: [dertuxmalwieder/yaydl](https://github.com/dertuxmalwieder/yaydl)
- **Description**: Yet another youtube downloader
- **Features**:
  - Supports YouTube, Vimeo, and other platforms
  - Active maintenance
  - CLI tool
- **Use Case**: Primary video downloader
- **Pros**: Most active, multi-platform support
- **Cons**: May need subprocess invocation

#### rust-youtube-downloader (162 ⭐)
- **Repository**: [smoqadam/rust-youtube-downloader](https://github.com/smoqadam/rust-youtube-downloader)
- **Description**: Youtube video downloader written in Rust
- **Features**:
  - Pure Rust implementation
  - Simple and focused
- **Use Case**: Simple YouTube-only downloader
- **Pros**: Pure Rust, simple API
- **Cons**: Less feature-rich, less active

#### rafy-rs (49 ⭐)
- **Repository**: [ritiek/rafy-rs](https://github.com/ritiek/rafy-rs)
- **Description**: Rust library to download YouTube content and retrieve metadata
- **Features**:
  - Download YouTube content
  - Retrieve metadata (title, description, chapters)
  - Library API
- **Use Case**: Metadata extraction + download
- **Pros**: Library API, includes metadata
- **Cons**: Less mature

#### yt-downloader-rust (21 ⭐)
- **Repository**: [hasezoey/yt-downloader-rust](https://github.com/hasezoey/yt-downloader-rust)
- **Description**: A better youtube-dl CLI interface
- **Features**:
  - Enhanced youtube-dl interface
  - CLI tool
- **Use Case**: Alternative to yaydl
- **Pros**: Familiar youtube-dl interface
- **Cons**: Less active

---

## Image Processing

### Core Image Libraries

#### photon (3.3k ⭐) - RECOMMENDED
- **Repository**: [silvia-odwyer/photon](https://github.com/silvia-odwyer/photon)
- **Description**: Rust/WebAssembly image processing library
- **Features**:
  - Fast and efficient
  - WebAssembly support
  - Comprehensive filters and operations
  - Cross-platform
- **Use Case**: Primary image processing
- **Pros**: Most popular, fast, Wasm support
- **Cons**: May not have all advanced operations

#### imageproc (107 ⭐)
- **Repository**: [chyh1990/imageproc](https://github.com/chyh1990/imageproc)
- **Description**: An advanced image processing library for Rust
- **Features**:
  - Filters, transformations
  - Histograms
  - Morphological operations
  - Geometric transformations
- **Use Case**: Advanced image operations
- **Pros**: Comprehensive operations
- **Cons**: Less active development

#### raster (94 ⭐)
- **Repository**: [kosinix/raster](https://github.com/kosinix/raster)
- **Description**: An image processing library for Rust
- **Features**:
  - Traditional image operations
  - Filtering
  - Color manipulation
- **Use Case**: Basic image processing
- **Pros**: Simple API
- **Cons**: Less feature-rich

### CLI Image Tools

#### imagineer (216 ⭐)
- **Repository**: [foresterre/imagineer](https://github.com/foresterre/imagineer)
- **Description**: Accessible image processing and conversion from terminal
- **Features**:
  - Front-end for image-rs/image
  - CLI tool
  - Batch processing
- **Use Case**: CLI image operations
- **Pros**: Command-line ready
- **Cons**: Overkill for library use

#### imagekit (174 ⭐)
- **Repository**: [hzbd/imagekit](https://github.com/hzbd/imagekit)
- **Description**: ImageKit is a powerful and fast command-line tool for batch processing images
- **Features**:
  - Batch processing
  - Watermarking
  - Image resize
  - Multi-language
- **Use Case**: Batch image operations
- **Pros**: CLI-focused
- **Cons**: Not a library

---

## PowerPoint/Presentation Generation

### PowerPoint Libraries

#### openxml-office (48 ⭐) - RECOMMENDED
- **Repository**: [DraviaVemal/openxml-office](https://github.com/DraviaVemal/openxml-office)
- **Description**: Create or Modify PowerPoint/Presentation (pptx), Excel/Spreadsheet (xlsx) & Word/Document (docx) file with ease
- **Features**:
  - Supports pptx, xlsx, docx
  - Create and modify files
  - Comprehensive API
  - Active maintenance
- **Use Case**: Primary PowerPoint generation
- **Pros**: Most comprehensive, multi-format support
- **Cons**: May have learning curve

#### litchi (9 ⭐)
- **Repository**: [DevExzh/litchi](https://github.com/DevExzh/litchi)
- **Description**: A high-performance Rust library for parsing/creating Microsoft Office (OLE2 and OOXML), OpenDocument (ODF), and Apple iWork files
- **Features**:
  - High-performance
  - Multiple format support (OLE2, OOXML, ODF, iWork)
  - Parsing and creation
- **Use Case**: Performance-critical operations
- **Pros**: High performance, multiple formats
- **Cons**: Less documentation

#### PowerPointRS (0 ⭐)
- **Repository**: [Wonshtrum/PowerPointRS](https://github.com/Wonshtrum/PowerPointRS)
- **Description**: PowerPointGenerator rewrite in Rust
- **Features**:
  - PowerPoint generation
- **Use Case**: Alternative PPTX library
- **Pros**: Rust-native
- **Cons**: Very immature, unmaintained

### Alternative Approaches

#### SxPres (1 ⭐)
- **Repository**: [fatiservae/SxPres](https://github.com/fatiservae/SxPres)
- **Description**: A minimalistic software written in Rust to build rich HTML based slide presentations
- **Features**:
  - HTML-based slides
  - Minimalistic
- **Use Case**: Alternative to PPTX
- **Pros**: Web-friendly
- **Cons**: Not PowerPoint format

---

## CLI Framework

### CLI Libraries

#### seahorse (300 ⭐)
- **Repository**: [ksk001100/seahorse](https://github.com/ksk001100/seahorse)
- **Description**: A minimal CLI framework written in Rust
- **Features**:
  - Minimal design
  - Simple API
  - Subcommands support
- **Use Case**: Simple CLI applications
- **Pros**: Minimal, easy to learn
- **Cons**: Less feature-rich

#### argc (1.1k ⭐)
- **Repository**: [sigoden/argc](https://github.com/sigoden/argc)
- **Description**: A Bash CLI framework, also a Bash command runner
- **Features**:
  - Bash CLI framework
  - Command runner
- **Use Case**: Bash integration
- **Pros**: Bash-friendly
- **Cons**: Bash-specific

#### wena (257 ⭐)
- **Repository**: [wena-cli/wena](https://github.com/wena-cli/wena)
- **Description**: Wena is a micro-framework that provides an elegant starting point for your console application
- **Features**:
  - Micro-framework
  - Elegant design
  - Console application focus
- **Use Case**: Starting point for CLIs
- **Pros**: Elegant, focused
- **Cons**: Less mature

### TUI Frameworks

#### r3bl-open-core (434 ⭐)
- **Repository**: [r3bl-org/r3bl-open-core](https://github.com/r3bl-org/r3bl-open-core)
- **Description**: TUI framework and developer productivity apps in Rust
- **Features**:
  - Terminal UI framework
  - Syntax highlighting
  - Editor support
  - Cross-platform
- **Use Case**: Interactive CLI applications
- **Pros**: Feature-rich, active
- **Cons**: Overkill for simple CLI

---

## HTTP & Web

### HTTP Clients

#### reqwest (11.3k ⭐) - RECOMMENDED
- **Repository**: [seanmonstar/reqwest](https://github.com/seanmonstar/reqwest)
- **Description**: An easy and powerful Rust HTTP Client
- **Features**:
  - Easy to use
  - Async and sync support
  - JSON support
  - TLS support
  - Widely adopted
- **Use Case**: Primary HTTP client
- **Pros**: Standard choice, excellent documentation
- **Cons**: None significant

### Async Runtime

#### tokio (30.7k ⭐) - ESSENTIAL
- **Repository**: [tokio-rs/tokio](https://github.com/tokio-rs/tokio)
- **Description**: A runtime for writing reliable asynchronous applications with Rust
- **Features**:
  - I/O, networking, scheduling, timers
  - Async runtime
  - Comprehensive ecosystem
  - Industry standard
- **Use Case**: Every async Rust application
- **Pros**: Standard, well-documented, active
- **Cons**: None

### Specialized Clients

#### reqwest_eventsource (69 ⭐)
- **Repository**: [jpopesculian/reqwest-eventsource](https://github.com/jpopesculian/reqwest-eventsource)
- **Description**: Helper to build an Eventsource using reqwest
- **Use Case**: Server-Sent Events

#### reqwest_dav (40 ⭐)
- **Repository**: [niuhuan/reqwest_dav](https://github.com/niuhuan/reqwest_dav)
- **Description**: An async webdav client for rust with tokio and reqwest
- **Use Case**: WebDAV operations

#### kubernetes-rust (250 ⭐)
- **Repository**: [ynqa/kubernetes-rust](https://github.com/ynqa/kubernetes-rust)
- **Description**: Rust client for Kubernetes
- **Use Case**: Kubernetes integration

---

## Additional Utilities

### Hashing & Deduplication

#### image crate (official)
- **Description**: Core image processing in Rust
- **Features**:
  - Image I/O
  - Perceptual hashing support
  - Color manipulation
- **Use Case**: Perceptual hashing for slide deduplication
- **Pros**: Official, well-tested
- **Cons**: High-level API only

#### blake3
- **Description**: Fast cryptographic hash function
- **Features**:
  - Fast, secure
  - Incremental hashing
- **Use Case**: File deduplication, checksums
- **Pros**: Very fast
- **Cons**: Not perceptual hash

#### sha2
- **Description**: SHA-256/512 implementations
- **Use Case**: Cryptographic hashing
- **Pros**: Standard
- **Cons**: Slower than blake3

### Progress Bars

#### indicatif
- **Description**: Modern progress bars and spinners
- **Features**:
  - Multiple progress bars
  - Spinners
  - Estimation
  - Styling
- **Use Case**: Processing progress display
- **Pros**: Modern, feature-rich
- **Cons**: None significant

#### console
- **Description**: Terminal manipulation library
- **Features**:
  - Terminal size detection
  - Colors
  - Cursor control
- **Use Case**: Terminal UI operations
- **Pros**: Comprehensive
- **Cons**: Overkill for simple progress

### Error Handling

#### anyhow (RECOMMENDED)
- **Description**: Easy error handling in Rust
- **Features**:
  - Context propagation
  - Any error type
  - Downcasting
- **Use Case**: Application error handling
- **Pros**: Easy to use
- **Cons**: May lose type information

#### thiserror
- **Description**: Derive macros for error types
- **Features**:
  - Derive Error trait
  - Display formatting
  - From conversions
- **Use Case**: Library error handling
- **Pros**: Type-safe, idiomatic
- **Cons**: More verbose

#### color-eyre
- **Description**: Pretty error reports
- **Features**:
  - Colored error reports
  - Backtraces
  - Context
- **Use Case**: User-friendly error messages
- **Pros**: Beautiful output
- **Cons**: Performance overhead

### Configuration

#### clap (RECOMMENDED)
- **Description**: Command line argument parser (derive API)
- **Features**:
  - Derive API
  - Subcommands
  - Validation
  - Help generation
  - Shell completion
- **Use Case**: CLI argument parsing
- **Pros**: Derive API, powerful
- **Cons**: Learning curve

#### serde (ESSENTIAL)
- **Description**: Serialization/deserialization framework
- **Features**:
  - Generic serialization
  - Derive macros
  - Format-agnostic
- **Use Case**: Config file parsing, JSON
- **Pros**: Standard, ecosystem-wide
- **Cons**: None

#### serde_json / toml / yaml
- **Description**: Format-specific serde implementations
- **Use Case**: Config file formats
- **Pros**: Standard
- **Cons**: None

### Async Runtime

#### tokio - ESSENTIAL
- **Description**: Full-featured async runtime
- **Use Case**: Async operations
- **Pros**: Standard, feature-rich
- **Cons**: None

#### async-std
- **Description**: Alternative to Tokio
- **Use Case**: Tokio alternative
- **Pros**: Simpler
- **Cons**: Smaller ecosystem

#### smol
- **Description**: Small async runtime
- **Use Case**: Lightweight applications
- **Pros**: Small
- **Cons**: Less features

### File I/O

#### tokio-util
- **Description**: Async I/O utilities
- **Use Case**: Async file operations
- **Pros**: Tokio integration
- **Cons**: Tokio required

#### futures
- **Description**: Async utilities
- **Use Case**: Async combinators
- **Pros**: Standard
- **Cons**: Low-level

#### tracing
- **Description**: Structured logging
- **Use Case**: Application logging
- **Pros**: Structured, ecosystem
- **Cons**: Learning curve

---

## Recommended Stack

### Core Dependencies

Include the following crates in your project:

- **Video Processing**: `video-rs` or `ffmpeg-sidecar`
- **OCR**: `ocrs` (primary) or `rust-paddle-ocr` (multi-language)
- **Audio Transcription**: `whisper-rs` with GPU features, optionally `symphonia`
- **YouTube Download**: `yaydl` or `rafy-rs`
- **Image Processing**: `photon`, `imageproc`, `image`
- **PowerPoint**: `openxml-office`
- **CLI**: `clap` with derive features, `indicatif`
- **HTTP**: `reqwest` with json features
- **Async Runtime**: `tokio` with full features
- **Error Handling**: `anyhow`, `thiserror`
- **Hashing**: `blake3`
- **Logging**: `tracing`, `tracing-subscriber`
- **Serialization**: `serde` with derive features, `serde_json`

---

## Architecture Recommendations

### 1. CLI Layer
**Libraries**: `clap`, `indicatif`

**Responsibilities**:
- Command-line argument parsing
- Help generation
- Shell completion
- Progress display
- User interaction

Use clap's derive API for parsing commands with subcommands, arguments, and validation. Combine with `indicatif` for progress tracking during long-running operations.

### 2. Async Runtime Layer
**Libraries**: `tokio`, `futures`

**Responsibilities**:
- Async task spawning
- Concurrent processing
- I/O operations
- Timeout management

Initialize tokio runtime with full features, parse CLI arguments, and dispatch to appropriate async handlers for each subcommand.

### 3. Video Download Layer
**Libraries**: `yaydl` (subprocess) or `rafy-rs`

**Responsibilities**:
- YouTube video download
- Metadata extraction
- Chapter/timestamp retrieval
- Progress reporting

Download video using yaydl as subprocess or rafy-rs library. Track progress with indicatif progress bar and extract metadata including title, description, and chapter information.

### 4. Video Processing Layer
**Libraries**: `video-rs` or `ffmpeg-sidecar`

**Responsibilities**:
- Frame extraction
- Video decoding
- Timestamp tracking
- Format handling

Create decoder from video file path, iterate through frames, and collect frames at specified intervals. Handle frame decoding errors and track timestamps for each extracted frame.

### 5. Image Processing Layer
**Libraries**: `photon`, `image`

**Responsibilities**:
- Image format conversion
- Perceptual hashing
- Image manipulation
- Quality analysis

Convert image to grayscale, resize to small dimensions, compute average perceptual hash. Use Hamming distance to compare hashes against previously seen frames and determine uniqueness based on configurable threshold.

### 6. OCR Layer
**Libraries**: `ocrs` or `rust-paddle-ocr`

**Responsibilities**:
- Text extraction from images
- Multi-language support
- Confidence scoring
- Post-processing

Initialize OCR engine, pass image to recognizer, and return results with text, bounding boxes, and confidence scores. Handle multi-language requests and post-process results.

### 7. Audio Transcription Layer
**Libraries**: `whisper-rs`, `ffmpeg-sidecar` or `symphonia`

**Responsibilities**:
- Audio extraction from video
- Audio format conversion (to 16kHz mono WAV)
- Speech-to-text transcription
- Timestamp extraction
- Model management

Extract audio track from video using FFmpeg, convert to Whisper-compatible format (16kHz mono PCM WAV), load appropriate Whisper model (small.en recommended), configure parameters including language and sampling strategy, perform transcription with GPU acceleration if available, and extract text with timestamps.

### 8. Deduplication Layer
**Libraries**: `image`, `blake3`

**Responsibilities**:
- Perceptual hashing
- Duplicate detection
- Similarity measurement
- Hash storage

Maintain a set of previously seen perceptual hashes. For each new frame, compute Hamming distance against all seen hashes. If all distances exceed threshold, mark as unique and add to set. Track timestamp and associated text for each unique slide.

### 9. Export Layer
**Libraries**: `openxml-office`

**Responsibilities**:
- PowerPoint creation
- Slide layout
- Text formatting
- Image insertion
- File output

Create new presentation, iterate through unique slides, add each as new slide with image and text content, apply layout and formatting, save to output file path. Handle image scaling and positioning within slide bounds.

---

## Missing/Underdeveloped Areas

### 1. Scene Detection
**Status**: No mature Rust scene cut detection library

**Challenges**:
- May need to implement using FFmpeg scene detection filter
- Limited algorithm implementations in Rust
- Python/OpenCV ecosystem more mature

**Potential Solutions**:
- Use FFmpeg scene detection filter via `video-rs`
- Implement simple algorithms (histogram difference, edge detection)
- Consider C++/OpenCV bindings with `opencv-rs`

**Algorithms to Implement**:
- Histogram-based detection
- Edge-based detection
- Motion detection
- Fadecut detection

### 2. Advanced OCR Models
**Status**: Limited compared to Python ecosystem

**Challenges**:
- Fewer OCR model options
- Less flexibility with custom training
- Performance may lag behind optimized Python implementations

**Available Options**:
- `ocrs` - Machine learning powered, good but limited backends
- `rust-paddle-ocr` - PaddleOCR, good multi-language
- `deepseek-ocr.rs` - State-of-the-art but complex

**Missing**:
- EasyOCR
- Tesseract with modern bindings
- Custom model training pipelines
- Lightweight models

### 3. Computer Vision
**Status**: Limited high-level CV libraries

**Challenges**:
- `opencv-rs` exists but less mature than Python OpenCV
- Fewer high-level algorithms
- Less documentation

**Available Options**:
- `opencv-rs` - OpenCV bindings, works but less polished
- `imageproc` - Basic image operations
- Custom implementations

**Missing**:
- Object detection libraries
- Face detection/recognition
- Motion analysis
- Feature matching

### 4. AI/ML
**Status**: Limited ML inference options in Rust

**Challenges**:
- Smaller ecosystem compared to Python
- Fewer pre-trained models
- Less tooling for model training

**Available Options**:
- `candle` - Modern ML framework by Hugging Face
- `tch-rs` - PyTorch bindings
- `burn` - Deep learning framework
- `tract` - Neural network inference

**Missing**:
- More model zoos
- Better training tools
- Easier model conversion
- More examples and tutorials

### 5. PowerPoint Templates
**Status**: Limited template support in `openxml-office`

**Challenges**:
- Need to build templates manually
- No existing template libraries
- Limited styling options

**Potential Solutions**:
- Build template system in-house
- Use HTML to PPTX converters
- Contribute templates to `openxml-office`

---

## Performance Considerations

### 1. Parallel Processing
**Strategy**: Use Tokio for async, Rayon for CPU-bound tasks

**Benefits**:
- Utilize all CPU cores
- Non-blocking I/O operations
- Better responsiveness

Use Rayon's parallel iterators to process frames concurrently across available CPU cores while Tokio handles async I/O operations.

### 2. Memory Management
**Strategy**: Process frames in batches

**Benefits**:
- Avoid loading entire video into memory
- Reduce peak memory usage
- Enable processing of large videos

Process frames in fixed-size batches, releasing memory after each batch completes. Use streaming approach for video decoding instead of loading entire file.

### 3. Caching
**Strategy**: Cache intermediate results

**Benefits**:
- Avoid reprocessing
- Faster re-runs
- Resume interrupted jobs

**Implementation**:
Use embedded database like sled or RocksDB to cache intermediate results with keys based on input parameters. Check cache before processing and store results after successful computation.

### 4. GPU Acceleration for Transcription
**Strategy**: Leverage hardware acceleration for Whisper

**Benefits**:
- 10-50x faster transcription on GPU
- Lower CPU usage
- Better energy efficiency

**Platform-specific Flags**:
- macOS (Apple Silicon): Use `metal` feature
- NVIDIA GPUs: Use `cuda` feature
- Other GPUs: Use `vulkan` feature

---

## Testing Strategy

### 1. Unit Tests
**Focus**: Individual components

**Examples**:
- Hash computation correctness
- OCR text extraction accuracy
- Perceptual hash thresholds
- Image operations

### 2. Integration Tests
**Focus**: Component interaction

**Examples**:
- Full workflow: download → extract → dedup → OCR → export
- Error handling paths
- Progress reporting
- File cleanup

### 3. Performance Tests
**Focus**: Benchmarking

**Examples**:
- Frame extraction speed
- OCR throughput
- Hash computation performance
- Memory usage

### 4. Golden Master Tests
**Focus**: Output consistency

**Examples**:
- PPTX output matches reference
- Text extraction matches expected
- Image quality within tolerance

---

## Deployment Strategy

### 1. Single Binary Distribution
**Strategy**: Use `cargo build --release`

**Benefits**:
- No runtime dependencies
- Easy distribution
- Cross-platform compilation

**Platforms**:
- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64)
- Windows (x86_64)

### 2. Docker Image
**Strategy**: Multi-stage build for minimal size

Use Rust image for building, then copy binary to slim base image with only necessary runtime dependencies (FFmpeg, OCR, Whisper models).

### 3. Package Managers
**Targets**:
- Homebrew (macOS)
- AUR (Arch Linux)
- Scoop (Windows)
- Cargo (cross-platform)

---

## Documentation Requirements

### 1. User Documentation
- Installation guide
- Quick start
- Command reference
- Examples
- FAQ

### 2. Developer Documentation
- Architecture overview
- Contributing guide
- API documentation
- Testing guide
- Performance tuning

### 3. Examples
- Basic usage
- Advanced options
- Batch processing
- Custom configuration

---

## License Considerations

### 1. Library Licenses
**Compatible Licenses**:
- MIT/Apache-2.0 (most Rust libraries)
- Permissive licenses
- No copyleft

### 2. Project License
**Recommended**: MIT or Apache-2.0
**Reason**:
- Permissive
- Compatible with most dependencies
- Business-friendly

### 3. Dependency Review
**Tools**:
- `cargo-deny`
- `cargo-license`
- Regular audits

---

## Security Considerations

### 1. Input Validation
**Sanitize**:
- YouTube URLs
- File paths
- User-provided parameters

### 2. Resource Limits
**Limit**:
- Maximum video size
- Maximum frame count
- Processing timeout
- Memory usage

### 3. Dependency Updates
**Strategy**:
- Regular updates
- Security advisories
- `cargo-audit`

---

## Future Enhancements

### 1. Advanced Features
- Scene-aware extraction
- Smart transition detection
- Multi-language audio models
- Multi-language OCR

### 2. Export Formats
- Google Slides
- PDF
- Markdown
- JSON API

### 3. Performance
- GPU acceleration
- Distributed processing
- Incremental updates
- Compression optimization

### 4. User Experience
- Interactive CLI (TUI)
- Web interface
- Desktop GUI
- Browser extension

---

## Conclusion

The Rust ecosystem provides comprehensive tools for building a YouTube video slide extractor. While some areas (scene detection, advanced OCR) are less mature compared to Python, the available libraries are production-ready and offer significant benefits in performance, safety, and ease of distribution.

### Key Recommendations:
1. Use `video-rs` or `ffmpeg-sidecar` for video processing
2. Choose `ocrs` for simplicity or `rust-paddle-ocr` for multi-language
3. Use `whisper-rs` with Metal/CUDA acceleration for audio transcription
4. Use `yaydl` as subprocess for YouTube download
5. Leverage `photon` and `image` for image processing and hashing
6. Use `openxml-office` for PowerPoint generation
7. Build with `tokio` and `clap` for async runtime and CLI
8. Implement parallel processing with `rayon` for performance

The ecosystem is mature enough to build a robust, high-performance CLI application that can compete with Python-based solutions while offering superior performance and distribution characteristics.
