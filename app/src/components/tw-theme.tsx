import { useColorScheme } from "nativewind";
import type React from "react";
import { View } from "react-native";
import { themes } from "../contants/colors";

export function TwThemeProvider({ children }: React.PropsWithChildren) {
  const { colorScheme } = useColorScheme();

  return (
    <View
      className={colorScheme === "dark" ? "flex-1 dark" : "flex-1"}
      style={themes[colorScheme ?? "dark"]}
    >
      {children}
    </View>
  );
}
