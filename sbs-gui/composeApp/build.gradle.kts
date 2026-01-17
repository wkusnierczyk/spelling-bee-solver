import org.jetbrains.compose.desktop.application.dsl.TargetFormat

plugins {
    kotlin("multiplatform")
    kotlin("plugin.serialization")
    id("org.jetbrains.compose")
}

kotlin {
    jvm("desktop")
    
    // SWITCHED FROM wasmJs TO js(IR) FOR STABILITY
    js(IR) {
        moduleName = "sbs-gui"
        browser {
            commonWebpackConfig {
                outputFileName = "sbs-gui.js"
            }
        }
        binaries.executable()
    }
    
    sourceSets {
        val commonMain by getting {
            dependencies {
                implementation(compose.runtime)
                implementation(compose.foundation)
                implementation(compose.material)
                implementation(compose.ui)
                implementation(compose.components.resources)
                
                // Ktor 2.3.12 is stable for JS
                val ktorVersion = "2.3.12"
                implementation("io.ktor:ktor-client-core:$ktorVersion")
                implementation("io.ktor:ktor-client-content-negotiation:$ktorVersion")
                implementation("io.ktor:ktor-serialization-kotlinx-json:$ktorVersion")
            }
        }
        val desktopMain by getting {
            dependencies {
                implementation(compose.desktop.currentOs)
                implementation("io.ktor:ktor-client-cio:2.3.12")
            }
        }
        val jsMain by getting {
            dependencies {
                // The JS engine is robust and stable
                implementation("io.ktor:ktor-client-js:2.3.12")
            }
        }
    }
}

compose.desktop {
    application {
        mainClass = "MainKt"
        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
            packageName = "sbs-gui"
            packageVersion = "1.0.0"
        }
    }
}
