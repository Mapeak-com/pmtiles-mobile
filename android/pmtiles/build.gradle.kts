import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("org.mozilla.rust-android-gradle.rust-android") version "0.9.6"
    id("maven-publish")
}

android {
    namespace = "com.mapeak.pmtiles"
    compileSdk = 34
    ndkVersion = "28.2.13676358"

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

    sourceSets["main"].jniLibs.srcDir(layout.buildDirectory.dir("rustJniLibs/android"))
    sourceSets["main"].java.srcDir(layout.buildDirectory.dir("generated/uniffi"))

    publishing {
        singleVariant("release") { withSourcesJar() }
    }
}

cargo {
    module = "../../core"
    libname = "pmtiles_core"
    targets = listOf("arm64", "x86_64", "arm")
    profile = "release"
}

val uniffiBindgen = tasks.register<Exec>("uniffiBindgen") {
    dependsOn("cargoBuild")
    workingDir = file("../../core")
    val lib = layout.buildDirectory
        .file("rustJniLibs/android/arm64-v8a/libpmtiles_core.so").get().asFile
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

tasks.named("preBuild").configure { dependsOn("cargoBuild") }
tasks.withType<KotlinCompile>().configureEach { dependsOn(uniffiBindgen) }

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
