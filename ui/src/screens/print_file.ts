import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { TEXT } from "../labels";
import { currentScreen } from "../router";

export async function choosePrintFile(): Promise<string | null> {
  const picked = await open({ multiple: false, title: TEXT.chooseFile });
  return typeof picked === "string" ? picked : null;
}

export function registerFileDrop(
  dropzone: HTMLElement,
  onFile: (path: string) => void,
): void {
  getCurrentWebview()
    .onDragDropEvent((event) => {
      if (currentScreen() !== "print") {
        return;
      }
      const payload = event.payload;
      if (payload.type === "drop") {
        const path = payload.paths[0];
        if (path !== undefined) {
          onFile(path);
        }
        dropzone.removeAttribute("data-dragging");
      } else if (payload.type === "enter" || payload.type === "over") {
        dropzone.setAttribute("data-dragging", "");
      } else {
        dropzone.removeAttribute("data-dragging");
      }
    })
    .catch(() => {});
}
