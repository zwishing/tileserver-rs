# typed: false
# frozen_string_literal: true

class TileserverRs < Formula
  desc "High-performance vector tile server with native MapLibre rendering"
  homepage "https://github.com/vinayakkulkarni/tileserver-rs"
  version "2.19.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/vinayakkulkarni/tileserver-rs/releases/download/v#{version}/tileserver-rs-aarch64-apple-darwin.tar.gz"
      sha256 "318baabe0361785c77ab27844a69f68cf91726960dda2a46042b2ca3bb301361"
    elsif Hardware::CPU.intel?
      url "https://github.com/vinayakkulkarni/tileserver-rs/releases/download/v#{version}/tileserver-rs-x86_64-apple-darwin.tar.gz"
      sha256 "TODO"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/vinayakkulkarni/tileserver-rs/releases/download/v#{version}/tileserver-rs-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "945d082c485bdfff7c9726c9a9cc3117ba4b27e09987907eb3f47f96b7ecc211"
    elsif Hardware::CPU.intel?
      url "https://github.com/vinayakkulkarni/tileserver-rs/releases/download/v#{version}/tileserver-rs-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "d33e19fad8fec1297a46ed0369b908e8d8c96a890e188974253f06b98098f8e6"
    end
  end

  def install
    bin.install "tileserver-rs"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tileserver-rs --version")
  end
end
