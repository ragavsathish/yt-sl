"""
Auto-label frames using Qwen-VL as teacher model.

Sends each frame to a local vision LLM (Qwen-VL via LM Studio)
to classify as SLIDE or NOT_SLIDE. Outputs JSONL for Oumi training.

Usage:
    python label.py --frames-dir /tmp/yt-sl-test-frames --output data/
    python label.py --frames-dir /tmp/yt-sl-test-frames --output data/ --api http://localhost:1234/v1
"""

import argparse
import base64
import json
import os
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed

import requests

CLASSIFY_PROMPT = (
    "Is this image a presentation slide (containing text, diagrams, charts, or bullet points)? "
    "Answer with exactly one word: SLIDE or NOT_SLIDE."
)

STUDENT_PROMPT = (
    "Is this image a presentation slide? Answer with exactly one word: SLIDE or NOT_SLIDE."
)


def classify_with_teacher(
    image_path: str,
    api_base: str,
    model: str,
) -> str | None:
    """Ask Qwen-VL teacher to classify a single frame."""
    image_data = Path(image_path).read_bytes()
    b64 = base64.b64encode(image_data).decode()
    data_url = f"data:image/jpeg;base64,{b64}"

    payload = {
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": CLASSIFY_PROMPT},
                    {"type": "image_url", "image_url": {"url": data_url}},
                ],
            }
        ],
    }

    try:
        resp = requests.post(
            f"{api_base}/chat/completions",
            json=payload,
            timeout=120,
        )
        resp.raise_for_status()
        content = resp.json()["choices"][0]["message"]["content"].strip().upper()

        if "NOT_SLIDE" in content:
            return "NOT_SLIDE"
        elif "SLIDE" in content:
            return "SLIDE"
        return None
    except Exception as e:
        print(f"    error: {e}")
        return None


def label_with_teacher(
    frames_dir: str,
    api_base: str,
    model: str,
    concurrency: int,
) -> dict[str, str]:
    """Label all frames using the teacher model."""
    frames = sorted(Path(frames_dir).glob("*.jpg")) + sorted(
        Path(frames_dir).glob("*.png")
    )
    print(f"Labeling {len(frames)} frames with teacher ({model})...")

    labels = {}

    def process(frame):
        label = classify_with_teacher(str(frame), api_base, model)
        return frame.name, label

    with ThreadPoolExecutor(max_workers=concurrency) as pool:
        futures = {pool.submit(process, f): f for f in frames}
        done = 0
        for future in as_completed(futures):
            name, label = future.result()
            done += 1
            if label:
                labels[name] = label
                print(f"  [{done}/{len(frames)}] {name}: {label}")
            else:
                print(f"  [{done}/{len(frames)}] {name}: SKIPPED")

    return labels


def write_dataset(labels: dict[str, str], frames_dir: str, output_dir: str):
    """Write labeled data as JSONL for Oumi training."""
    os.makedirs(output_dir, exist_ok=True)

    entries = []
    for filename, label in sorted(labels.items()):
        frame_path = os.path.abspath(os.path.join(frames_dir, filename))
        if not os.path.exists(frame_path):
            continue
        entries.append({
            "input": STUDENT_PROMPT,
            "output": label,
            "image": frame_path,
        })

    # 80/20 train/eval split
    split = int(len(entries) * 0.8)
    train = entries[:split]
    eval_set = entries[split:]

    for path, data in [
        (os.path.join(output_dir, "train.jsonl"), train),
        (os.path.join(output_dir, "eval.jsonl"), eval_set),
    ]:
        with open(path, "w") as f:
            for entry in data:
                f.write(json.dumps(entry) + "\n")

    slide_count = sum(1 for e in entries if e["output"] == "SLIDE")
    print(f"\nWrote {len(train)} train, {len(eval_set)} eval to {output_dir}/")
    print(f"  SLIDE: {slide_count}  NOT_SLIDE: {len(entries) - slide_count}")


def main():
    parser = argparse.ArgumentParser(
        description="Auto-label frames using Qwen-VL teacher model"
    )
    parser.add_argument("--frames-dir", required=True, help="Directory with frame images")
    parser.add_argument("--output", default="data/", help="Output directory for JSONL")
    parser.add_argument(
        "--api", default="http://localhost:1234/v1", help="Teacher model API base URL"
    )
    parser.add_argument(
        "--model", default="qwen/qwen3-vl-8b", help="Teacher model name"
    )
    parser.add_argument(
        "--concurrency", type=int, default=4, help="Concurrent API requests"
    )
    args = parser.parse_args()

    labels = label_with_teacher(args.frames_dir, args.api, args.model, args.concurrency)

    if labels:
        write_dataset(labels, args.frames_dir, args.output)
    else:
        print("No labels generated. Is the teacher model running?")


if __name__ == "__main__":
    main()
