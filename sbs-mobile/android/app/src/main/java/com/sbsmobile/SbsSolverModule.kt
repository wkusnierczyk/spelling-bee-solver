package com.sbsmobile

import com.facebook.react.bridge.Promise
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.ReactContextBaseJavaModule
import com.facebook.react.bridge.ReactMethod
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream

class SbsSolverModule(reactContext: ReactApplicationContext) :
    ReactContextBaseJavaModule(reactContext) {

    companion object {
        const val NAME = "SbsSolver"

        @Volatile
        private var dictionaryPtr: Long = 0

        private var nativeLoaded = false

        @Synchronized
        fun ensureNativeLoaded() {
            if (!nativeLoaded) {
                System.loadLibrary("sbs_ffi")
                System.loadLibrary("sbs_jni")
                nativeLoaded = true
            }
        }
    }

    private external fun sbsLoadDictionary(path: String): Long
    private external fun sbsFreeDictionary(ptr: Long)
    private external fun sbsSolve(dictPtr: Long, requestJson: String): String
    private external fun sbsFreeString(ptr: Long)
    private external fun sbsVersion(): String

    override fun getName(): String = NAME

    private fun ensureDictionary(): Long {
        if (dictionaryPtr != 0L) return dictionaryPtr

        ensureNativeLoaded()

        val context = reactApplicationContext
        val dictFile = File(context.filesDir, "dictionary.txt")

        if (!dictFile.exists()) {
            context.assets.open("dictionary.txt").use { input ->
                FileOutputStream(dictFile).use { output ->
                    input.copyTo(output)
                }
            }
        }

        val ptr = sbsLoadDictionary(dictFile.absolutePath)
        if (ptr == 0L) {
            throw RuntimeException("Failed to load dictionary from ${dictFile.absolutePath}")
        }
        dictionaryPtr = ptr
        return ptr
    }

    @ReactMethod
    fun solve(letters: String, present: String, repeats: Int, promise: Promise) {
        if (letters.isEmpty() || present.isEmpty()) {
            promise.reject("INVALID_INPUT", "letters and present must not be empty")
            return
        }

        try {
            val dictPtr = ensureDictionary()
            val request = JSONObject().apply {
                put("letters", letters)
                put("present", present)
                if (repeats > 0) {
                    put("repeats", repeats)
                }
            }

            val resultJson = sbsSolve(dictPtr, request.toString())
            val result = JSONObject(resultJson)

            if (result.has("error")) {
                promise.reject("SOLVE_ERROR", result.getString("error"))
            } else {
                promise.resolve(resultJson)
            }
        } catch (e: Exception) {
            promise.reject("SOLVE_ERROR", e.message, e)
        }
    }

    @ReactMethod
    fun version(promise: Promise) {
        try {
            ensureNativeLoaded()
            promise.resolve(sbsVersion())
        } catch (e: Exception) {
            promise.reject("VERSION_ERROR", e.message, e)
        }
    }
}
