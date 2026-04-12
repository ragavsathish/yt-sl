use base64::{engine::general_purpose, Engine as _};
use clap::Parser;
use image::imageops;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use tokio::sync::Semaphore;

const HASH_SIZE: u32 = 8;
const MAX_IMAGE_DIM: u32 = 1024;
const TRAINING_PROMPT: &str =
    "Is this image a presentation slide? Answer with exactly one word: SLIDE or NOT_SLIDE.";

type R<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

// ── CLI ─────────────────────────────────────────────────────────────────────

/// Pure slide extractor. Does NOT call yt-dlp or ffmpeg.
/// Expects a directory of frame images and an optional transcript JSON.
///
/// Usage:
///   # You handle download + extraction yourself:
///   yt-dlp -f best -o video.mp4 URL
///   ffmpeg -i video.mp4 -vf fps=1/5 -q:v 2 frames/frame_%04d.jpg
///   ffmpeg -i video.mp4 -vn -acodec pcm_s16le -ar 16000 -ac 1 audio.wav
///   curl localhost:1234/v1/audio/transcriptions -F file=@audio.wav -F model=whisper-1 -F response_format=verbose_json > transcript.json
///
///   # Then run this:
///   yt-sl --frames frames/ --output slides/ --transcript transcript.json
#[derive(Parser)]
#[command(name = "yt-sl", about = "Dedup frames + vision OCR → markdown report")]
struct Args {
    /// Directory containing extracted frame images (jpg/png)
    #[arg(short, long)]
    frames: String,

    /// Output directory for slides + report
    #[arg(short, long, default_value = "./output")]
    output: String,

    /// Optional transcript JSON file (Whisper verbose_json format)
    #[arg(long)]
    transcript: Option<String>,

    /// Title for the report
    #[arg(long, default_value = "Untitled")]
    title: String,

    /// Source URL (for report metadata only)
    #[arg(long)]
    url: Option<String>,

    /// Frame extraction interval used (for timestamp calculation)
    #[arg(short, long, default_value = "5")]
    interval: u64,

    /// Hash similarity threshold for dedup (0.0-1.0)
    #[arg(short = 'T', long, default_value = "0.90")]
    threshold: f64,

    /// Vision model name
    #[arg(long, default_value = "qwen/qwen3-vl-8b")]
    model: String,

    /// Vision API base URL (OpenAI-compatible)
    #[arg(long, default_value = "http://localhost:1234/v1")]
    vision_api: String,

    /// Max concurrent vision API requests
    #[arg(long, default_value = "4")]
    concurrency: usize,
}

// ── Data types ──────────────────────────────────────────────────────────────

struct SlideData {
    index: usize,
    timestamp: f64,
    image_path: PathBuf,
    text: String,
    transcript: String,
}

#[derive(Deserialize)]
struct Segment {
    start: f64,
    #[allow(dead_code)]
    end: f64,
    text: String,
}

#[derive(Deserialize)]
struct TranscriptFile {
    text: String,
    segments: Option<Vec<Segment>>,
}

// ── OpenAI-compatible API types ─────────────────────────────────────────────

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: Vec<ContentPart>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrlData },
}

#[derive(Serialize)]
struct ImageUrlData {
    url: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMsg,
}

#[derive(Deserialize)]
struct ChatMsg {
    content: String,
}

// ── Main pipeline ───────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> R<()> {
    let args = Args::parse();

    let slides_dir = Path::new(&args.output).join("slides");
    std::fs::create_dir_all(&slides_dir)?;

    // 1. Load transcript if provided
    let (full_transcript, segments) = if let Some(ref path) = args.transcript {
        let data = std::fs::read_to_string(path)?;
        let tf: TranscriptFile = serde_json::from_str(&data)?;
        let segs = tf.segments.unwrap_or_default();
        eprintln!("[1/4] Transcript loaded ({} segments)", segs.len());
        (tf.text, segs)
    } else {
        eprintln!("[1/4] No transcript provided, skipping");
        (String::new(), vec![])
    };

    // 2. Read + dedup frames
    let mut frame_paths: Vec<PathBuf> = std::fs::read_dir(&args.frames)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext == "jpg" || ext == "jpeg" || ext == "png")
        })
        .collect();
    frame_paths.sort();

    if frame_paths.is_empty() {
        return Err(format!("No image files found in {}", args.frames).into());
    }

    // Load real timestamps if available (from ffmpeg scene detection)
    let timestamps_file = Path::new(&args.frames).join("timestamps.txt");
    let real_timestamps: Vec<f64> = if timestamps_file.exists() {
        std::fs::read_to_string(&timestamps_file)?
            .lines()
            .filter_map(|l| l.trim().parse::<f64>().ok())
            .collect()
    } else {
        vec![]
    };

    // Detect slide region — try several frames to find one with a visible stage layout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    // Sample several frames and pick the tightest slide region (smallest area = best crop)
    let mut slide_region = None;
    let mut best_area = f64::MAX;
    let sample_count = 8;
    let step = frame_paths.len().max(1) / sample_count.max(1);
    for i in 0..sample_count {
        let idx = (i * step + step / 2).min(frame_paths.len() - 1);
        if let Ok(Some(region)) =
            detect_slide_region(&client, &frame_paths[idx], &args.model, &args.vision_api).await
        {
            let area = region.w_pct * region.h_pct;
            if area < best_area && (region.w_pct < 85.0 || region.h_pct < 85.0) {
                best_area = area;
                slide_region = Some(region);
            }
        }
    }
    match &slide_region {
        Some(r) => eprintln!(
            "  slide region: {}%,{}% {}%x{}%",
            r.x_pct as u32, r.y_pct as u32, r.w_pct as u32, r.h_pct as u32
        ),
        None => eprintln!("  slide region: full frame (could not detect)"),
    }

    let unique_frames = dedup_frames(&frame_paths, args.threshold, slide_region);
    eprintln!(
        "[2/4] Dedup: {} frames -> {} unique",
        frame_paths.len(),
        unique_frames.len()
    );

    // 3. Vision OCR + classification
    let sem = std::sync::Arc::new(Semaphore::new(args.concurrency));
    let mut handles = Vec::new();

    for (i, frame_path) in unique_frames.iter().enumerate() {
        let client = client.clone();
        let sem = sem.clone();
        let model = args.model.clone();
        let api = args.vision_api.clone();
        let path = frame_path.clone();
        let dest = slides_dir.join(format!("slide_{:04}.jpg", i + 1));

        // Use real timestamp if available, otherwise estimate from index
        let frame_idx = frame_paths
            .iter()
            .position(|p| p == frame_path)
            .unwrap_or(i);
        let timestamp = real_timestamps
            .get(frame_idx)
            .copied()
            .unwrap_or(i as f64 * args.interval as f64);

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            eprintln!("  ocr: {}", path.file_name().unwrap().to_str().unwrap());
            let result = vision_ocr(&client, &path, &model, &api).await;
            match result {
                Ok(Some(text)) => {
                    let _ = std::fs::copy(&path, &dest);
                    // label: SLIDE (with OCR text)
                    Some((i, timestamp, dest, text, path, "SLIDE".to_string()))
                }
                Ok(None) => {
                    eprintln!("    -> NOT_SLIDE");
                    // label: NOT_SLIDE (still record for training)
                    Some((
                        i,
                        timestamp,
                        PathBuf::new(),
                        String::new(),
                        path,
                        "NOT_SLIDE".to_string(),
                    ))
                }
                Err(e) => {
                    eprintln!("    -> error: {}", e);
                    None
                }
            }
        }));
    }

    let mut slides: Vec<SlideData> = Vec::new();
    let mut training_labels: Vec<(PathBuf, String)> = Vec::new();
    for handle in handles {
        if let Some((idx, ts, dest, text, src, label)) = handle.await? {
            training_labels.push((src, label.clone()));
            if label == "SLIDE" {
                slides.push(SlideData {
                    index: idx + 1,
                    timestamp: ts,
                    image_path: dest,
                    text,
                    transcript: String::new(),
                });
            }
        }
    }
    slides.sort_by(|a, b| a.index.cmp(&b.index));

    // Text-based dedup: remove slides with duplicate/near-duplicate OCR text
    let before_text_dedup = slides.len();
    slides = dedup_by_text(slides);
    eprintln!(
        "[3/4] OCR done: {} slides ({} removed as text duplicates)",
        slides.len(),
        before_text_dedup - slides.len()
    );

    // Save training data in background
    save_training_data(&training_labels);

    // Assign transcript segments to slides
    assign_segments(&mut slides, &segments, args.interval as f64);

    // 4. Generate markdown
    let report_path = Path::new(&args.output).join("report.md");
    generate_markdown(
        &args.title,
        args.url.as_deref().unwrap_or(""),
        &full_transcript,
        &slides,
        &report_path,
    )?;
    eprintln!("[4/4] Report: {}", report_path.display());

    Ok(())
}

// ── Slide region detection ──────────────────────────────────────────────────

const REGION_PROMPT: &str = "\
This frame is from a video recording of a presentation. A slide is projected on a screen somewhere in the frame.

Return the bounding box of ONLY the projected slide/screen area as four integers: x y w h
where x,y is the top-left corner and w,h is width and height, all as percentages (0-100) of the frame dimensions.

Respond with ONLY four numbers separated by spaces, nothing else. Example: 30 5 45 55";

#[derive(Clone, Copy)]
struct CropRegion {
    x_pct: f64,
    y_pct: f64,
    w_pct: f64,
    h_pct: f64,
}

async fn detect_slide_region(
    client: &reqwest::Client,
    path: &Path,
    model: &str,
    api_base: &str,
) -> R<Option<CropRegion>> {
    let image_data = tokio::task::spawn_blocking({
        let path = path.to_path_buf();
        move || resize_image(&path)
    })
    .await??;
    let b64 = general_purpose::STANDARD.encode(&image_data);
    let data_url = format!("data:image/jpeg;base64,{}", b64);

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: vec![
                ContentPart::Text {
                    text: REGION_PROMPT.to_string(),
                },
                ContentPart::ImageUrl {
                    image_url: ImageUrlData { url: data_url },
                },
            ],
        }],
    };

    let resp: ChatResponse = client
        .post(format!("{}/chat/completions", api_base))
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let content = resp
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();

    // Parse "x y w h" from response
    let nums: Vec<f64> = content
        .split_whitespace()
        .filter_map(|s| {
            s.trim_matches(|c: char| !c.is_ascii_digit() && c != '.')
                .parse()
                .ok()
        })
        .collect();

    if nums.len() >= 4 && nums[2] > 10.0 && nums[3] > 10.0 {
        Ok(Some(CropRegion {
            x_pct: nums[0].clamp(0.0, 100.0),
            y_pct: nums[1].clamp(0.0, 100.0),
            w_pct: nums[2].clamp(10.0, 100.0),
            h_pct: nums[3].clamp(10.0, 100.0),
        }))
    } else {
        Ok(None)
    }
}

fn crop_to_region(path: &Path, region: CropRegion) -> R<image::DynamicImage> {
    let img = image::open(path)?;
    let (w, h) = (img.width() as f64, img.height() as f64);
    let x = (region.x_pct / 100.0 * w) as u32;
    let y = (region.y_pct / 100.0 * h) as u32;
    let cw = (region.w_pct / 100.0 * w) as u32;
    let ch = (region.h_pct / 100.0 * h) as u32;
    let cw = cw.min(img.width() - x);
    let ch = ch.min(img.height() - y);
    Ok(img.crop_imm(x, y, cw, ch))
}

fn avg_hash_cropped(path: &Path, region: Option<CropRegion>) -> R<u64> {
    let gray = match region {
        Some(r) => crop_to_region(path, r)?.to_luma8(),
        None => image::open(path)?.to_luma8(),
    };
    let small = imageops::resize(&gray, HASH_SIZE, HASH_SIZE, imageops::FilterType::Lanczos3);
    let mean = small.pixels().map(|p| p[0] as u64).sum::<u64>() / (HASH_SIZE * HASH_SIZE) as u64;
    let mut hash: u64 = 0;
    for (i, pixel) in small.pixels().enumerate() {
        if pixel[0] as u64 >= mean {
            hash |= 1 << i;
        }
    }
    Ok(hash)
}

// ── Perceptual hash dedup ───────────────────────────────────────────────────

fn avg_hash(path: &Path) -> R<u64> {
    let img = image::open(path)?.to_luma8();
    let small = imageops::resize(&img, HASH_SIZE, HASH_SIZE, imageops::FilterType::Lanczos3);
    let mean = small.pixels().map(|p| p[0] as u64).sum::<u64>() / (HASH_SIZE * HASH_SIZE) as u64;
    let mut hash: u64 = 0;
    for (i, pixel) in small.pixels().enumerate() {
        if pixel[0] as u64 >= mean {
            hash |= 1 << i;
        }
    }
    Ok(hash)
}

fn hamming_similarity(a: u64, b: u64) -> f64 {
    1.0 - (a ^ b).count_ones() as f64 / 64.0
}

fn dedup_frames(paths: &[PathBuf], threshold: f64, region: Option<CropRegion>) -> Vec<PathBuf> {
    if paths.is_empty() {
        return vec![];
    }

    let hash_fn = |p: &Path| -> R<u64> {
        if region.is_some() {
            avg_hash_cropped(p, region)
        } else {
            avg_hash(p)
        }
    };

    let mut unique = vec![paths[0].clone()];
    let mut accepted_hashes = vec![hash_fn(&paths[0]).unwrap_or(0)];

    for path in &paths[1..] {
        if let Ok(hash) = hash_fn(path) {
            let is_duplicate = accepted_hashes
                .iter()
                .any(|&h| hamming_similarity(h, hash) >= threshold);
            if !is_duplicate {
                unique.push(path.clone());
                accepted_hashes.push(hash);
            }
        }
    }
    unique
}

// ── Vision LLM (Qwen-VL) ───────────────────────────────────────────────────

const VISION_PROMPT: &str = "\
You are analyzing a frame extracted from a video recording of a presentation.

If a presentation slide is visible in this image — even partially alongside a speaker — \
extract ALL readable text from the slide, preserving structure with line breaks. \
Respond with the extracted text only, no preamble.

Only respond NOT_SLIDE if there is truly NO readable slide text visible (e.g., only a \
speaker with no text behind them, only the audience, or a blank screen). A few blurry \
or partially visible words do NOT count as readable text.";

fn resize_image(path: &Path) -> R<Vec<u8>> {
    let img = image::open(path)?;
    let (w, h) = (img.width(), img.height());
    let img = if w > MAX_IMAGE_DIM || h > MAX_IMAGE_DIM {
        img.resize(MAX_IMAGE_DIM, MAX_IMAGE_DIM, imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Jpeg)?;
    Ok(buf.into_inner())
}

async fn vision_ocr(
    client: &reqwest::Client,
    path: &Path,
    model: &str,
    api_base: &str,
) -> R<Option<String>> {
    let image_data = tokio::task::spawn_blocking({
        let path = path.to_path_buf();
        move || resize_image(&path)
    })
    .await??;
    let b64 = general_purpose::STANDARD.encode(&image_data);
    let data_url = format!("data:image/jpeg;base64,{}", b64);

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: vec![
                ContentPart::Text {
                    text: VISION_PROMPT.to_string(),
                },
                ContentPart::ImageUrl {
                    image_url: ImageUrlData { url: data_url },
                },
            ],
        }],
    };

    let resp: ChatResponse = client
        .post(format!("{}/chat/completions", api_base))
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let content = resp
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();

    if content.to_uppercase().contains("NOT_SLIDE") {
        Ok(None)
    } else {
        Ok(Some(content))
    }
}

// ── Text-based dedup ────────────────────────────────────────────────────

fn normalize_text(text: &str) -> String {
    text.split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

fn text_similarity(a: &str, b: &str) -> f64 {
    let a = normalize_text(a);
    let b = normalize_text(b);
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let (shorter, longer) = if a.len() <= b.len() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    // Check if shorter is a substring of longer (partial slide capture)
    if longer.contains(shorter.as_str()) {
        return 1.0;
    }
    // Word overlap ratio
    let words_a: std::collections::HashSet<&str> = a.split_whitespace().collect();
    let words_b: std::collections::HashSet<&str> = b.split_whitespace().collect();
    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

fn dedup_by_text(slides: Vec<SlideData>) -> Vec<SlideData> {
    let mut unique: Vec<SlideData> = Vec::new();

    for slide in slides {
        // Skip slides with very short text (likely fragments from speaker close-ups)
        if slide.text.split_whitespace().count() < 5 {
            let _ = std::fs::remove_file(&slide.image_path);
            continue;
        }

        // Find matching existing slide by text similarity
        let match_idx = unique
            .iter()
            .position(|existing| text_similarity(&existing.text, &slide.text) > 0.6);

        match match_idx {
            Some(idx) => {
                // Keep the version with more text (cleaner slide capture)
                if slide.text.len() > unique[idx].text.len() {
                    let _ = std::fs::remove_file(&unique[idx].image_path);
                    unique[idx] = slide;
                } else {
                    let _ = std::fs::remove_file(&slide.image_path);
                }
            }
            None => unique.push(slide),
        }
    }
    unique
}

// ── Training data collection ────────────────────────────────────────────

fn save_training_data(labels: &[(PathBuf, String)]) {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("yt-sl")
        .join("training");

    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }

    let path = dir.join("labels.jsonl");
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(f) => f,
        Err(_) => return,
    };

    let mut count = 0;
    for (image_path, label) in labels {
        let abs = std::fs::canonicalize(image_path).unwrap_or_else(|_| image_path.clone());
        let entry = serde_json::json!({
            "input": TRAINING_PROMPT,
            "output": label,
            "image": abs.to_str().unwrap_or("")
        });
        if writeln!(file, "{}", entry).is_ok() {
            count += 1;
        }
    }

    if count > 0 {
        // Count total labels collected so far
        let total = std::fs::read_to_string(&path)
            .map(|s| s.lines().count())
            .unwrap_or(0);

        eprintln!("  training: +{} labels saved (total: {})", count, total);

        if total < 500 {
            eprintln!("  -> collect ~{} more labels before training", 500 - total);
        } else {
            eprintln!("  -> ready to train! run: oumi train -c classifier/src/train.yaml");
        }
    }
}

// ── Segment assignment ──────────────────────────────────────────────────────

fn assign_segments(slides: &mut [SlideData], segments: &[Segment], interval: f64) {
    for i in 0..slides.len() {
        let start = slides[i].timestamp;
        let end = if i + 1 < slides.len() {
            slides[i + 1].timestamp
        } else {
            start + interval * 10.0
        };

        let text: Vec<&str> = segments
            .iter()
            .filter(|s| s.start >= start && s.start < end)
            .map(|s| s.text.trim())
            .collect();
        slides[i].transcript = text.join(" ");
    }
}

// ── Markdown generation ─────────────────────────────────────────────────────

fn generate_markdown(
    title: &str,
    url: &str,
    transcript: &str,
    slides: &[SlideData],
    output: &Path,
) -> R<()> {
    let mut md = format!("# {}\n\n", title);

    if !url.is_empty() {
        md.push_str(&format!("**Source:** [{}]({})\n\n", url, url));
    }

    if !transcript.is_empty() {
        md.push_str("## Full Transcript\n\n<details>\n<summary>Click to expand</summary>\n\n");
        md.push_str(transcript);
        md.push_str("\n\n</details>\n\n");
    }

    md.push_str("## Slides\n\n");
    for slide in slides {
        let mins = slide.timestamp as u64 / 60;
        let secs = slide.timestamp as u64 % 60;
        md.push_str(&format!(
            "### Slide {} ({}:{:02})\n\n",
            slide.index, mins, secs
        ));

        let rel_path = slide.image_path.file_name().unwrap().to_str().unwrap();
        md.push_str(&format!(
            "![Slide {}](slides/{})\n\n",
            slide.index, rel_path
        ));

        md.push_str("#### Text\n\n");
        md.push_str(&slide.text);
        md.push_str("\n\n");

        if !slide.transcript.is_empty() {
            md.push_str("#### Speaker Notes\n\n");
            md.push_str(&slide.transcript);
            md.push_str("\n\n");
        }

        md.push_str("---\n\n");
    }

    std::fs::write(output, md)?;
    Ok(())
}
