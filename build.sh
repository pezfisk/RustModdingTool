#!/bin/bash

IMAGE_FILE="notfound.png"

if [ "$1" == "release" ]; then
  COMPILE_MODE="--release"
  echo "Compiling in release mode..."
else
  COMPILE_MODE=""
  echo "Compiling in debug mode..."
fi

cargo build $COMPILE_MODE

if [ $? -eq 0 ]; then
  echo "Compilation successful."

  if [ "$1" == "release" ]; then
    TARGET_DIR="target/release"
  else
    TARGET_DIR="target/debug"
  fi

  if [ -d "$TARGET_DIR" ]; then
    cp "$IMAGE_FILE" "$TARGET_DIR/"
    if [ $? -eq 0 ]; then
      echo "Successfully copied $IMAGE_FILE to $TARGET_DIR"
    else
      echo "Error: Failed to copy $IMAGE_FILE to $TARGET_DIR"
    fi
  else
    echo "Error: Target directory $TARGET_DIR not found after compilation."
  fi
else
  echo "Error: Rust compilation failed."
fi
