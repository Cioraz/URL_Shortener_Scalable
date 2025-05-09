"use client";

import { ReactNode, useEffect, useState } from "react";

export function Providers({ children }: { children: ReactNode }) {
  const [mounted, setMounted] = useState(false);

  // Ensure theme is applied only on the client side
  useEffect(() => {
    setMounted(true);
  }, []);

  // Return a blank div until mounted to prevent hydration mismatch
  if (!mounted) {
    return <div style={{ visibility: "hidden" }}>{children}</div>;
  }

  return <>{children}</>;
}