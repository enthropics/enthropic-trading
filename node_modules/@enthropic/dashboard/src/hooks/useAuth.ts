import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import axios from 'axios';

const API_URL = import.meta.env.VITE_API_URL || '/api';

export interface User {
    id: string;
    username: string;
    email: string;
    role: string;
    permissions: string[];
}

interface AuthState {
    user: User | null;
    accessToken: string | null;
    refreshToken: string | null;
    expiresAt: Date | null;
    isAuthenticated: boolean;
    login: (username: string, password: string) => Promise<void>;
    logout: () => void;
    refresh: () => Promise<void>;
}

export const useAuth = create<AuthState>()(
    persist(
        (set, get) => ({
            user: null,
            accessToken: null,
            refreshToken: null,
            expiresAt: null,
            isAuthenticated: false,

            login: async (username: string, password: string) => {
                try {
                    const response = await axios.post(`${API_URL}/auth/login`, {
                        username,
                        password,
                    });

                    const { accessToken, refreshToken, expiresAt, user } = response.data;

                    set({
                        user,
                        accessToken,
                        refreshToken,
                        expiresAt: expiresAt ? new Date(expiresAt) : null,
                        isAuthenticated: true,
                    });

                    // Set default auth header
                    axios.defaults.headers.common['Authorization'] = `Bearer ${accessToken}`;
                } catch (error) {
                    console.error('Login failed:', error);
                    throw error;
                }
            },

            logout: () => {
                const { accessToken } = get();

                // Call logout endpoint (fire and forget)
                if (accessToken) {
                    axios.post(`${API_URL}/auth/logout`, {}, {
                        headers: { Authorization: `Bearer ${accessToken}` }
                    }).catch((error) => {
                        console.error('Logout API call failed:', error);
                    });
                }

                // Clear state
                set({
                    user: null,
                    accessToken: null,
                    refreshToken: null,
                    expiresAt: null,
                    isAuthenticated: false,
                });

                // Remove auth header
                delete axios.defaults.headers.common['Authorization'];
            },

            refresh: async () => {
                const { refreshToken } = get();

                if (!refreshToken) {
                    throw new Error('No refresh token available');
                }

                try {
                    const response = await axios.post(`${API_URL}/auth/refresh`, {
                        refreshToken,
                    });

                    const { accessToken, expiresAt } = response.data;

                    set({
                        accessToken,
                        expiresAt: expiresAt ? new Date(expiresAt) : null
                    });

                    axios.defaults.headers.common['Authorization'] = `Bearer ${accessToken}`;
                } catch (error) {
                    console.error('Token refresh failed:', error);
                    // Logout on refresh failure
                    get().logout();
                    throw error;
                }
            },
        }),
        {
            name: 'enthropic-auth',
            partialize: (state) => ({
                user: state.user,
                accessToken: state.accessToken,
                refreshToken: state.refreshToken,
                expiresAt: state.expiresAt,
                isAuthenticated: state.isAuthenticated,
            }),
        }
    )
);

// Setup axios interceptor for token refresh
axios.interceptors.response.use(
    (response) => response,
    async (error) => {
        const originalRequest = error.config;

        // Avoid infinite loop
        if (originalRequest._retry) {
            return Promise.reject(error);
        }

        // Handle 401 Unauthorized
        if (error.response?.status === 401 && !originalRequest._retry) {
            originalRequest._retry = true;

            const state = useAuth.getState();

            if (state.refreshToken) {
                try {
                    await state.refresh();

                    // Retry original request with new token
                    originalRequest.headers['Authorization'] = `Bearer ${state.accessToken}`;
                    return axios(originalRequest);
                } catch (refreshError) {
                    // Refresh failed, logout user
                    state.logout();
                    return Promise.reject(refreshError);
                }
            } else {
                // No refresh token, logout
                state.logout();
            }
        }

        return Promise.reject(error);
    }
);