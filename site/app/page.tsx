import Link from "next/link";
import {
  FiDownload,
  FiGithub,
  FiLayers,
  FiPauseCircle,
  FiShield,
} from "react-icons/fi";

import { Footer } from "@/components/footer";
import { Navbar } from "@/components/navbar";

const releaseUrl = "https://github.com/nishanth-kj/1nternet-Downloader/releases";
const repositoryUrl = "https://github.com/nishanth-kj/1nternet-Downloader";

const benefits = [
  {
    icon: FiLayers,
    title: "Keep everything together",
    text: "See active, paused, and finished downloads in one straightforward list.",
  },
  {
    icon: FiPauseCircle,
    title: "Pick up where you left off",
    text: "Pause a download when you need to and continue it later when the server supports it.",
  },
  {
    icon: FiShield,
    title: "Made for reliable downloads",
    text: "The desktop app validates download responses before saving your files.",
  },
];

export default function Home() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <Navbar />
      <main>
        <section className="border-b">
          <div className="mx-auto grid max-w-6xl gap-14 px-5 py-20 sm:px-8 md:grid-cols-[1.05fr_.95fr] md:items-center md:py-28">
            <div>
              <p className="mb-5 text-sm font-semibold text-primary">INTERNET DOWNLOADER</p>
              <h1 className="max-w-xl text-4xl font-bold tracking-tight sm:text-5xl">
                Downloads, without the clutter.
              </h1>
              <p className="mt-6 max-w-lg text-lg leading-8 text-muted-foreground">
                A simple Windows download manager for keeping files moving,
                pausing when needed, and seeing what is happening at a glance.
              </p>
              <div className="mt-8 flex flex-col gap-3 sm:flex-row">
                <Link href={releaseUrl} target="_blank" rel="noreferrer" className="inline-flex items-center justify-center gap-2 rounded-md bg-primary px-5 py-3 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90">
                  <FiDownload className="h-4 w-4" /> Get the desktop app
                </Link>
                <Link href="#how-it-works" className="inline-flex items-center justify-center rounded-md border px-5 py-3 text-sm font-medium transition-colors hover:bg-muted">
                  See how it works
                </Link>
              </div>
              <p className="mt-4 text-sm text-muted-foreground">Windows desktop app · Browser handoff available</p>
            </div>

            <div className="rounded-xl border bg-card p-3 shadow-sm">
              <div className="overflow-hidden rounded-lg border bg-background">
                <div className="flex items-center justify-between border-b px-4 py-3 text-sm">
                  <span className="font-semibold">Downloads</span>
                  <span className="rounded-full bg-primary/10 px-2.5 py-1 text-xs font-medium text-primary">2 active</span>
                </div>
                <div className="space-y-4 p-4">
                  <DownloadRow name="holiday-photos.zip" progress="74%" status="4.8 MB/s" value="74" />
                  <DownloadRow name="project-files.tar.gz" progress="28%" status="2.1 MB/s" value="28" />
                  <DownloadRow name="readme.pdf" progress="Done" status="Finished" value="100" muted />
                </div>
              </div>
            </div>
          </div>
        </section>

        <section id="features" className="mx-auto max-w-6xl px-5 py-20 sm:px-8">
          <div className="max-w-2xl">
            <p className="text-sm font-semibold text-primary">JUST THE ESSENTIALS</p>
            <h2 className="mt-3 text-3xl font-bold tracking-tight">One place for your files.</h2>
          </div>
          <div className="mt-10 grid gap-5 md:grid-cols-3">
            {benefits.map(({ icon: Icon, title, text }) => (
              <article key={title} className="rounded-lg border bg-card p-6">
                <Icon className="h-5 w-5 text-primary" />
                <h3 className="mt-5 font-semibold">{title}</h3>
                <p className="mt-2 text-sm leading-6 text-muted-foreground">{text}</p>
              </article>
            ))}
          </div>
        </section>

        <section id="how-it-works" className="border-y bg-muted/40">
          <div className="mx-auto max-w-6xl px-5 py-20 sm:px-8">
            <div className="max-w-2xl">
              <p className="text-sm font-semibold text-primary">HOW IT WORKS</p>
              <h2 className="mt-3 text-3xl font-bold tracking-tight">Start in three small steps.</h2>
            </div>
            <ol className="mt-10 grid gap-8 md:grid-cols-3">
              {[
                "Install the Windows desktop app.",
                "Add a file link or send one from the browser extension.",
                "Follow the progress, pause, resume, or open the finished file.",
              ].map((step, index) => (
                <li key={step} className="flex gap-4">
                  <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary text-sm font-semibold text-primary-foreground">{index + 1}</span>
                  <p className="pt-1 text-sm leading-6 text-muted-foreground">{step}</p>
                </li>
              ))}
            </ol>
          </div>
        </section>

        <section id="download" className="mx-auto max-w-3xl px-5 py-20 text-center sm:px-8">
          <FiDownload className="mx-auto h-7 w-7 text-primary" />
          <h2 className="mt-5 text-3xl font-bold tracking-tight">Ready when you are.</h2>
          <p className="mt-4 text-muted-foreground">Download the latest desktop release, or read the project details on GitHub.</p>
          <div className="mt-8 flex flex-col justify-center gap-3 sm:flex-row">
            <Link href={releaseUrl} target="_blank" rel="noreferrer" className="inline-flex items-center justify-center gap-2 rounded-md bg-primary px-5 py-3 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90">
              <FiDownload className="h-4 w-4" /> View releases
            </Link>
            <Link href={repositoryUrl} target="_blank" rel="noreferrer" className="inline-flex items-center justify-center gap-2 rounded-md border px-5 py-3 text-sm font-medium transition-colors hover:bg-muted">
              <FiGithub className="h-4 w-4" /> Project on GitHub
            </Link>
          </div>
          <p className="mt-5 text-xs text-muted-foreground">Platform support and feature availability can vary. Review the release notes before installing.</p>
        </section>
      </main>
      <Footer />
    </div>
  );
}

function DownloadRow({ name, progress, status, value, muted = false }: { name: string; progress: string; status: string; value: string; muted?: boolean }) {
  return (
    <div>
      <div className="flex items-center justify-between gap-4 text-sm">
        <span className="truncate font-medium">{name}</span>
        <span className="shrink-0 text-xs text-muted-foreground">{progress}</span>
      </div>
      <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-muted">
        <div className={`h-full rounded-full ${muted ? "bg-muted-foreground/60" : "bg-primary"}`} style={{ width: `${value}%` }} />
      </div>
      <p className="mt-1.5 text-xs text-muted-foreground">{status}</p>
    </div>
  );
}
