import type { Metadata } from "next";
import "./globals.css";
export const metadata: Metadata = { title: "Orbit — Storymesh demo", description: "Next.js + Storybook + Storymesh の実動サンプル" };
export default function RootLayout({ children }: { children: React.ReactNode }) {
 return <html lang="ja"><body>{children}</body></html>;
}
