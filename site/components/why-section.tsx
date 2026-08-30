export function WhySection() {
  return (
    <section className="border-t bg-muted/40 py-20">
      <div className="container mx-auto max-w-6xl px-4 sm:px-6 lg:px-8">
        <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-8 text-center">
          <div>
            <h3 className="text-xl font-semibold mb-2">Lightweight</h3>
            <p className="text-sm text-muted-foreground">Uses minimal system resources.</p>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-2">Fast</h3>
            <p className="text-sm text-muted-foreground">Optimized for maximum speeds.</p>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-2">Easy to use</h3>
            <p className="text-sm text-muted-foreground">Clean and straightforward interface.</p>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-2">Open source</h3>
            <p className="text-sm text-muted-foreground">Fully transparent and community-driven.</p>
          </div>
        </div>
      </div>
    </section>
  );
}
