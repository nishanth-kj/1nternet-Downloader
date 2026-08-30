import Link from "next/link";
import { FiDownload } from "react-icons/fi";

export function DownloadSection() {
  return (
    <section id="download" className="py-24 text-center">
      <div className="container mx-auto max-w-4xl px-4 sm:px-6 lg:px-8">
        <h2 className="text-3xl font-bold tracking-tight mb-4">Download Internet-Downloader</h2>
        <p className="text-lg text-muted-foreground mb-8">
          Get the latest version of Internet-Downloader for your desktop.
        </p>
        <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
          <Link 
            href="#"
            className="inline-flex w-full sm:w-auto items-center justify-center rounded-md bg-primary px-8 py-3 text-sm font-medium text-primary-foreground shadow transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <FiDownload className="mr-2 h-4 w-4" /> Download for Windows
          </Link>
          <Link 
            href="https://github.com/nishanth-kj/1nternet-Downloader/releases"
            target="_blank"
            className="inline-flex w-full sm:w-auto items-center justify-center rounded-md border border-input bg-background px-8 py-3 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            View Releases
          </Link>
        </div>
      </div>
    </section>
  );
}
