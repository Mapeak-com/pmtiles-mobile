plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("maven-publish")
}

android {
    namespace = "com.mapeak.pmtiles"
    compileSdk = 34

    defaultConfig {
        minSdk = 21
        externalNativeBuild {
            cmake {
                cppFlags += "-std=c++17"
            }
        }
        ndk {
            // ABIs to ship in the AAR.
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
            version = "3.22.1"
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }

    sourceSets["main"].kotlin.srcDir("src/main/kotlin")

    publishing {
        singleVariant("release") { withSourcesJar() }
    }
}

dependencies {
    // none needed for the core reader
}

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = "com.mapeak"
            artifactId = "pmtiles"
            // JitPack builds from a git tag and sets $VERSION to that tag; locally
            // PACKAGE_VERSION can override. Falls back to a dev version otherwise.
            version = (System.getenv("VERSION")
                ?: System.getenv("PACKAGE_VERSION")
                ?: "0.1.0").removePrefix("v")
            afterEvaluate { from(components["release"]) }
        }
    }
    // No remote repository block is needed: JitPack consumes the artifact that
    // `publishToMavenLocal` writes to ~/.m2 when it builds the repo at a tag.
}
