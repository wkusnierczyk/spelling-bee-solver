cmake_minimum_required(VERSION 3.22)
project(sbs_jni C)

add_library(sbs_ffi SHARED IMPORTED)
set_target_properties(sbs_ffi PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libsbs_ffi.so
)

add_library(sbs_jni SHARED sbs_jni.c)
target_link_libraries(sbs_jni sbs_ffi log)
