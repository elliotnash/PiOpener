const { getDefaultConfig } = require("expo/metro-config");
const { withNativeWind } = require("nativewind/metro");

const config = {
  ...getDefaultConfig(__dirname),
  resolver: {
    sourceExts: ["js", "jsx", "json", "ts", "tsx", "cjs", "mjs"],
    assetExts: ["glb", "gltf", "png", "jpg", "obj"],
  },
};

module.exports = withNativeWind(config, { input: "./src/global.css" });
