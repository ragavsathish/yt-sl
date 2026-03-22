class YtSl < Formula
  desc "Extract presentation slides from video frames using vision LLMs"
  homepage "https://github.com/ragavsathish/yt-sl"
  url "https://github.com/ragavsathish/yt-sl/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "d57645334d15139451b0cf478fb2213142b61fa4f10047b713e589b7513ce275"
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
