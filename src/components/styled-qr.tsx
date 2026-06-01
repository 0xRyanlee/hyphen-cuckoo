import { useEffect, useRef } from "react";
import QRCodeStyling from "qr-code-styling";

interface StyledQRProps {
  value: string;
  size?: number;
  dotColor?: string;
  logo?: string | null;
}

function buildOptions(value: string, size: number, dotColor: string, logo?: string | null) {
  return {
    width: size,
    height: size,
    type: "canvas" as const,
    data: value,
    margin: 8,
    dotsOptions: { type: "rounded" as const, color: dotColor },
    cornersSquareOptions: { type: "extra-rounded" as const, color: dotColor },
    cornersDotOptions: { type: "dot" as const, color: dotColor },
    backgroundOptions: { color: "#ffffff" },
    qrOptions: { errorCorrectionLevel: (logo ? "H" : "M") as "H" | "M" },
    ...(logo
      ? { image: logo, imageOptions: { crossOrigin: "anonymous", margin: 6, imageSize: 0.3 } }
      : {}),
  };
}

/** WeChat/Alipay-style QR: rounded modules, extra-rounded eyes, optional center logo. */
export function StyledQR({ value, size = 220, dotColor = "#1f2937", logo }: StyledQRProps) {
  const ref = useRef<HTMLDivElement>(null);
  const qrRef = useRef<QRCodeStyling | null>(null);

  useEffect(() => {
    if (!qrRef.current) {
      qrRef.current = new QRCodeStyling(buildOptions(value, size, dotColor, logo));
      if (ref.current) {
        ref.current.innerHTML = "";
        qrRef.current.append(ref.current);
      }
    } else {
      qrRef.current.update(buildOptions(value, size, dotColor, logo));
    }
  }, [value, size, dotColor, logo]);

  return <div ref={ref} className="flex items-center justify-center" />;
}

/** Generates a high-DPI PNG of the styled QR (for print-shop stickers). */
export function downloadStyledQR(
  value: string,
  opts: { name: string; size?: number; dotColor?: string; logo?: string | null }
) {
  const size = opts.size ?? 1024; // print-grade
  const qr = new QRCodeStyling(buildOptions(value, size, opts.dotColor ?? "#1f2937", opts.logo));
  qr.download({ name: opts.name, extension: "png" });
}
