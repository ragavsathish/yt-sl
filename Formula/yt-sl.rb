class YtSl < Formula
  desc "Extract presentation slides from video frames using vision LLMs"
  homepage "https://github.com/ragavsathish/yt-sl"
  url "https://github.com/ragavsathish/yt-sl/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER"
  license "MIT"

  depends_on "rust" => :build
  depends_on "yt-dlp"
  depends_on "ffmpeg"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "yt-sl", shell_output("#{bin}/yt-sl --help")
  end
end
