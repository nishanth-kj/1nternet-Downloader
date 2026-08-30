import { FiZap, FiRefreshCw, FiLayers, FiLayout, FiHardDrive } from "react-icons/fi";

export function Features() {
  return (
    <section id="features" className="border-t bg-muted/40 py-20">
      <div className="container mx-auto max-w-6xl px-4 sm:px-6 lg:px-8">
        <h2 className="text-3xl font-bold tracking-tight text-center mb-12">Features</h2>
        
        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
          <div className="rounded-lg border bg-card p-6 shadow-sm">
            <FiZap className="h-6 w-6 text-primary mb-4" />
            <h3 className="text-lg font-semibold mb-2">Fast Downloads</h3>
            <p className="text-sm text-muted-foreground">Download files efficiently with a lightweight download engine.</p>
          </div>
          <div className="rounded-lg border bg-card p-6 shadow-sm">
            <FiRefreshCw className="h-6 w-6 text-primary mb-4" />
            <h3 className="text-lg font-semibold mb-2">Resume Downloads</h3>
            <p className="text-sm text-muted-foreground">Pause and resume interrupted downloads with ease.</p>
          </div>
          <div className="rounded-lg border bg-card p-6 shadow-sm">
            <FiLayers className="h-6 w-6 text-primary mb-4" />
            <h3 className="text-lg font-semibold mb-2">Multiple Downloads</h3>
            <p className="text-sm text-muted-foreground">Manage multiple downloads from one unified application.</p>
          </div>
          <div className="rounded-lg border bg-card p-6 shadow-sm">
            <FiLayout className="h-6 w-6 text-primary mb-4" />
            <h3 className="text-lg font-semibold mb-2">Download Management</h3>
            <p className="text-sm text-muted-foreground">Organize and monitor active and completed downloads intuitively.</p>
          </div>
          <div className="rounded-lg border bg-card p-6 shadow-sm">
            <FiHardDrive className="h-6 w-6 text-primary mb-4" />
            <h3 className="text-lg font-semibold mb-2">Torrent Support</h3>
            <p className="text-sm text-muted-foreground">Support torrent downloads directly within the application.</p>
          </div>
          <div className="rounded-lg border bg-card p-6 shadow-sm">
            <FiZap className="h-6 w-6 text-primary mb-4" />
            <h3 className="text-lg font-semibold mb-2">Lightweight</h3>
            <p className="text-sm text-muted-foreground">Designed to provide a simple and efficient desktop experience without bloat.</p>
          </div>
        </div>
      </div>
    </section>
  );
}
