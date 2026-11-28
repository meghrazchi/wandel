cask "wandel" do
  arch arm: "aarch64", intel: "x64"

  version "3.5.2"
  sha256 :no_check

  url "https://github.com/meghrazchi/wandel/releases/download/v#{version}/Wandel_#{version}_#{arch}.dmg",
      verified: "github.com/meghrazchi/wandel/"
  name "Wandel"
  desc "Fast, private, offline Image, Video, Audio, and PDF converter"
  homepage "https://github.com/meghrazchi/wandel"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "Wandel.app"
  binary "#{appdir}/Wandel.app/Contents/MacOS/wandel-cli", target: "wandel"

  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-cr", "#{appdir}/Wandel.app"],
                   sudo: false
  end

  zap trash: [
    "~/Library/Application Support/app.wandel.converter",
    "~/Library/Caches/app.wandel.converter",
    "~/Library/Preferences/app.wandel.converter.plist",
  ]
end
