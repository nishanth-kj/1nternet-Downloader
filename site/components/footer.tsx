import Link from "next/link";
import { FiDownload, FiFileText } from "react-icons/fi";
import { GoLaw } from "react-icons/go";
import { FaGithub } from "react-icons/fa";

export function Footer() {
  const currentYear = new Date().getFullYear();
  const yearDisplay = currentYear > 2026 ? `2026 - ${currentYear}` : "2026";

  return (
    <footer className="border-t bg-background">
      <div className="container mx-auto max-w-6xl px-4 py-12 sm:px-6 lg:px-8">
        <div className="flex flex-col gap-8 md:flex-row md:items-center md:justify-between">
          <div className="flex flex-col items-center gap-4 sm:flex-row md:items-center">
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-lg border bg-muted">
                <FiDownload className="h-5 w-5 text-primary" />
              </div>

              <div>
                <div className="font-semibold tracking-tight">
                  Internet-Downloader
                </div>
                <div className="text-xs text-muted-foreground">
                  Fast, simple downloads
                </div>
              </div>
            </div>

            <div className="hidden h-8 w-px bg-border sm:block" />

            <div className="text-sm text-muted-foreground">
              © {yearDisplay}
            </div>
          </div>

          <nav className="flex flex-wrap items-center justify-center gap-x-6 gap-y-3 text-sm">
            <Link
              href="https://github.com/nishanth-kj/1nternet-Downloader"
              className="flex items-center gap-2 text-muted-foreground transition-colors hover:text-foreground"
            >
              <FaGithub className="h-4 w-4" />
              GitHub
            </Link>

            <Link
              href="https://github.com/nishanth-kj/1nternet-Downloader/releases"
              className="text-muted-foreground transition-colors hover:text-foreground"
            >
              Releases
            </Link>

            <Link
              href="https://github.com/nishanth-kj/1nternet-Downloader"
              className="flex items-center gap-2 text-muted-foreground transition-colors hover:text-foreground"
            >
              <FiFileText className="h-4 w-4" />
              Documentation
            </Link>

            <Link
              href="https://github.com/nishanth-kj/1nternet-Downloader/blob/main/LICENSE"
              className="flex items-center gap-2 text-muted-foreground transition-colors hover:text-foreground"
            >
              <GoLaw className="h-4 w-4" />
              License
            </Link>
          </nav>
        </div>
      </div>
    </footer>
  );
}