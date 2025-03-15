import { useState, useRef, Suspense, useEffect, useCallback } from "react";
import { Canvas } from "@react-three/fiber/native";
import { useSpring, animated } from "@react-spring/three";
import { Ionicons } from "@expo/vector-icons";
import {
  TouchableOpacity,
  Text,
  View,
  Dimensions,
  Platform,
  SafeAreaView,
} from "react-native";
import { useGLTF } from "@react-three/drei";
import type { GLTF } from "three-stdlib";
import type * as THREE from "three";
import { GestureDetector, Gesture } from "react-native-gesture-handler";
import { useColorScheme } from "~/lib/useColorScheme";
import { Link, Stack } from "expo-router";
import { useTheme } from "@react-navigation/native";
import { useStatus } from "~/store/status";
import {
  closeDoorAction,
  openDoorAction,
  toggleDoorAction,
} from "~/actions/door";

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
        : require("../../assets/panel.glb"),
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
  const { isDarkColorScheme } = useColorScheme();
  const lineColor = isDarkColorScheme ? "#fff" : "#000";

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
                  <lineBasicMaterial color={lineColor} linewidth={1} />
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

const Scene = ({
  setpointProgress,
  openProgress,
}: { setpointProgress: number; openProgress: number }) => {
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

      <AnimatedGarageDoor openProgress={openProgress} />
      <AnimatedGarageDoorFrame openProgress={setpointProgress} />
    </>
  );
};

export default function IndexPage() {
  const [setpointProgress, setSetpointProgress] = useState(0);
  const startYRef = useRef(0);
  const startProgressRef = useRef(0);
  const isGesturing = useRef(false);
  const { height } = Dimensions.get("window");

  const { status } = useStatus();

  useEffect(() => {
    console.log("Set point changed!", status?.setpoint);
    if (!isGesturing.current) {
      if (status?.setpoint === "open") {
        setSetpointProgress(1);
      } else if (status?.setpoint === "closed") {
        setSetpointProgress(0);
      } else if (status?.setpoint === "ajar") {
        console.log("Setting it to ", status?.position);
        setSetpointProgress(status?.position ?? 0.5);
      }
    }
  }, [status?.setpoint]);

  const openDoor = useCallback(() => {
    openDoorAction();
    setSetpointProgress(1);
  }, []);

  const closeDoor = useCallback(() => {
    closeDoorAction();
    setSetpointProgress(0);
  }, []);

  const toggleDoor = useCallback(() => {
    toggleDoorAction();
    setSetpointProgress(status?.position ?? 0.5);
  }, []);

  // Define drag gesture
  const dragGesture = Gesture.Pan()
    .runOnJS(true)
    .minDistance(0)
    .onBegin((event) => {
      startYRef.current = event.absoluteY;
      // Store the current progress when beginning the drag
      startProgressRef.current = setpointProgress;
      isGesturing.current = true;
    })
    .onUpdate((event) => {
      // Calculate how far the finger has moved as a percentage of screen height
      const dragDistance = startYRef.current - event.absoluteY;
      // Calculate new progress based on starting progress and drag distance
      const newProgress = Math.max(
        0,
        Math.min(1, startProgressRef.current + dragDistance / (height * 0.3)),
      );
      setSetpointProgress(newProgress);
    })
    .onEnd((event) => {
      // Calculate how far the finger has moved as a percentage of screen height
      const dragDistance = startYRef.current - event.absoluteY;
      // Calculate new progress based on starting progress and drag distance
      const newProgress = Math.max(
        0,
        Math.min(1, startProgressRef.current + dragDistance / (height * 0.3)),
      );

      // Snap to closest complete state
      const distToZero = newProgress;
      const distToOne = 1 - newProgress;
      const distToSetpoint = Math.abs(startProgressRef.current - newProgress);

      const minDist = Math.min(distToZero, distToOne, distToSetpoint);

      if (minDist === distToZero) {
        closeDoor();
      } else if (minDist === distToOne) {
        openDoor();
      } else {
        setSetpointProgress(startProgressRef.current);
      }

      isGesturing.current = false;
    });

  const tapGesture = Gesture.Tap()
    .runOnJS(true)
    .maxDuration(250)
    .maxDistance(10)
    .requireExternalGestureToFail(dragGesture)
    .onStart(() => {
      console.log("Tapped!!!");
      isGesturing.current = false;
      toggleDoor();
    });

  const combinedGesture = Gesture.Exclusive(dragGesture, tapGesture);

  const theme = useTheme();

  return (
    <>
      <Stack.Screen
        options={{
          headerTitle: "Garage",
          headerShadowVisible: false,
          headerRight: () => (
            <Link href="/settings" asChild>
              <TouchableOpacity
                className={`bg-foreground/10 p-2 rounded-full ${Platform.OS === "web" ? "mr-4" : ""}`}
              >
                <Ionicons
                  name="settings-outline"
                  size={24}
                  color={theme.colors.text}
                />
              </TouchableOpacity>
            </Link>
          ),
        }}
      />
      <GestureDetector gesture={combinedGesture}>
        <View className="flex-1">
          <Canvas
            camera={{ position: [1, 2, 8], fov: 50 }}
            className="flex-1"
            pointerEvents="none"
          >
            <Suspense>
              <Scene
                openProgress={status?.position ?? 0}
                setpointProgress={setpointProgress}
              />
            </Suspense>
          </Canvas>

          <SafeAreaView>
            <TouchableOpacity
              className="m-4 p-4 self-center rounded-full bg-blue-500"
              onPress={() => {
                if (setpointProgress > 0.5) {
                  closeDoor();
                } else {
                  openDoor();
                }
              }}
            >
              <Text className="text-white font-bold">
                {setpointProgress > 0.5 ? "Close Door" : "Open Door"}
              </Text>
            </TouchableOpacity>
          </SafeAreaView>
        </View>
      </GestureDetector>
    </>
  );
}
