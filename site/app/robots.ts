import type { MetadataRoute } from "next";

const siteUrl = "https://nishanth-kj.github.io";

export default function robots(): MetadataRoute.Robots {
  return {
    rules: {
      userAgent: "*",
      allow: "/",
    },
    sitemap: `${siteUrl}/1nternet-Downloader/sitemap.xml`,
    host: `${siteUrl}/1nternet-Downloader/`,
  };
}
