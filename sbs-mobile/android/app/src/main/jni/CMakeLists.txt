cmake_minimum_required(VERSION 3.22)
project(sbs_jni C)

add_library(sbs_ffi SHARED IMPORTED)
set_target_properties(sbs_ffi PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libsbs_ffi.so
    IMPORTED_NO_SONAME TRUE
)

add_library(sbs_jni SHARED sbs_jni.c)
target_link_libraries(sbs_jni sbs_ffi log)

# Ensure the linker records "libsbs_ffi.so" (name only) rather than the
# absolute build-time path, so the Android runtime linker can find the
# library from the APK's lib directory.
set_target_properties(sbs_jni PROPERTIES
    BUILD_RPATH ""
    INSTALL_RPATH ""
)
