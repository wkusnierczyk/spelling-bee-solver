plugins {
    // Versions managed here
    val kotlinVersion = "1.9.21"
    val composeVersion = "1.6.1"
    
    kotlin("multiplatform") version kotlinVersion apply false
    kotlin("plugin.serialization") version kotlinVersion apply false
    id("org.jetbrains.compose") version composeVersion apply false
}
