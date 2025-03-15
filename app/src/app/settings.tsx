import { Stack } from "expo-router";
import { View, Text } from "react-native";
import { TextInput } from "react-native-gesture-handler";
import { useSettingsStore } from "~/store/settings";

export default function SettingsPage() {
  const settings = useSettingsStore();

  return (
    <>
      <Stack.Screen
        options={{
          headerTitle: "Settings",
          headerShadowVisible: false,
        }}
      />
      <InputSetting
        name="API URL"
        value={settings.apiUrl}
        onChangeText={settings.setApiUrl}
      />
      <InputSetting
        name="API Key"
        value={settings.apiKey}
        onChangeText={settings.setApiKey}
      />
    </>
  );
}

function InputSetting({
  value,
  name,
  onChangeText,
}: { value: string; name: string; onChangeText: (value: string) => void }) {
  return (
    <View className="mx-4 mt-4">
      <Text className="text-foreground ml-4 mb-1">{name}</Text>
      <View className="rounded-lg border border-border">
        <TextInput
          value={value}
          onChangeText={(text) => onChangeText(text)}
          className="text-foreground p-3 outline-none"
        />
      </View>
    </View>
  );
}
