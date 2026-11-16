import Link from "next/link";

import { Logo } from "./logo";

const repositoryUrl = "https://github.com/nishanth-kj/1nternet-Downloader";

export function Footer() {
  return (
    <footer className="border-t">
      <div className="mx-auto flex max-w-6xl flex-col gap-3 px-5 py-8 text-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between sm:px-8">
        <p className="flex items-center gap-2">
          <Logo size={20} />
          1IDM · 1nternet Downloader
        </p>
        <div className="flex gap-5">
          <Link href={repositoryUrl} target="_blank" rel="noreferrer" className="hover:text-foreground">GitHub</Link>
          <Link href={`${repositoryUrl}/blob/main/LICENSE`} target="_blank" rel="noreferrer" className="hover:text-foreground">License</Link>
        </div>
      </div>
    </footer>
  );
}
