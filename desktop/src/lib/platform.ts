import { platform } from "@tauri-apps/plugin-os";

/**
 * Returns true when running inside the Android build of the app.
 *
 * Decisions about mobile-specific behaviour MUST go through this helper
 * instead of sniffing screen size, so the desktop layout is never
 * affected. `platform()` is a compile-time constant on the Tauri side,
 * so the check is deterministic per build.
 */
export function isAndroid(): boolean {
  return platform() === "android";
}

/**
 * Returns true when running on any mobile target (currently Android).
 * Use this when a behaviour applies to every mobile platform; prefer
 * the narrower {@link isAndroid} when a path is Android-specific.
 */
export function isMobile(): boolean {
  return isAndroid();
}
