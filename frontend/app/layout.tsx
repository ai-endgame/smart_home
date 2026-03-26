import type { Metadata } from 'next';
import { Space_Grotesk, Sora } from 'next/font/google';
import { Nav } from '@/components/layout/nav';
import { NotificationProvider } from '@/lib/context/notification-context';
import { NotificationDrawer } from '@/components/layout/notification-drawer';
import { AuthGate } from '@/components/layout/auth-gate';
import { ThemeApplier } from '@/components/layout/theme-applier';
import Script from 'next/script';
import './globals.css';

const spaceGrotesk = Space_Grotesk({
  subsets: ['latin'],
  variable: '--font-space-grotesk',
});

const sora = Sora({
  subsets: ['latin'],
  variable: '--font-sora',
});

export const metadata: Metadata = {
  title: 'Smart Home',
  description: 'Smart Home Dashboard',
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <link rel="manifest" href="/manifest.json" />
        <meta name="theme-color" content="#6366f1" />
      </head>
      <body className={`${spaceGrotesk.variable} ${sora.variable}`}>
        <ThemeApplier />
        <NotificationProvider>
          <AuthGate>
            <div className="app-background" aria-hidden="true" />
            <Nav />
            <main className="relative z-10 mx-auto w-full max-w-7xl px-4 pb-10 pt-6 sm:px-6 lg:px-8">
              {children}
            </main>
            <NotificationDrawer />
          </AuthGate>
        </NotificationProvider>
        <Script
          id="sw-register"
          strategy="afterInteractive"
          dangerouslySetInnerHTML={{
            __html: `if ('serviceWorker' in navigator) { navigator.serviceWorker.register('/sw.js'); }`,
          }}
        />
      </body>
    </html>
  );
}
