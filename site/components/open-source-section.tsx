import Link from "next/link";
import { FiCode } from "react-icons/fi";
import { FaGithub } from "react-icons/fa";

export function OpenSourceSection() {
  return (
    <section className="border-t bg-muted/40 py-20">
      <div className="container mx-auto max-w-4xl px-4 sm:px-6 lg:px-8 text-center">
        <FiCode className="mx-auto h-12 w-12 text-primary mb-6" />
        <h2 className="text-3xl font-bold tracking-tight mb-4">Open Source</h2>
        <p className="text-muted-foreground mb-8 max-w-2xl mx-auto">
          Internet-Downloader is an open-source project. Explore the source code, contribute improvements, report issues, and follow development on GitHub.
        </p>
        <Link 
          href="https://github.com/nishanth-kj/1nternet-Downloader"
          target="_blank"
          className="inline-flex items-center justify-center rounded-md border border-input bg-background px-8 py-3 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <FaGithub className="mr-2 h-4 w-4" /> View on GitHub
        </Link>
      </div>
    </section>
  );
}
