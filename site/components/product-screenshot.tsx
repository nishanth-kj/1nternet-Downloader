import { FiLayout } from "react-icons/fi";

export function ProductScreenshot() {
  return (
    <section className="py-20">
      <div className="container mx-auto max-w-6xl px-4 sm:px-6 lg:px-8 text-center">
        <h2 className="text-3xl font-bold tracking-tight mb-6">Built for Simple Downloads</h2>
        <p className="text-muted-foreground max-w-2xl mx-auto mb-12">
          Internet-Downloader is a desktop application focused on fast and reliable download management. No complicated setups, just a clean interface for getting your files.
        </p>
        
        <div className="w-full max-w-4xl mx-auto rounded-xl border bg-card p-2 shadow-sm">
          <div className="aspect-[16/9] w-full rounded-lg bg-muted flex items-center justify-center overflow-hidden border">
            <div className="text-center p-6">
              <FiLayout className="mx-auto h-12 w-12 text-muted-foreground mb-4 opacity-50" />
              <p className="text-sm font-medium text-muted-foreground">Application UI Placeholder</p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
