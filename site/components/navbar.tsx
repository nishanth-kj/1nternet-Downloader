"use client";

import { useState } from "react";
import Link from "next/link";
import { FiMenu, FiX } from "react-icons/fi";

import { Logo } from "./logo";
import { ThemeToggle } from "./theme-toggle";

const repositoryUrl = "https://github.com/nishanth-kj/1nternet-Downloader";

export function Navbar() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <header className="sticky top-0 z-40 border-b bg-background/95 backdrop-blur">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-5 sm:px-8">
        <Link href="/" className="flex items-center gap-2 font-semibold tracking-tight">
          <Logo size={32} />
          <span>1IDM</span>
        </Link>
        <nav className="hidden items-center gap-6 text-sm md:flex">
          <Link href="#features" className="text-muted-foreground transition-colors hover:text-foreground">Features</Link>
          <Link href="#how-it-works" className="text-muted-foreground transition-colors hover:text-foreground">How it works</Link>
          <Link href="#download" className="text-muted-foreground transition-colors hover:text-foreground">Download</Link>
          <Link href={repositoryUrl} target="_blank" rel="noreferrer" className="text-muted-foreground transition-colors hover:text-foreground">GitHub</Link>
          <ThemeToggle />
        </nav>
        <div className="flex items-center gap-2 md:hidden">
          <ThemeToggle />
          <button onClick={() => setIsOpen((open) => !open)} className="rounded-md p-2 hover:bg-muted" aria-label="Toggle menu">
            {isOpen ? <FiX className="h-5 w-5" /> : <FiMenu className="h-5 w-5" />}
          </button>
        </div>
      </div>
      {isOpen && (
        <nav className="border-t bg-background px-5 py-3 md:hidden">
          {[['Features', '#features'], ['How it works', '#how-it-works'], ['Download', '#download']].map(([label, href]) => (
            <Link key={href} href={href} onClick={() => setIsOpen(false)} className="block rounded-md px-3 py-2.5 text-sm text-muted-foreground hover:bg-muted hover:text-foreground">{label}</Link>
          ))}
          <Link href={repositoryUrl} target="_blank" rel="noreferrer" className="block rounded-md px-3 py-2.5 text-sm text-muted-foreground hover:bg-muted hover:text-foreground">GitHub</Link>
        </nav>
      )}
    </header>
  );
}
