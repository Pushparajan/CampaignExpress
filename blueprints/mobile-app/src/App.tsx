/**
 * Campaign Express Mobile App Blueprint
 *
 * Main entry point. Initializes the CE SDK and renders the app navigator.
 */

import React, { useEffect, useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AppNavigator } from "./navigation/AppNavigator";
import { ceSdk } from "./services/ce-sdk";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: 2, refetchOnWindowFocus: false },
  },
});

export default function App() {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    async function init() {
      await ceSdk.initialize();
      ceSdk.trackAppOpen();
      setReady(true);
    }
    init();

    return () => {
      ceSdk.shutdown();
    };
  }, []);

  if (!ready) return null;

  return (
    <QueryClientProvider client={queryClient}>
      <AppNavigator />
    </QueryClientProvider>
  );
}
