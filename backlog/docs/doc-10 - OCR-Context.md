---
id: doc-10
title: OCR-Context
type: specification
created_date: '2026-01-15 22:15'
---
# OCR Context

> Extracts text from slide images using Tesseract OCR.

---

## Table of Contents

1. [Responsibility](#responsibility)
2. [Domain Types](#domain-types)
3. [Handlers](#handlers)
4. [Policies](#policies)
5. [External Dependencies](#external-dependencies)
6. [Testing](#testing)

---

## Responsibility

The OCR Context is responsible for:

- **Extracting** text from slide images
- **Detecting** slide language from text
- **Calculating** confidence scores for extracted text
- **Filtering** results by confidence threshold
- **Handling** multi-language OCR

---

## Domain Types

### SlideImage

Represents an image of a slide for OCR.

**Fields:**
- slide_id - Associated slide identifier
- image_path - Path to slide image
- width - Image width in pixels
- height - Image height in pixels

### ExtractedText

Represents text extracted from slide.

**Fields:**
- slide_id - Associated slide identifier
- text - Extracted text content
- confidence - OCR confidence score (0.0-1.0)
- language - Detected language
- regions - Optional list of text regions with coordinates

### TextRegion

Represents a region of text with coordinates.

**Fields:**
- text - Text content
- confidence - Region confidence score
- bounding_box - Rectangle coordinates (x, y, width, height)
- language - Detected language for region

---

## Handlers

### Extract Text

Extracts text from slide image using OCR.

**Input:** ExtractTextCommand
- slide_id - Slide to extract text from

**Output:** TextExtracted event
- slide_id - Slide identifier
- text - Extracted text
- confidence - OCR confidence score
- language - Detected language

**Behavior:**
- Loads slide image
- Preprocesses image for OCR
- Extracts text using Tesseract
- Calculates confidence score
- Detects language
- Returns error on OCR failure
- Extracts slide without text on failure (warning)

---

## Policies

### Confidence Filtering

Filters OCR results by confidence threshold.

**Default Threshold:** 0.5 (50%)

**Configurable Range:** 0.0-1.0

**Behavior:**
- Text below threshold is considered low quality
- Low confidence slides are marked in output
- User can adjust threshold based on quality needs

**Guidelines:**
- 0.8: High quality (only clear text)
- 0.5: Balanced (recommended)
- 0.3: Low quality (include fuzzy text)

---

### Image Preprocessing

Prepares images for better OCR results.

**Preprocessing Steps:**
1. **Grayscale conversion** - Reduces noise
2. **Binarization** - Converts to black/white
3. **Denoising** - Removes noise artifacts
4. **Contrast enhancement** - Improves text visibility

**Optional:**
- Upscaling (for low-resolution slides)
- Deskewing (corrects rotation)

### Language Detection

Detects language from extracted text.

**Supported Languages:**
- English
- Spanish
- French
- German
- Japanese
- Chinese
- Korean

**Detection Method:**
- Tesseract automatic language detection
- Confidence scores for each language
- Falls back to English if uncertain

**Configuration:**
- Can specify language manually
- Can enable/disable specific languages
- Default: Auto-detect

---

## External Dependencies

### OcrEngine

Extracts text from images.

**Signature:**
- `extract_text(&slide_id) -> Result<(text, confidence, language), error>`

**Implementation:** TesseractOcrEngine
- Uses tesseract-rs library
- Loads Tesseract language data
- Extracts text with confidence scores
- Returns error on OCR failure

### Tesseract Integration

**Tesseract Configuration:**
- Engine mode: LSTM (default)
- Page segmentation mode: Auto
- Language data: Downloaded on demand
- Image format: PNG

**Language Data:**
- Location: Configurable (default: /usr/share/tesseract-ocr)
- Files: eng.traineddata, spa.traineddata, etc.
- Downloaded from Tesseract GitHub if missing

---

## Testing

### Unit Tests

Test confidence filtering:
- Verify text above threshold is included
- Verify text below threshold is filtered
- Test edge cases (threshold = 0.0, threshold = 1.0)

Test language detection:
- Verify correct language detection for sample texts
- Test multilingual slides
- Verify fallback to English

Test preprocessing:
- Verify grayscale conversion
- Verify binarization
- Verify denoising

### Integration Tests

Test OCR extraction:
- Extract text from sample slides
- Verify text accuracy
- Verify confidence scores
- Test different languages

Test preprocessing:
- Test with various image qualities
- Verify preprocessing improves results
- Compare results with/without preprocessing

### Mock Dependencies

**MockOcrEngine:**
- Returns mock extracted text
- Simulates OCR delays
- Can simulate OCR failures
- Returns configurable confidence scores

---

## Implementation Notes

### Tesseract Setup

Install Tesseract:
- Ubuntu/Debian: `apt-get install tesseract-ocr`
- macOS: `brew install tesseract`
- Windows: Download installer from UB-Mannheim

Install language data:
- Ubuntu/Debian: `apt-get install tesseract-ocr-eng`
- macOS: Included with tesseract
- Windows: Download traineddata files

### Image Preprocessing Pipeline

Uses imageproc for preprocessing:

**Steps:**
1. Load image as grayscale
2. Apply Gaussian blur for denoising
3. Apply adaptive thresholding for binarization
4. Apply contrast adjustment
5. Save processed image

**Parameters:**
- Blur radius: 1.0
- Threshold block size: 11
- Contrast adjustment: 1.2

### OCR Process Flow

1. **Load slide image** from file system
2. **Preprocess image** for better results
3. **Extract text** using Tesseract
4. **Calculate confidence** from Tesseract output
5. **Detect language** from text content
6. **Return results** to caller

---

## Error Handling

### OCR Failures

**Causes:**
- Tesseract not installed
- Language data missing
- Image corrupted
- Memory insufficient
- Unsupported image format

**Handling:**
- Log error with details
- Return OcrFailed error
- Continue without text extraction (non-blocking)
- Mark slide confidence as None
- Include slide in output without text

### Language Detection Failures

**Causes:**
- Text too short for detection
- Multilingual content
- Unrecognized language

**Handling:**
- Log warning
- Default to English
- Allow manual language override

### Preprocessing Failures

**Causes:**
- Image loading error
- Unsupported image format
- Memory allocation failure

**Handling:**
- Log error with details
- Skip preprocessing
- Attempt OCR on original image
- Fallback to failure if original fails

---

## Performance Considerations

### OCR Speed

**Performance by Image Size:**
- 1920x1080: ~0.5-1.0 seconds
- 1280x720: ~0.3-0.5 seconds
- 640x480: ~0.1-0.2 seconds

**Optimizations:**
- Parallel processing for multiple slides
- Image resizing for faster OCR
- Batch processing (Tesseract 5.0+)

### Memory Usage

**Per Slide:**
- Image data: ~2-10MB
- Tesseract internal: ~50-100MB
- Peak usage: ~100-200MB

**Total for 100 slides:**
- Peak: ~200-500MB (with parallel processing)

---

## User Experience

### Progress Reporting

Reports OCR progress:

**Metrics:**
- Total slides to process
- Slides completed
- Average confidence score
- Languages detected

**Display:**
- Progress bar for OCR processing
- Spinner for active extraction
- Real-time confidence display

### Quality Indicators

Displays confidence scores in output:

**High Confidence (>0.8):**
- Marked as high quality
- No warnings in output

**Medium Confidence (0.5-0.8):**
- Marked as acceptable
- Optional warning in output

**Low Confidence (<0.5):**
- Marked as low quality
- Warning in output
- Suggests manual review

---

## Configuration Options

### OCR Configuration

**Parameters:**
- `confidence_threshold` - Minimum confidence (default: 0.5)
- `language` - Force specific language (default: auto-detect)
- `enable_preprocessing` - Apply image preprocessing (default: true)
- `upscale_factor` - Upscale images (default: 1.0, 2.0 for low-res)
- `parallel_jobs` - Number of parallel OCR jobs (default: CPU cores)

### CLI Flags

- `--ocr-languages` - Comma-separated list of languages
- `--ocr-confidence` - Minimum confidence threshold (0.0-1.0)
- `--no-preprocess` - Disable image preprocessing
- `--upscale` - Upscale images by 2x
