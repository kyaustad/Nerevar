"use client";

import { ThemeProvider } from "next-themes";
import { Toaster } from "@/components/ui/sonner";
import { ConfigProvider } from "@/features/app-config/context/config-context";
import { OpenMWConfigProvider } from "@/features/app-config/context/openmw-config-context";
import {
  useQuery,
  useMutation,
  useQueryClient,
  QueryClient,
  QueryClientProvider,
} from "@tanstack/react-query";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: true,
      staleTime: 1000 * 60 * 5, // 5 minutes
      gcTime: 1000 * 60 * 10, // 10 minutes
      retry: 3,
      retryDelay: 1000,
    },
  },
});

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="dark">
      <QueryClientProvider client={queryClient}>
        <ConfigProvider>
          <OpenMWConfigProvider>{children}</OpenMWConfigProvider>

          <Toaster richColors />
        </ConfigProvider>
      </QueryClientProvider>
    </ThemeProvider>
  );
}
