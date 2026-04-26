# typed: false
# frozen_string_literal: true

class TileserverRs < Formula
  desc "High-performance vector tile server with native MapLibre rendering"
  homepage "https://github.com/vinayakkulkarni/tileserver-rs"
  version "2.27.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/vinayakkulkarni/tileserver-rs/releases/download/v#{version}/tileserver-rs-aarch64-apple-darwin.tar.gz"
      sha256 "5a79d575db7e921e93adcc0853b5ba20ffac27c47627dbb831a33a9f38dbfa67"
    elsif Hardware::CPU.intel?
      url "https://github.com/vinayakkulkarni/tileserver-rs/releases/download/v#{version}/tileserver-rs-x86_64-apple-darwin.tar.gz"
      sha256 "TODO"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/vinayakkulkarni/tileserver-rs/releases/download/v#{version}/tileserver-rs-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "6ffc999a19751744cb31bf31ae5e45f2d082495e4af77a689100c159867d9033"
    elsif Hardware::CPU.intel?
      url "https://github.com/vinayakkulkarni/tileserver-rs/releases/download/v#{version}/tileserver-rs-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "d5f0ab50f8de510b8c1771d5f6bd0419a4feddf91040bc3ecb69c5f5a9bd0c27"
    end
  end

  def install
    bin.install "tileserver-rs"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tileserver-rs --version")
  end
end
