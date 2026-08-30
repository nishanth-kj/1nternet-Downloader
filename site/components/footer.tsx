import Link from "next/link";
import { Download } from "lucide-react";

export function Footer() {
  return (
    <footer className="border-t py-10">
      <div className="container mx-auto max-w-6xl px-4 sm:px-6 lg:px-8">
        <div className="flex flex-col md:flex-row justify-between items-center gap-6">
          <div className="flex items-center gap-2">
            <Download className="h-5 w-5 text-primary" />
            <span className="font-semibold">Internet-Downloader</span>
          </div>
          <div className="flex flex-wrap justify-center gap-6 text-sm font-medium text-muted-foreground">
            <Link href="https://github.com/nishanth-kj/1nternet-Downloader" className="hover:text-foreground transition-colors">GitHub</Link>
            <Link href="https://github.com/nishanth-kj/1nternet-Downloader/releases" className="hover:text-foreground transition-colors">Releases</Link>
            <Link href="https://github.com/nishanth-kj/1nternet-Downloader" className="hover:text-foreground transition-colors">Documentation</Link>
            <Link href="https://github.com/nishanth-kj/1nternet-Downloader/blob/main/LICENSE" className="hover:text-foreground transition-colors">License</Link>
          </div>
        </div>
        <div className="mt-8 text-center text-sm text-muted-foreground">
          © 2026 Internet-Downloader
        </div>
      </div>
    </footer>
  );
}
