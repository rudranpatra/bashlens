# Homebrew formula stub. NOT yet functional: the sha256 values below are
# placeholders (`REPLACE_ME_*`) because no tagged release exists yet - the
# `.github/workflows/release.yml` workflow produces the tarballs/checksums
# this formula needs the moment `v0.1.0` is actually tagged and released.
#
# To activate:
#   1. git tag v0.1.0 && git push --tags   (triggers release.yml)
#   2. Copy the four sha256 values from the release's checksums.txt
#   3. Replace the REPLACE_ME_* placeholders below
#   4. Publish this file to a tap, e.g. github.com/rudranpatra/homebrew-bashlens
#
# Until then: `brew install --build-from-source` against this formula will
# fail on the placeholder checksums by design, rather than silently
# installing something unverifiable.
class Bashlens < Formula
  desc "Inspect shell install scripts before you run them"
  homepage "https://github.com/rudranpatra/bashlens"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/rudranpatra/bashlens/releases/download/v0.1.0/bashlens-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_ME_AARCH64_APPLE_DARWIN"
    else
      url "https://github.com/rudranpatra/bashlens/releases/download/v0.1.0/bashlens-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_ME_X86_64_APPLE_DARWIN"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/rudranpatra/bashlens/releases/download/v0.1.0/bashlens-aarch64-unknown-linux-musl.tar.gz"
      sha256 "REPLACE_ME_AARCH64_LINUX_MUSL"
    else
      url "https://github.com/rudranpatra/bashlens/releases/download/v0.1.0/bashlens-x86_64-unknown-linux-musl.tar.gz"
      sha256 "REPLACE_ME_X86_64_LINUX_MUSL"
    end
  end

  def install
    bin.install "bashlens"
  end

  test do
    system "#{bin}/bashlens", "--help"
  end
end
