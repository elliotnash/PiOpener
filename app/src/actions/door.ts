import { useSettingsStore } from "~/store/settings";

function postDoorRequest(path: string) {
  const { apiKey, apiUrl } = useSettingsStore.getState();

  fetch(`${apiUrl}/${path}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${apiKey}`,
    },
  });
}

export function openDoorAction() {
  postDoorRequest("open");
}
export function closeDoorAction() {
  postDoorRequest("close");
}
export function toggleDoorAction() {
  postDoorRequest("toggle");
}
