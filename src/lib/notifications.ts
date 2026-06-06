export async function notify(title: string, body: string): Promise<void> {
  if (typeof window === "undefined" || !("Notification" in window)) return;
  if (Notification.permission === "granted") {
    try {
      new Notification(title, { body, silent: false });
    } catch {
      /* ignore */
    }
    return;
  }
  if (Notification.permission !== "denied") {
    try {
      const res = await Notification.requestPermission();
      if (res === "granted") new Notification(title, { body, silent: false });
    } catch {
      /* ignore */
    }
  }
}

export function requestNotificationPermission(): void {
  if (typeof window !== "undefined" && "Notification" in window) {
    if (Notification.permission === "default") void Notification.requestPermission();
  }
}
