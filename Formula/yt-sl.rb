class YtSl < Formula
  desc "Extract presentation slides from video frames using vision LLMs"
  homepage "https://github.com/ragavsathish/yt-sl"
  url "https://github.com/ragavsathish/yt-sl/archive/refs/tags/v0.3.0.tar.gz"
  sha256 "75cb23566fc9be3e7f66495228b45c4c78c18f603267a6d714601445d3597b3b"
  license "MIT"

  depends_on "rust" => :build
  depends_on "yt-dlp"
  depends_on "ffmpeg"

  def install
    system "cargo", "install", *std_cargo_args
    bin.install "yt-sl.sh"
  end

  def caveats
    <<~EOS
      yt-sl requires a local OpenAI-compatible vision model for OCR.
      Install LM Studio (https://lmstudio.ai/) and load a vision model
      like qwen/qwen3-vl-8b, then start the local server on port 1234.
    EOS
  end

  test do
    assert_match "yt-sl", shell_output("#{bin}/yt-sl --help")
  end
end
