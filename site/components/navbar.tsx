import Link from "next/link";
import { Download } from "lucide-react";
import { ThemeToggle } from "./theme-toggle";

export function Navbar() {
  return (
    <header className="sticky top-0 z-40 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container mx-auto max-w-6xl px-4 sm:px-6 lg:px-8">
        <div className="flex h-16 items-center justify-between">
          <div className="flex items-center gap-2">
            <Download className="h-6 w-6 text-primary" />
            <span className="text-lg font-bold tracking-tight">Internet-Downloader</span>
          </div>
          <nav className="hidden md:flex items-center gap-6 text-sm font-medium">
            <Link href="#features" className="transition-colors hover:text-primary">Features</Link>
            <Link href="#download" className="transition-colors hover:text-primary">Download</Link>
            <Link href="https://github.com/nishanth-kj/1nternet-Downloader" target="_blank" className="transition-colors hover:text-primary">GitHub</Link>
            <ThemeToggle />
          </nav>
          <div className="md:hidden flex items-center gap-4">
            <ThemeToggle />
          </div>
        </div>
      </div>
    </header>
  );
}
