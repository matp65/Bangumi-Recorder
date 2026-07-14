"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { api } from "@/lib/api/client";
import type { UserInfo } from "@/lib/api/types";
import {
  clearStoredSession,
  getStoredToken,
  getStoredUsername,
  isTokenExpired,
  storeSession,
} from "@/lib/auth-storage";

interface AuthContextValue {
  hydrated: boolean;
  token: string | null;
  username: string | null;
  user: UserInfo | null;
  login: (
    username: string,
    password: string,
  ) => Promise<{ ok: boolean; message?: string }>;
  register: (
    username: string,
    password: string,
    registerToken?: string,
  ) => Promise<{ ok: boolean; message?: string }>;
  logout: () => void;
  refreshUser: () => Promise<void>;
  updateLocalUser: (patch: Partial<UserInfo>) => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [hydrated, setHydrated] = useState(false);
  const [token, setToken] = useState<string | null>(null);
  const [username, setUsername] = useState<string | null>(null);
  const [user, setUser] = useState<UserInfo | null>(null);

  const logout = useCallback(() => {
    clearStoredSession();
    setToken(null);
    setUsername(null);
    setUser(null);
  }, []);

  const refreshUser = useCallback(async () => {
    if (!getStoredToken()) return;
    const response = await api.getUserInfo();
    if (response.status === 0 && response.data?.id) setUser(response.data);
  }, []);

  useEffect(() => {
    queueMicrotask(() => {
      const storedToken = getStoredToken();
      if (storedToken && !isTokenExpired(storedToken)) {
        setToken(storedToken);
        setUsername(getStoredUsername());
      } else if (storedToken) clearStoredSession();
      setHydrated(true);
    });
  }, []);

  useEffect(() => {
    if (!token) return;
    queueMicrotask(() => void refreshUser());
  }, [token, refreshUser]);

  useEffect(() => {
    window.addEventListener("bangumi-recorder:session-expired", logout);
    return () =>
      window.removeEventListener("bangumi-recorder:session-expired", logout);
  }, [logout]);

  const login = useCallback(async (name: string, password: string) => {
    const response = await api.login(name, password);
    if (response.status !== 0 || !response.data?.token)
      return { ok: false, message: response.message };
    storeSession(response.data.token, name);
    setToken(response.data.token);
    setUsername(name);
    return { ok: true };
  }, []);

  const register = useCallback(
    async (name: string, password: string, registerToken?: string) => {
      const response = await api.register(name, password, registerToken);
      if (response.status !== 0 || !response.data?.token)
        return { ok: false, message: response.message };
      storeSession(response.data.token, name);
      setToken(response.data.token);
      setUsername(name);
      return { ok: true };
    },
    [],
  );

  const value = useMemo<AuthContextValue>(
    () => ({
      hydrated,
      token,
      username,
      user,
      login,
      register,
      logout,
      refreshUser,
      updateLocalUser: (patch) =>
        setUser((current) => (current ? { ...current, ...patch } : current)),
    }),
    [hydrated, token, username, user, login, register, logout, refreshUser],
  );
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) throw new Error("useAuth must be used inside AuthProvider");
  return context;
}
