/**
 * React hooks for Campaign Express loyalty program integration.
 */

"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";
import type { LoyaltyEarnRequest, LoyaltyRedeemRequest } from "@/lib/types";

export function useLoyaltyBalance(userId: string) {
  return useQuery({
    queryKey: ["loyalty", "balance", userId],
    queryFn: () => apiClient.getLoyaltyBalance(userId),
    enabled: !!userId,
    staleTime: 60_000,
  });
}

export function useEarnStars() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: LoyaltyEarnRequest) => apiClient.earnStars(data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: ["loyalty", "balance", variables.user_id],
      });
    },
  });
}

export function useRedeemStars() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: LoyaltyRedeemRequest) => apiClient.redeemStars(data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: ["loyalty", "balance", variables.user_id],
      });
    },
  });
}
