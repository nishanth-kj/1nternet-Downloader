import { Navbar } from "@/components/navbar";
import { Hero } from "@/components/hero";
import { Features } from "@/components/features";
import { ProductScreenshot } from "@/components/product-screenshot";
import { WhySection } from "@/components/why-section";
import { DownloadSection } from "@/components/download-section";
import { OpenSourceSection } from "@/components/open-source-section";
import { Footer } from "@/components/footer";

export default function Home() {
  return (
    <div className="flex min-h-screen flex-col bg-background text-foreground transition-colors duration-300">
      <Navbar />
      <main className="flex-1">
        <Hero />
        <Features />
        <ProductScreenshot />
        <WhySection />
        <DownloadSection />
        <OpenSourceSection />
      </main>
      <Footer />
    </div>
  );
}
