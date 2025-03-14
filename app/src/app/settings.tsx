import { Stack } from "expo-router";
import { View, Text } from "react-native";
import { TextInput } from "react-native-gesture-handler";

export default function SettingsPage() {
  return (
    <>
      <Stack.Screen
        options={{
          headerTitle: "Settings",
          headerShadowVisible: false,
        }}
      />
      <InputSetting name="API URL" />
      <InputSetting name="API Key" />
    </>
  );
}

function InputSetting({ name }: { name: string }) {
  return (
    <View className="mx-4 mt-4">
      <Text className="text-foreground ml-4 mb-1">{name}</Text>
      <View className="bg-foreground/5 rounded-md border border-border">
        <TextInput className="text-foreground p-3 outline-none" />
      </View>
    </View>
  );
}
