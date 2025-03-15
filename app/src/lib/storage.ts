import { setItemAsync, getItemAsync, deleteItemAsync } from "expo-secure-store";
import type { StateStorage } from "zustand/middleware";
import AsyncStorage from "@react-native-async-storage/async-storage";
import { Platform } from "react-native";

export const expoSecureStorage: StateStorage =
  Platform.OS === "web"
    ? AsyncStorage
    : {
        setItem: async (key: string, value: string) =>
          await setItemAsync(key, value),
        getItem: async (key: string) =>
          (await getItemAsync(key)) as Promise<string> | null,
        removeItem: async (key: string) => await deleteItemAsync(key),
      };
