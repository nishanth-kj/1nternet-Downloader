const prefix = process.env.NODE_ENV === "production" ? "/1nternet-Downloader" : "";

export function Logo({ size = 32, className = "" }: { size?: number; className?: string }) {
  return (
    <img
      src={`${prefix}/logo.svg`}
      alt="1IDM"
      width={size}
      height={size}
      className={`inline-block bg-transparent ${className}`.trim()}
      style={{ backgroundColor: "transparent" }}
    />
  );
}
