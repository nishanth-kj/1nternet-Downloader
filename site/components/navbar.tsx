"use client";

import { useState } from "react";
import Link from "next/link";
import { FiDownload, FiMenu, FiX } from "react-icons/fi";
import { ThemeToggle } from "./theme-toggle";

export function Navbar() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <header className="sticky top-0 z-40 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container mx-auto max-w-6xl px-4 sm:px-6 lg:px-8">
        <div className="flex h-16 items-center justify-between">
          <div className="flex items-center gap-2">
            <FiDownload className="h-6 w-6 text-primary" />
            <span className="text-lg font-bold tracking-tight">Internet-Downloader</span>
          </div>
          
          {/* Desktop Nav */}
          <nav className="hidden md:flex items-center gap-6 text-sm font-medium">
            <Link href="#features" className="transition-colors hover:text-primary">Features</Link>
            <Link href="#download" className="transition-colors hover:text-primary">Download</Link>
            <Link href="https://github.com/nishanth-kj/1nternet-Downloader" target="_blank" className="transition-colors hover:text-primary">GitHub</Link>
            <ThemeToggle />
          </nav>
          
          {/* Mobile Nav Toggle */}
          <div className="md:hidden flex items-center gap-4">
            <ThemeToggle />
            <button
              onClick={() => setIsOpen(!isOpen)}
              className="text-foreground p-1 focus:outline-none focus:ring-2 focus:ring-ring rounded-md"
              aria-label="Toggle menu"
            >
              {isOpen ? <FiX className="h-6 w-6" /> : <FiMenu className="h-6 w-6" />}
            </button>
          </div>
        </div>
      </div>

      {/* Mobile Nav Dropdown */}
      {isOpen && (
        <div className="md:hidden border-t bg-background">
          <nav className="flex flex-col items-center gap-2 py-4 px-4 text-sm font-medium shadow-inner bg-muted/20">
            <Link 
              href="#features" 
              className="transition-colors hover:text-primary w-full text-center py-3 rounded-md hover:bg-muted"
              onClick={() => setIsOpen(false)}
            >
              Features
            </Link>
            <Link 
              href="#download" 
              className="transition-colors hover:text-primary w-full text-center py-3 rounded-md hover:bg-muted"
              onClick={() => setIsOpen(false)}
            >
              Download
            </Link>
            <Link 
              href="https://github.com/nishanth-kj/1nternet-Downloader" 
              target="_blank" 
              className="transition-colors hover:text-primary w-full text-center py-3 rounded-md hover:bg-muted"
              onClick={() => setIsOpen(false)}
            >
              GitHub
            </Link>
          </nav>
        </div>
      )}
    </header>
  );
}
