# Verified against the real v0.1.0 GitHub release: checksums below are
# copied from that release's checksums.txt and the download+run path was
# tested against the actual release asset (see corpus/CORPUS.md history /
# commit log - not a placeholder formula).
#
# Not yet published to a tap: to make `brew install rudranpatra/bashlens/bashlens`
# work, publish this file to github.com/rudranpatra/homebrew-bashlens (or
# add it to an existing tap) as `Formula/bashlens.rb`.
class Bashlens < Formula
  desc "Inspect shell install scripts before you run them"
  homepage "https://github.com/rudranpatra/bashlens"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/rudranpatra/bashlens/releases/download/v0.1.0/bashlens-aarch64-apple-darwin.tar.gz"
      sha256 "c6f2eccda3d9c40948b60ddf2238a358b7c9cf1e42946a373e8b0a134042f553"
    else
      url "https://github.com/rudranpatra/bashlens/releases/download/v0.1.0/bashlens-x86_64-apple-darwin.tar.gz"
      sha256 "74c9f00402f9cda90c579ffa46be43a3fd19f9f491235147cf441549ea12f843"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/rudranpatra/bashlens/releases/download/v0.1.0/bashlens-aarch64-unknown-linux-musl.tar.gz"
      sha256 "e6b6f9db4ff874dad1d2fe131b5a9d9648da9259fd95c57df36ff9d7a1988691"
    else
      url "https://github.com/rudranpatra/bashlens/releases/download/v0.1.0/bashlens-x86_64-unknown-linux-musl.tar.gz"
      sha256 "9f685c39588a35dcca440480bfb9786dd32959ee8ff04b8d9caf32ac67cbaf3e"
    end
  end

  def install
    bin.install "bashlens"
  end

  test do
    system "#{bin}/bashlens", "--help"
  end
end
