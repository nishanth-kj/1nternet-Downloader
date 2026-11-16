import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  metadataBase: new URL("https://nishanth-kj.github.io/1nternet-Downloader/"),
  alternates: {
    canonical: "/1nternet-Downloader/",
  },
  title: "1IDM | 1nternet Downloader",
  description: "Download, manage, pause, resume, and organize files on Windows, Android, and from the browser.",
  icons: {
    icon: "/logo.svg",
  },
  keywords: ["download manager", "internet downloader", "desktop downloader", "open source downloader", "fast download", "resume download"],
  authors: [{ name: "Nishanth K J", url: "https://github.com/nishanth-kj" }],
  openGraph: {
    title: "1IDM | 1nternet Downloader",
    description: "Download manager for Windows, Android, and the browser.",
    url: "https://nishanth-kj.github.io/1nternet-Downloader/",
    siteName: "1IDM",
    type: "website",
    images: [{ url: "/logo.svg", alt: "1IDM" }],
  },
  twitter: {
    card: "summary_large_image",
    title: "1IDM | 1nternet Downloader",
    description: "Download manager for Windows, Android, and the browser.",
    images: ["/logo.svg"],
  }
};

import { ThemeProvider } from "@/components/theme-provider";

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html
      lang="en"
      className={`${inter.variable} h-full antialiased`}
      suppressHydrationWarning
    >
      <body className="min-h-full flex flex-col">
        <ThemeProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange
        >
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}
