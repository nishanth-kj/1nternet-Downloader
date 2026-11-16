import Link from "next/link";

import { Logo } from "./logo";

const repositoryUrl = "https://github.com/nishanth-kj/1nternet-Downloader";

export function DownloadSection() {
  return (
    <section id="download" className="py-24 text-center">
      <div className="container mx-auto max-w-4xl px-4 sm:px-6 lg:px-8">
        <Logo size={72} className="mx-auto mb-6" />
        <h2 className="text-3xl font-bold tracking-tight mb-4">Download with 1IDM</h2>
        <p className="text-lg text-muted-foreground mb-10">
          Same mark on Windows, Android, and the browser extension.
        </p>
        <div className="grid gap-4 sm:grid-cols-3">
          {[
            { title: "Windows", href: `${repositoryUrl}/releases`, label: "Desktop app" },
            { title: "Android", href: `${repositoryUrl}/releases`, label: "APK" },
            { title: "Browser", href: `${repositoryUrl}/tree/main/browser/chrome`, label: "Download with browser" },
          ].map((item) => (
            <Link
              key={item.title}
              href={item.href}
              target="_blank"
              rel="noreferrer"
              className="flex flex-col items-center gap-3 rounded-xl border bg-card p-6 shadow-sm transition-colors hover:bg-accent"
            >
              <Logo size={48} />
              <span className="font-semibold">{item.title}</span>
              <span className="text-sm text-muted-foreground">{item.label}</span>
            </Link>
          ))}
        </div>
      </div>
    </section>
  );
}
