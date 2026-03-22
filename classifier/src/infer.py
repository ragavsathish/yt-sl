"""
Run inference with the fine-tuned slide classifier.

Usage:
    python infer.py --image frame.jpg --model output/slide-classifier/
    python infer.py --frames-dir /tmp/frames --model output/slide-classifier/
"""

import argparse
import json
from pathlib import Path

from oumi.core.configs import InferenceConfig, ModelParams
from oumi.inference import NativeTextInferenceEngine


PROMPT = "Is this image a presentation slide? Answer with exactly one word: SLIDE or NOT_SLIDE."


def classify_image(engine, image_path: str) -> tuple[str, str]:
    """Classify a single image. Returns (label, raw_response)."""
    conversation = {
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "image_url", "image_url": {"url": image_path}},
                    {"type": "text", "text": PROMPT},
                ],
            }
        ]
    }

    response = engine.infer(conversation)
    raw = response["messages"][-1]["content"].strip().upper()

    if "NOT_SLIDE" in raw:
        return "NOT_SLIDE", raw
    elif "SLIDE" in raw:
        return "SLIDE", raw
    else:
        return "UNKNOWN", raw


def main():
    parser = argparse.ArgumentParser(description="Classify frames with fine-tuned model")
    parser.add_argument("--image", help="Single image to classify")
    parser.add_argument("--frames-dir", help="Directory of frames to classify")
    parser.add_argument("--model", required=True, help="Path to fine-tuned model checkpoint")
    parser.add_argument("--output", help="Output JSON with results")
    args = parser.parse_args()

    config = InferenceConfig(
        model=ModelParams(
            model_name_or_path=args.model,
            trust_remote_code=True,
        ),
        generation={"max_new_tokens": 10, "temperature": 0.0},
    )

    engine = NativeTextInferenceEngine(config)

    if args.image:
        label, raw = classify_image(engine, args.image)
        print(f"{args.image}: {label}")

    elif args.frames_dir:
        frames = sorted(Path(args.frames_dir).glob("*.jpg")) + sorted(
            Path(args.frames_dir).glob("*.png")
        )
        results = {}
        slides = 0
        for frame in frames:
            label, raw = classify_image(engine, str(frame))
            results[frame.name] = label
            if label == "SLIDE":
                slides += 1
            print(f"  {frame.name}: {label}")

        print(f"\n{slides}/{len(frames)} classified as SLIDE")

        if args.output:
            with open(args.output, "w") as f:
                json.dump(results, f, indent=2)
            print(f"Results saved to {args.output}")


if __name__ == "__main__":
    main()
