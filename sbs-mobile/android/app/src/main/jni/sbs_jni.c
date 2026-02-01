#include <jni.h>
#include <string.h>
#include <stdlib.h>

/* FFI function declarations from libsbs_ffi */
extern void *sbs_load_dictionary(const char *path);
extern void  sbs_free_dictionary(void *ptr);
extern char *sbs_solve(const void *dict, const char *request_json);
extern void  sbs_free_string(char *s);
extern const char *sbs_version(void);

JNIEXPORT jlong JNICALL
Java_com_sbsmobile_SbsSolverModule_sbsLoadDictionary(
    JNIEnv *env, jobject thiz, jstring path) {
    const char *c_path = (*env)->GetStringUTFChars(env, path, NULL);
    if (!c_path) return 0;
    void *ptr = sbs_load_dictionary(c_path);
    (*env)->ReleaseStringUTFChars(env, path, c_path);
    return (jlong)(intptr_t)ptr;
}

JNIEXPORT void JNICALL
Java_com_sbsmobile_SbsSolverModule_sbsFreeDictionary(
    JNIEnv *env, jobject thiz, jlong ptr) {
    if (ptr != 0) {
        sbs_free_dictionary((void *)(intptr_t)ptr);
    }
}

JNIEXPORT jstring JNICALL
Java_com_sbsmobile_SbsSolverModule_sbsSolve(
    JNIEnv *env, jobject thiz, jlong dict_ptr, jstring request_json) {
    if (dict_ptr == 0) {
        return (*env)->NewStringUTF(env, "{\"error\":\"null dictionary pointer\"}");
    }
    const char *c_request = (*env)->GetStringUTFChars(env, request_json, NULL);
    if (!c_request) {
        return (*env)->NewStringUTF(env, "{\"error\":\"null request\"}");
    }
    char *result = sbs_solve((const void *)(intptr_t)dict_ptr, c_request);
    (*env)->ReleaseStringUTFChars(env, request_json, c_request);
    if (!result) {
        return (*env)->NewStringUTF(env, "{\"error\":\"solve returned null\"}");
    }
    jstring j_result = (*env)->NewStringUTF(env, result);
    sbs_free_string(result);
    return j_result;
}

JNIEXPORT void JNICALL
Java_com_sbsmobile_SbsSolverModule_sbsFreeString(
    JNIEnv *env, jobject thiz, jlong ptr) {
    if (ptr != 0) {
        sbs_free_string((char *)(intptr_t)ptr);
    }
}

JNIEXPORT jstring JNICALL
Java_com_sbsmobile_SbsSolverModule_sbsVersion(
    JNIEnv *env, jobject thiz) {
    const char *version = sbs_version();
    return (*env)->NewStringUTF(env, version ? version : "unknown");
}
