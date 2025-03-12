import "./global.css";

import { useState } from "react";
import { Canvas } from "@react-three/fiber/native";
import { useSpring, animated } from "@react-spring/three";
import { TouchableOpacity, Text, View, StyleSheet } from "react-native";

const AnimatedGarageDoor = ({ isOpen }: { isOpen: boolean }) => {
  const { rotation, position } = useSpring({
    rotation: isOpen ? [-Math.PI / 2, 0, 0] : [0, 0, 0],
    position: isOpen ? [0, 1, -1] : [0, 0, 0],
    config: { mass: 1, tension: 25, friction: 8 },
  });

  return (
    // biome-ignore lint/suspicious/noExplicitAny: Vector3
    <animated.mesh rotation={rotation as any} position={position as any}>
      <boxGeometry args={[2, 2, 0.1]} />
      <meshStandardMaterial color="gray" />
      <mesh position={[0.8, 0, 0.06]}>
        <cylinderGeometry args={[0.03, 0.03, 0.2]} />
        <meshStandardMaterial color="darkgray" />
      </mesh>
    </animated.mesh>
  );
};

const Scene = ({ isOpen }: { isOpen: boolean }) => {
  return (
    <>
      <ambientLight intensity={0.75} />
      <pointLight position={[10, 5, 5]} intensity={1000} castShadow />

      {/* Garage structure */}
      <mesh position={[0, -1.1, 0]}>
        <boxGeometry args={[2.5, 0.2, 2]} />
        {/* Increase material roughness for better light response */}
        <meshStandardMaterial color="#555" roughness={0.4} metalness={0.2} />
      </mesh>

      <AnimatedGarageDoor isOpen={isOpen} />
    </>
  );
};

export default function App() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <View className="flex-1 bg-black">
      <Canvas camera={{ position: [1, 2, 8], fov: 50 }} className="flex-1">
        <Scene isOpen={isOpen} />
      </Canvas>

      <TouchableOpacity
        className="absolute bottom-10 p-4 self-center rounded-full bg-blue-500"
        onPress={() => setIsOpen(!isOpen)}
      >
        <Text className="text-white font-bold">
          {isOpen ? "Close Door" : "Open Door"}
        </Text>
      </TouchableOpacity>
    </View>
  );
}
