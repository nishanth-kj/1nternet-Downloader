# Overlay triplet for building curl (+ its deps) for Android arm64-v8a, matching this
# app's minSdk (24) instead of vcpkg's community arm64-android-release triplet, which
# targets API 28. Building against a higher API than minSdk risks linking against libc
# symbols (e.g. fortified functions like __sendto_chk) that don't exist on older devices.
#
# Usage: vcpkg install curl:arm64-android --overlay-triplets=. (from this directory)
# app/build.gradle passes the matching -DVCPKG_OVERLAY_TRIPLETS/-DVCPKG_TARGET_TRIPLET
# arguments so Gradle's build picks up packages installed with this triplet.

set(VCPKG_TARGET_ARCHITECTURE arm64)
set(VCPKG_CRT_LINKAGE dynamic)
set(VCPKG_LIBRARY_LINKAGE static)
set(VCPKG_CMAKE_SYSTEM_NAME Android)
set(VCPKG_CMAKE_SYSTEM_VERSION 24)
set(VCPKG_MAKE_BUILD_TRIPLET "--host=aarch64-linux-android")
set(VCPKG_CMAKE_CONFIGURE_OPTIONS -DANDROID_ABI=arm64-v8a)
set(VCPKG_BUILD_TYPE release)
