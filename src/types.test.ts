import { describe, expect, it } from "vitest";
import {
  AUDIO_OUTPUT_FORMATS,
  DEFAULT_SETTINGS,
  IMAGE_OUTPUT_FORMATS,
  QueueItem,
  VIDEO_OUTPUT_FORMATS,
  formatBytes,
  formatDuration,
  targetLabelForItem,
} from "./types";

function makeItem(partial: Partial<QueueItem>): QueueItem {
  return {
    id: "id-1",
    path: "/tmp/file.png",
    fileName: "file.png",
    sizeBytes: 1024,
    mediaKind: "image",
    sourceFormat: "PNG",
    width: 100,
    height: 100,
    durationSecs: null,
    thumbnailDataUrl: null,
    relativeDir: null,
    pageCount: null,
    status: "pending",
    progress: 0,
    ...partial,
  };
}

describe("types helpers", () => {
  it("formats bytes accurately across units", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(200 * 1024)).toBe("200 KB");
    expect(formatBytes(5.2 * 1024 * 1024)).toBe("5.2 MB");
    expect(formatBytes(2.5 * 1024 * 1024 * 1024)).toBe("2.5 GB");
  });

  it("formats durations cleanly and handles null / non-finite values", () => {
    expect(formatDuration(null)).toBe("");
    expect(formatDuration(Number.NaN)).toBe("");
    expect(formatDuration(Number.POSITIVE_INFINITY)).toBe("");
    expect(formatDuration(5)).toBe("0:05");
    expect(formatDuration(65.4)).toBe("1:05");
    expect(formatDuration(125)).toBe("2:05");
  });

  it("computes target labels for images, PDFs, videos, audio, and unknown fallbacks", () => {
    expect(targetLabelForItem(makeItem({ mediaKind: "image" }), DEFAULT_SETTINGS)).toBe("WebP");
    expect(
      targetLabelForItem(
        makeItem({ mediaKind: "pdf", overrideImageFormat: "png" }),
        DEFAULT_SETTINGS
      )
    ).toBe("PNG");
    expect(
      targetLabelForItem(
        makeItem({
          mediaKind: "image",
          overrideImageFormat: "svg",
        }),
        DEFAULT_SETTINGS
      )
    ).toBe("SVG");

    expect(targetLabelForItem(makeItem({ mediaKind: "video" }), DEFAULT_SETTINGS)).toBe(
      "MP4 (H.264)"
    );
    expect(
      targetLabelForItem(
        makeItem({
          mediaKind: "video",
          overrideVideoTarget: { kind: "video", format: "unknown_fmt" as never },
        }),
        DEFAULT_SETTINGS
      )
    ).toBe("VIDEO");

    expect(
      targetLabelForItem(
        makeItem({
          mediaKind: "audio",
          overrideVideoTarget: { kind: "audio", format: "mp3" },
        }),
        DEFAULT_SETTINGS
      )
    ).toBe("MP3 Audio");
    expect(
      targetLabelForItem(
        makeItem({
          mediaKind: "audio",
          overrideVideoTarget: { kind: "audio", format: "unknown_audio" as never },
        }),
        DEFAULT_SETTINGS
      )
    ).toBe("AUDIO");
  });

  it("exposes non-empty format catalogs and valid defaults", () => {
    expect(IMAGE_OUTPUT_FORMATS.length).toBeGreaterThanOrEqual(10);
    expect(VIDEO_OUTPUT_FORMATS.length).toBeGreaterThanOrEqual(9);
    expect(AUDIO_OUTPUT_FORMATS.length).toBeGreaterThanOrEqual(5);
    expect(DEFAULT_SETTINGS.image.format).toBe("webp");
    expect(DEFAULT_SETTINGS.video.target.kind).toBe("video");
  });
});
