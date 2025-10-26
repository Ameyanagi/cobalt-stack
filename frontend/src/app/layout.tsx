import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "@/styles/base.css";
import "@/styles/themes/default.css";
import "@/styles/themes/nature.css";
import "@/styles/themes/violet-bloom.css";
import { Providers } from "./providers";
import { ThemeProvider } from "@/contexts/theme-context";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Cobalt Stack",
  description: "Full-stack application with Rust backend and Next.js frontend",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <ThemeProvider>
          <Providers>{children}</Providers>
        </ThemeProvider>
      </body>
    </html>
  );
}
