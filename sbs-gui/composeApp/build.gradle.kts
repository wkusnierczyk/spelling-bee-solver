import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.jetbrains.kotlin.gradle.targets.js.webpack.KotlinWebpack

plugins {
    kotlin("multiplatform")
    kotlin("plugin.serialization")
    id("org.jetbrains.compose")
}

kotlin {
    jvm("desktop")
    
    js(IR) {
        moduleName = "sbs-gui"
        browser {
            commonWebpackConfig {
                outputFileName = "sbs-gui.js"
                // REVERTED: cssSupport removed to fix build failure
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

// --- FORCE COPY TASK ---
// This task runs after the build. It hunts down skiko.wasm in the build cache
// and copies it to the distribution folder so Docker can find it.
tasks.register("copyWasmToDist") {
    dependsOn("jsBrowserDistribution")
    doLast {
        val distDir = file("build/dist/js/productionExecutable")
        // We look inside the npm extraction folder for skiko
        val npmDir = file("build/js/packages/sbs-gui/node_modules")
        
        // Find skiko.wasm recursively
        val wasmFiles = fileTree(npmDir).matching { include("**/skiko.wasm") }
        
        if (wasmFiles.isEmpty) {
            println("WARNING: Could not find skiko.wasm in $npmDir")
        } else {
            wasmFiles.forEach { file ->
                println("Copying WASM: ${file.absolutePath} -> $distDir")
                copy {
                    from(file)
                    into(distDir)
                }
            }
        }
    }
}

// Make sure the copy runs when we ask for the distribution
tasks.named("jsBrowserDistribution") {
    finalizedBy("copyWasmToDist")
}
