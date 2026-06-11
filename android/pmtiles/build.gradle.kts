import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("maven-publish")
}

val ndkVer = "28.2.13676358"
val abis = listOf("arm64-v8a", "armeabi-v7a", "x86_64")

android {
    namespace = "com.mapeak.pmtiles"
    compileSdk = 34
    ndkVersion = ndkVer

    defaultConfig {
        minSdk = 21
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }

    // Rust .so files (built by :cargoNdkBuild) + generated UniFFI Kotlin.
    sourceSets["main"].jniLibs.srcDir(layout.buildDirectory.dir("jniLibs"))
    sourceSets["main"].java.srcDir(layout.buildDirectory.dir("generated/uniffi"))

    publishing {
        singleVariant("release") { withSourcesJar() }
    }
}

val jniLibsDir = layout.buildDirectory.dir("jniLibs")

// Cross-compile the Rust core into libpmtiles_core.so per ABI via cargo-ndk.
val cargoNdkBuild = tasks.register<Exec>("cargoNdkBuild") {
    workingDir = file("../../core")
    environment("ANDROID_NDK_HOME", android.sdkDirectory.resolve("ndk/$ndkVer").absolutePath)
    val args = mutableListOf("ndk", "-o", jniLibsDir.get().asFile.absolutePath, "-P", "21")
    abis.forEach { args += listOf("-t", it) }
    args += listOf("build", "--release")
    commandLine("cargo")
    setArgs(args)
    outputs.dir(jniLibsDir)
}

// Generate the UniFFI Kotlin bindings from the compiled library's metadata.
val uniffiBindgen = tasks.register<Exec>("uniffiBindgen") {
    dependsOn(cargoNdkBuild)
    workingDir = file("../../core")
    val lib = jniLibsDir.get().file("arm64-v8a/libpmtiles_core.so").asFile
    val outDir = layout.buildDirectory.dir("generated/uniffi").get().asFile
    outputs.dir(outDir)
    commandLine(
        "cargo", "run", "--quiet", "--bin", "uniffi-bindgen", "--",
        "generate",
        "--library", lib.absolutePath,
        "--language", "kotlin",
        "--out-dir", outDir.absolutePath,
        "--no-format",
    )
}

tasks.named("preBuild").configure { dependsOn(cargoNdkBuild) }
tasks.withType<KotlinCompile>().configureEach { dependsOn(uniffiBindgen) }
// The sources jar (withSourcesJar) also reads the generated/uniffi dir.
tasks.matching { it.name.startsWith("source") && it.name.endsWith("Jar") }
    .configureEach { dependsOn(uniffiBindgen) }

dependencies {
    implementation("net.java.dev.jna:jna:5.14.0@aar")
}

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = "com.mapeak"
            artifactId = "pmtiles"
            version = (System.getenv("VERSION")
                ?: System.getenv("PACKAGE_VERSION")
                ?: "0.1.0").removePrefix("v")
            afterEvaluate { from(components["release"]) }
        }
    }
}
