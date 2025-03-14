import { useState, useRef, Suspense } from "react";
import { Canvas } from "@react-three/fiber/native";
import { useSpring, animated } from "@react-spring/three";
import {
  TouchableOpacity,
  Text,
  View,
  Dimensions,
  Platform,
} from "react-native";
import { useGLTF } from "@react-three/drei";
import type { GLTF } from "three-stdlib";
import type * as THREE from "three";
import {
  GestureDetector,
  Gesture,
  Directions,
} from "react-native-gesture-handler";

const panelPositions = [
  [0, -0.75, 0],
  [0, -0.25, 0],
  [0, 0.25, 0],
  [0, 0.75, 0],
];

const useDoorPanel = () => {
  return (
    useGLTF(
      Platform.OS === "web"
        ? "/assets/panel.glb"
        : require("../assets/panel.glb"),
    ) as GLTF
  ).scene as THREE.Group;
};

const AnimatedGarageDoor = ({ openProgress }: { openProgress: number }) => {
  const panel = useDoorPanel();

  // Calculate rotation and position based on openProgress (0 to 1)
  const { rotation, position } = useSpring({
    rotation: [(-Math.PI / 2) * openProgress, 0, 0],
    position: [0, openProgress, -openProgress],
    config: { mass: 1, tension: 250, friction: 26 },
  });

  panel.traverse((child) => {
    if (child.type === "Mesh") {
      // @ts-ignore
      child.material.color.set("#be9878");
    }
  });

  return (
    // biome-ignore lint/suspicious/noExplicitAny: Vector3
    <animated.mesh rotation={rotation as any} position={position as any}>
      <primitive
        object={panel.clone()}
        scale={0.5}
        position={panelPositions[0]}
      />
      <primitive
        object={panel.clone()}
        scale={0.5}
        position={panelPositions[1]}
      />
      <primitive
        object={panel.clone()}
        scale={0.5}
        position={panelPositions[2]}
      />
      <primitive
        object={panel.clone()}
        scale={0.5}
        position={panelPositions[3]}
      />
      <meshStandardMaterial color="grey" />
    </animated.mesh>
  );
};

const AnimatedGarageDoorFrame = ({
  openProgress,
}: { openProgress: number }) => {
  const panel = useDoorPanel();

  // Calculate rotation and position based on openProgress (0 to 1)
  const { rotation, position } = useSpring({
    rotation: [(-Math.PI / 2) * openProgress, 0, 0],
    position: [0, openProgress, -openProgress],
    config: { mass: 1, tension: 250, friction: 26 },
  });

  return (
    // biome-ignore lint/suspicious/noExplicitAny: Vector3
    <animated.group rotation={rotation as any} position={position as any}>
      {panelPositions.map((pos, index) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: <explanation>
        // biome-ignore lint/suspicious/noExplicitAny: <explanation>
        <group key={index} position={pos as any} scale={0.5}>
          {/* Extract all meshes from the panel and create edge geometries */}
          {panel.clone().children.map((child, childIndex) => {
            if (child.type === "Mesh") {
              return (
                <lineSegments
                  rotation={[Math.PI / 2, 0, 0]}
                  key={`${index}-${
                    // biome-ignore lint/suspicious/noArrayIndexKey: <explanation>
                    childIndex
                  }`}
                >
                  <edgesGeometry args={[(child as THREE.Mesh).geometry]} />
                  <lineBasicMaterial color="#fff" linewidth={1} />
                </lineSegments>
              );
            }
            return null;
          })}
        </group>
      ))}
    </animated.group>
  );
};

const Scene = ({ openProgress }: { openProgress: number }) => {
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

      <AnimatedGarageDoor openProgress={0} />
      <AnimatedGarageDoorFrame openProgress={openProgress} />
    </>
  );
};

export default function Garage() {
  const [openProgress, setOpenProgress] = useState(0);
  const startYRef = useRef(0);
  const startProgressRef = useRef(0);
  const { height } = Dimensions.get("window");

  // Define drag gesture
  const dragGesture = Gesture.Pan()
    .runOnJS(true)
    .minDistance(0)
    .onBegin((event) => {
      startYRef.current = event.absoluteY;
      // Store the current progress when beginning the drag
      startProgressRef.current = openProgress;
    })
    .onUpdate((event) => {
      // Calculate how far the finger has moved as a percentage of screen height
      const dragDistance = startYRef.current - event.absoluteY;
      // Calculate new progress based on starting progress and drag distance
      const newProgress = Math.max(
        0,
        Math.min(1, startProgressRef.current + dragDistance / (height * 0.3)),
      );
      setOpenProgress(newProgress);
    })
    .onEnd((event) => {
      // Calculate how far the finger has moved as a percentage of screen height
      const dragDistance = startYRef.current - event.absoluteY;
      // Calculate new progress based on starting progress and drag distance
      const newProgress = Math.max(
        0,
        Math.min(1, startProgressRef.current + dragDistance / (height * 0.3)),
      );
      // Snap to fully open or closed based on current progress
      if (newProgress > 0.5) {
        setOpenProgress(1);
      } else {
        setOpenProgress(0);
      }
    });

  return (
    <GestureDetector gesture={dragGesture}>
      <View className="flex-1 bg-black">
        <Canvas
          camera={{ position: [1, 2, 8], fov: 50 }}
          className="flex-1"
          pointerEvents="none"
        >
          <Suspense>
            <Scene openProgress={openProgress} />
          </Suspense>
        </Canvas>

        <TouchableOpacity
          className="absolute bottom-10 p-4 self-center rounded-full bg-blue-500"
          onPress={() => setOpenProgress(openProgress > 0 ? 0 : 1)}
        >
          <Text className="text-white font-bold">
            {openProgress > 0.5 ? "Close Door" : "Open Door"}
          </Text>
        </TouchableOpacity>
      </View>
    </GestureDetector>
  );
}
