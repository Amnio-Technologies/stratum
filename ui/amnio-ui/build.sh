# Set build type (Default: Debug)
if [[ "$1" == "--release" ]]; then
    BUILD_TYPE="release"
    CMAKE_BUILD_TYPE="Release"
else
    BUILD_TYPE="debug"
    CMAKE_BUILD_TYPE="Debug"
fi

echo "Building amnio-ui as a STATIC LIBRARY ($BUILD_TYPE)..."

# Remove old build directory
rm -rf build && mkdir build && cd build || { echo "❌ Failed to create build directory"; exit 1; }

# Run CMake
cmake -G "MinGW Makefiles" -DCMAKE_BUILD_TYPE=$CMAKE_BUILD_TYPE .. || { echo "❌ CMake configuration failed"; exit 1; }

# Build amnio-ui as a STATIC library (.a/.lib)
cmake --build . || { echo "❌ Build failed"; exit 1; }

echo "✅ Build Complete!"
