import { useState } from "react";
import { Canvas } from "@react-three/fiber/native";
import { useSpring, animated } from "@react-spring/three";
import { TouchableOpacity, Text, View, StyleSheet } from "react-native";

const AnimatedGarageDoor = ({ isOpen }: { isOpen: boolean }) => {
  const { rotation } = useSpring({
    rotation: isOpen ? [-Math.PI / 2, 0, 0] : [0, 0, 0],
    config: { mass: 1, tension: 180, friction: 20 },
  });

  return (
    <animated.mesh rotation={rotation as any} position={[0, 0, 0]}>
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
      <mesh position={[0, -1, 0]}>
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
    <View style={styles.container}>
      <Canvas style={styles.canvas}>
        <Scene isOpen={isOpen} />
      </Canvas>

      <TouchableOpacity
        style={styles.button}
        onPress={() => setIsOpen(!isOpen)}
      >
        <Text style={styles.buttonText}>
          {isOpen ? "Close Door" : "Open Door"}
        </Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#000",
  },
  canvas: {
    flex: 1,
  },
  button: {
    position: "absolute",
    bottom: 40,
    alignSelf: "center",
    backgroundColor: "blue",
    padding: 15,
    borderRadius: 10,
  },
  buttonText: {
    color: "white",
    fontWeight: "bold",
  },
});
