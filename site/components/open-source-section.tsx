import Link from "next/link";
import { Code } from "lucide-react";

export function OpenSourceSection() {
  return (
    <section className="border-t bg-muted/40 py-20">
      <div className="container mx-auto max-w-4xl px-4 sm:px-6 lg:px-8 text-center">
        <Code className="mx-auto h-12 w-12 text-primary mb-6" />
        <h2 className="text-3xl font-bold tracking-tight mb-4">Open Source</h2>
        <p className="text-muted-foreground mb-8 max-w-2xl mx-auto">
          Internet-Downloader is an open-source project. Explore the source code, contribute improvements, report issues, and follow development on GitHub.
        </p>
        <Link 
          href="https://github.com/nishanth-kj/1nternet-Downloader"
          target="_blank"
          className="inline-flex items-center justify-center rounded-md border border-input bg-background px-8 py-3 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="mr-2 h-4 w-4"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/></svg> View on GitHub
        </Link>
      </div>
    </section>
  );
}
