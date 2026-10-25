export function reducedMotion(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function motionDuration(ms = 150): number {
  return reducedMotion() ? 0 : ms;
}
