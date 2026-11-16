import type { MetadataRoute } from "next";

// Required for metadata routes when next.config.ts uses `output: "export"`.
export const dynamic = "force-static";

const siteUrl = "https://nishanth-kj.github.io/1nternet-Downloader";

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    {
      url: `${siteUrl}/`,
      lastModified: new Date(),
      changeFrequency: "monthly",
      priority: 1,
    },
  ];
}
