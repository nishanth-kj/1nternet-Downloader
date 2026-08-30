import Link from "next/link";
import { Download, Layout } from "lucide-react";

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
          <Download className="mr-2 h-4 w-4" /> Download for Windows
        </Link>
        <Link 
          href="https://github.com/nishanth-kj/1nternet-Downloader"
          target="_blank"
          className="inline-flex items-center justify-center rounded-md border border-input bg-background px-8 py-3 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="mr-2 h-4 w-4"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/></svg> View on GitHub
        </Link>
      </div>

      <div className="mt-16 w-full max-w-4xl rounded-xl border bg-card p-2 shadow-sm">
        <div className="aspect-[16/9] w-full rounded-lg bg-muted flex items-center justify-center overflow-hidden border">
          <div className="text-center p-6">
            <Layout className="mx-auto h-12 w-12 text-muted-foreground mb-4 opacity-50" />
            <p className="text-sm font-medium text-muted-foreground">Application Screenshot Placeholder</p>
          </div>
        </div>
      </div>
    </section>
  );
}
