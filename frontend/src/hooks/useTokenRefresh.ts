import { useEffect, useRef } from 'react';
import { useAuthStore } from '../stores/authStore';

const TOKEN_REFRESH_INTERVAL = 50 * 60 * 1000; // 50 minutes (tokens expire in 60 minutes)

export const useTokenRefresh = () => {
  const { isAuthenticated, refreshToken, logout } = useAuthStore();
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (!isAuthenticated) {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      return;
    }

    // Set up automatic token refresh
    intervalRef.current = setInterval(async () => {
      try {
        await refreshToken();
      } catch (error) {
        console.error('Token refresh failed:', error);
        await logout();
      }
    }, TOKEN_REFRESH_INTERVAL);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [isAuthenticated, refreshToken, logout]);
};
