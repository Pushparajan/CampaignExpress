/**
 * Campaign Express Web App Blueprint - App Root
 *
 * Sets up React Query provider and initializes the CE tracker.
 */

import type { AppProps } from "next/app";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { tracker } from "@/lib/ce-tracker";
import "@/styles/globals.css";

export default function App({ Component, pageProps }: AppProps) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: { retry: 2, refetchOnWindowFocus: false },
        },
      })
  );

  useEffect(() => {
    tracker.init();
    return () => tracker.destroy();
  }, []);

  return (
    <QueryClientProvider client={queryClient}>
      <Component {...pageProps} />
    </QueryClientProvider>
  );
}
