import type { MetadataRoute } from "next";

const siteUrl = "https://nishanth-kj.github.io/1nternet-Downloader";

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    {
      url: `${siteUrl}/`,
      lastModified: new Date("2026-10-11"),
      changeFrequency: "monthly",
      priority: 1,
    },
  ];
}
