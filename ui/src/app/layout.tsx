"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import Layout from "@/components/layout";
import ErrorBoundary from "@/components/error-boundary";
import "./globals.css";

const FONT_CLASS =
  "font-sans antialiased";

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 30_000,
            retry: 1,
            refetchOnWindowFocus: false,
          },
        },
      })
  );

  return (
    <html lang="en" className="dark">
      <head>
        <title>Campaign Express</title>
        <meta
          name="description"
          content="Campaign Express Management Dashboard"
        />
      </head>
      <body className={FONT_CLASS}>
        <QueryClientProvider client={queryClient}>
          <ErrorBoundary>
            <Layout>{children}</Layout>
          </ErrorBoundary>
        </QueryClientProvider>
      </body>
    </html>
  );
}
