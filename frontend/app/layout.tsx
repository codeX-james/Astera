import type { Metadata } from 'next';
import './globals.css';
import Navbar from '@/components/Navbar';

export const metadata: Metadata = {
  title: 'Astera — Real World Assets on Stellar',
  description:
    'Tokenize invoices. Fund real businesses. Earn on-chain yield. Built on Stellar Soroban.',
  openGraph: {
    title: 'Astera',
    description: 'Invoice financing for emerging markets, powered by Stellar.',
    siteName: 'Astera',
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap"
          rel="stylesheet"
        />
      </head>
      <body className="min-h-screen bg-brand-dark text-white">
        <Navbar />
        <main>{children}</main>
      </body>
    </html>
  );
}
