import "./global.css";
import { GestureHandlerRootView } from "react-native-gesture-handler";
import Garage from "./garage";

export default function App() {
  return (
    <GestureHandlerRootView>
      <Garage />
    </GestureHandlerRootView>
  );
}
