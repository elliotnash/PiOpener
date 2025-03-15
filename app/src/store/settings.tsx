import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import { expoSecureStorage } from "~/lib/storage";

interface SettingsState {
  apiUrl: string;
  setApiUrl: (apiUrl: string) => void;
  apiKey: string;
  setApiKey: (apiKey: string) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      apiUrl: "",
      setApiUrl: (apiUrl: string) => set({ apiUrl }),
      apiKey: "",
      setApiKey: (apiKey: string) => set({ apiKey }),
    }),
    {
      name: "settings-storage",
      storage: createJSONStorage(() => expoSecureStorage),
    },
  ),
);
