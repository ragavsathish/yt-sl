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
    #[arg(short = 'T', long, default_value = "0.85")]
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

    let unique_frames = dedup_frames(&frame_paths, args.threshold);
    eprintln!(
        "[2/4] Dedup: {} frames -> {} unique",
        frame_paths.len(),
        unique_frames.len()
    );

    // 3. Vision OCR + classification
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let sem = std::sync::Arc::new(Semaphore::new(args.concurrency));
    let mut handles = Vec::new();

    for (i, frame_path) in unique_frames.iter().enumerate() {
        let client = client.clone();
        let sem = sem.clone();
        let model = args.model.clone();
        let api = args.vision_api.clone();
        let path = frame_path.clone();
        let dest = slides_dir.join(format!("slide_{:04}.jpg", i + 1));
        let timestamp = i as f64 * args.interval as f64;

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
    eprintln!("[3/4] OCR done: {} slides extracted", slides.len());

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

fn dedup_frames(paths: &[PathBuf], threshold: f64) -> Vec<PathBuf> {
    if paths.is_empty() {
        return vec![];
    }
    let mut unique = vec![paths[0].clone()];
    let mut last_hash = avg_hash(&paths[0]).unwrap_or(0);

    for path in &paths[1..] {
        if let Ok(hash) = avg_hash(path) {
            if hamming_similarity(last_hash, hash) < threshold {
                unique.push(path.clone());
                last_hash = hash;
            }
        }
    }
    unique
}

// ── Vision LLM (Qwen-VL) ───────────────────────────────────────────────────

const VISION_PROMPT: &str = "\
You are analyzing a frame extracted from a video recording of a presentation.

If this image shows a presentation slide (containing text, diagrams, charts, or bullet points), \
extract ALL visible text from the slide exactly as written, preserving structure with line breaks. \
Respond with the extracted text only, no preamble.

If this image does NOT show a presentation slide (e.g., it shows a speaker, audience, webcam view, \
or transition screen), respond with exactly: NOT_SLIDE";

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
        eprintln!("  training: +{} labels saved to {}", count, path.display());
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
