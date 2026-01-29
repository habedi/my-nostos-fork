class Nostos < Formula
  desc "Functional programming language with lightweight processes and non-blocking I/O"
  homepage "https://github.com/pegesund/nostos"
  version "0.2.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/pegesund/nostos/releases/download/v#{version}/nostos-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "8771f55be42338c68a64e7f350caf74596ce7716e27a393d5c1e71c96d4e9c60"
    else
      url "https://github.com/pegesund/nostos/releases/download/v#{version}/nostos-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "5a3652e1942f8891dd6cd111728c9091a670c7e9fc631bc2c13c6323f660b831"
    end
  end

  on_linux do
    url "https://github.com/pegesund/nostos/releases/download/v#{version}/nostos-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "5979eca043cc831e74ef91e3970ccbc5a42aa6e530639ede6078c2577784d304"
  end

  def install
    bin.install "nostos"
    bin.install "nostos-lsp" if File.exist?("nostos-lsp")

    # Install stdlib (also embedded in binary, but useful for reference)
    (share/"nostos"/"stdlib").install Dir["stdlib/*"] if File.directory?("stdlib")
  end

  def caveats
    <<~EOS
      Nostos has been installed!

      To get started:
        nostos                    # Start the REPL
        nostos myfile.nos         # Run a program

      VS Code extension available at:
        https://github.com/pegesund/nostos/tree/master/editors/vscode
    EOS
  end

  test do
    assert_match "42", shell_output("echo 'main() = 42' | #{bin}/nostos /dev/stdin")
  end
end
