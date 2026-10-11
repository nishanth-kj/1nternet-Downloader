import Link from "next/link";
import { FiDownload } from "react-icons/fi";
import { FaGithub } from "react-icons/fa";

export function Hero() {
  return (
    <section className="container mx-auto max-w-6xl px-4 sm:px-6 lg:px-8 py-20 md:py-32 flex flex-col items-center text-center">
      <h1 className="text-4xl md:text-6xl font-extrabold tracking-tight mb-6">
        Internet-Downloader
      </h1>
      <p className="text-xl md:text-2xl text-muted-foreground font-medium mb-4 max-w-2xl">
        A fast and simple download manager for your desktop.
      </p>
      <p className="text-base text-muted-foreground mb-10 max-w-xl">
        Download, manage, pause, resume, and organize your downloads with a lightweight desktop application.
      </p>
      <div className="flex flex-col sm:flex-row gap-4 w-full sm:w-auto">
        <Link
          href="https://github.com/nishanth-kj/1nternet-Downloader/releases"
          target="_blank"
          rel="noreferrer"
          className="inline-flex items-center justify-center rounded-md bg-primary px-8 py-3 text-sm font-medium text-primary-foreground shadow transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <FiDownload className="mr-2 h-4 w-4" /> View releases
        </Link>
        <Link 
          href="https://github.com/nishanth-kj/1nternet-Downloader"
          target="_blank"
          className="inline-flex items-center justify-center rounded-md border border-input bg-background px-8 py-3 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <FaGithub className="mr-2 h-4 w-4" /> View on GitHub
        </Link>
      </div>

      <div className="mt-16 w-full max-w-4xl rounded-xl border bg-card p-8 shadow-sm text-left">
        <p className="text-sm font-semibold uppercase tracking-wide text-muted-foreground mb-5">Current capabilities</p>
        <ul className="grid gap-4 sm:grid-cols-2">
          <li><strong>HTTP and HTTPS</strong><p className="text-sm text-muted-foreground">Download files with progress reporting.</p></li>
          <li><strong>Pause and resume</strong><p className="text-sm text-muted-foreground">Continue supported downloads after interruptions.</p></li>
          <li><strong>Desktop queue</strong><p className="text-sm text-muted-foreground">Manage multiple downloads from one native app.</p></li>
          <li><strong>Browser handoff</strong><p className="text-sm text-muted-foreground">Send supported links from the extension to the desktop app.</p></li>
        </ul>
        <p className="mt-6 text-xs text-muted-foreground">Feature availability differs by platform. Check the README and release notes before installing.</p>
      </div>
    </section>
  );
}
