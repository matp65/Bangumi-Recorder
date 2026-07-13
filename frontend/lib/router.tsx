"use client";

import { useSyncExternalStore } from "react";

const ROUTE_EVENT = "bangumi-recorder:navigate";

function subscribe(listener: () => void) {
  window.addEventListener("popstate", listener);
  window.addEventListener(ROUTE_EVENT, listener);
  return () => {
    window.removeEventListener("popstate", listener);
    window.removeEventListener(ROUTE_EVENT, listener);
  };
}

function snapshot() {
  return `${window.location.pathname}${window.location.search}`;
}

export function useCurrentRoute() {
  return useSyncExternalStore(subscribe, snapshot, () => "/");
}

export function navigate(href: string, options?: { replace?: boolean }) {
  if (typeof window === "undefined") return;
  const current = `${window.location.pathname}${window.location.search}`;
  if (current === href) return;
  if (options?.replace) window.history.replaceState({}, "", href);
  else window.history.pushState({}, "", href);
  window.dispatchEvent(new Event(ROUTE_EVENT));
  window.scrollTo({ top: 0, behavior: "auto" });
}

export function AppLink({ href, onClick, ...props }: React.ComponentProps<"a"> & { href: string }) {
  return <a href={href} onClick={(event) => {
    onClick?.(event);
    if (event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
    event.preventDefault();
    navigate(href);
  }} {...props} />;
}
