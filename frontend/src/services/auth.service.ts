import { apiClient, apiService } from './api';
import type { LoginRequest, AuthResponse } from '../types';

export const authService = {
  async login(credentials: LoginRequest): Promise<AuthResponse> {
    const response = await apiClient.post<AuthResponse>('/auth/login', credentials);
    const { token, user } = response.data;
    
    apiService.setAuthToken(token);
    localStorage.setItem('user', JSON.stringify(user));
    
    return response.data;
  },

  async logout(): Promise<void> {
    try {
      await apiClient.post('/auth/logout');
    } finally {
      apiService.clearAuthToken();
    }
  },

  async refreshToken(): Promise<AuthResponse> {
    const response = await apiClient.post<AuthResponse>('/auth/refresh');
    const { token } = response.data;
    
    apiService.setAuthToken(token);
    
    return response.data;
  },

  getCurrentUser() {
    const userStr = localStorage.getItem('user');
    return userStr ? JSON.parse(userStr) : null;
  },

  isAuthenticated(): boolean {
    return !!localStorage.getItem('auth_token');
  },
};
