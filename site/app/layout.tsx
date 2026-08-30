import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Internet-Downloader | Fast & Simple Desktop Download Manager",
  description: "Download, manage, pause, resume, and organize your downloads with a lightweight desktop application.",
  keywords: ["download manager", "internet downloader", "desktop downloader", "open source downloader", "fast download", "resume download"],
  authors: [{ name: "Nishanth K J", url: "https://github.com/nishanth-kj" }],
  openGraph: {
    title: "Internet-Downloader",
    description: "A fast and simple download manager for your desktop.",
    url: "https://nishanth-kj.github.io/1nternet-Downloader/",
    siteName: "Internet-Downloader",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Internet-Downloader",
    description: "A fast and simple download manager for your desktop.",
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
