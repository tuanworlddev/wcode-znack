// App update flow via tauri-plugin-updater + GitHub Releases (latest.json).
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export interface UpdateInfo {
  version: string;
  notes: string;
  date: string;
}

/** Returns the pending update, or null when already up to date. */
export async function checkForUpdate(): Promise<Update | null> {
  return await check();
}

export function updateInfo(u: Update): UpdateInfo {
  return {
    version: u.version,
    notes: u.body ?? "",
    date: u.date ?? "",
  };
}

/** Download + install the update, reporting progress in [0..1], then relaunch. */
export async function installUpdate(
  u: Update,
  onProgress: (fraction: number) => void
): Promise<void> {
  let total = 0;
  let received = 0;
  await u.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        total = event.data.contentLength ?? 0;
        break;
      case "Progress":
        received += event.data.chunkLength;
        if (total > 0) onProgress(Math.min(received / total, 1));
        break;
      case "Finished":
        onProgress(1);
        break;
    }
  });
  await relaunch();
}
