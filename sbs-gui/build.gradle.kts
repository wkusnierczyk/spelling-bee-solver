plugins {
    // UPDATED VERSIONS for Wasm Support
    val kotlinVersion = "1.9.23"
    val composeVersion = "1.6.2"
    
    kotlin("multiplatform") version kotlinVersion apply false
    kotlin("plugin.serialization") version kotlinVersion apply false
    id("org.jetbrains.compose") version composeVersion apply false
}
