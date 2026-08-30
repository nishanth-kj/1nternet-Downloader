import Link from "next/link";
import { FiDownload, FiLayout } from "react-icons/fi";
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
          href="#download"
          className="inline-flex items-center justify-center rounded-md bg-primary px-8 py-3 text-sm font-medium text-primary-foreground shadow transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <FiDownload className="mr-2 h-4 w-4" /> Download for Windows
        </Link>
        <Link 
          href="https://github.com/nishanth-kj/1nternet-Downloader"
          target="_blank"
          className="inline-flex items-center justify-center rounded-md border border-input bg-background px-8 py-3 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <FaGithub className="mr-2 h-4 w-4" /> View on GitHub
        </Link>
      </div>

      <div className="mt-16 w-full max-w-4xl rounded-xl border bg-card p-2 shadow-sm">
        <div className="aspect-[16/9] w-full rounded-lg bg-muted flex items-center justify-center overflow-hidden border">
          <div className="text-center p-6">
            <FiLayout className="mx-auto h-12 w-12 text-muted-foreground mb-4 opacity-50" />
            <p className="text-sm font-medium text-muted-foreground">Application Screenshot Placeholder</p>
          </div>
        </div>
      </div>
    </section>
  );
}
