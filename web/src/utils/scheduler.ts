/**
 * Cooperative multitasking helper for CPU-bound tasks in the browser.
 *
 * Yields execution back to the browser event loop, allowing:
 * - Real-time DOM paints and status updates
 * - Smooth 60 FPS CSS spinner animation
 * - Pointer and keyboard input processing
 * - Resetting the browser's "unresponsive script / page hanging" watchdog timer
 */
interface GlobalWithScheduler {
  scheduler?: {
    yield?: () => Promise<void>;
  };
}

export async function yieldToMain(): Promise<void> {
  const globalWithScheduler = globalThis as unknown as GlobalWithScheduler;
  // If modern scheduler.yield is available (Chromium 129+), use it
  if (typeof globalWithScheduler.scheduler?.yield === 'function') {
    try {
      await globalWithScheduler.scheduler.yield();
      return;
    } catch {
      // Fall through to setTimeout fallback
    }
  }

  // Standard macrotask yield via setTimeout(..., 0)
  return new Promise<void>((resolve) => {
    setTimeout(resolve, 0);
  });
}
