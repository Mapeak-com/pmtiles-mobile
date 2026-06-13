import org.gradle.api.DefaultTask
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.provider.ListProperty
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputDirectory
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.Optional
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import org.gradle.process.ExecOperations
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import javax.inject.Inject

plugins {
    // AGP 9 has built-in Kotlin; no separate kotlin.android plugin.
    id("com.android.library")
    id("maven-publish")
}

val ndkVer = "28.2.13676358"
val cargoAbis = listOf("arm64-v8a", "armeabi-v7a", "x86_64")

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

    publishing {
        singleVariant("release") { withSourcesJar() }
    }
}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}

// Cross-compiles the Rust core into libpmtiles_core.so per ABI via cargo-ndk,
// writing into the AGP-provided jniLibs source directory.
abstract class CargoNdkBuild @Inject constructor(private val execOps: ExecOperations) : DefaultTask() {
    @get:OutputDirectory abstract val outputDir: DirectoryProperty
    @get:Internal abstract val coreDir: DirectoryProperty
    @get:Input abstract val abis: ListProperty<String>
    @get:Input @get:Optional abstract val ndkHome: Property<String>

    @TaskAction
    fun build() {
        val out = outputDir.get().asFile.apply { mkdirs() }
        execOps.exec {
            workingDir = coreDir.get().asFile
            ndkHome.orNull?.let { environment("ANDROID_NDK_HOME", it) }
            val args = mutableListOf("ndk", "-o", out.absolutePath, "-P", "21")
            abis.get().forEach { args += listOf("-t", it) }
            args += listOf("build", "--release")
            commandLine = listOf("cargo") + args
        }
    }
}

// Generates the UniFFI Kotlin bindings from the compiled library's metadata.
abstract class UniffiBindgen @Inject constructor(private val execOps: ExecOperations) : DefaultTask() {
    @get:OutputDirectory abstract val outputDir: DirectoryProperty
    @get:InputDirectory @get:PathSensitive(PathSensitivity.RELATIVE) abstract val jniLibsDir: DirectoryProperty
    @get:Internal abstract val coreDir: DirectoryProperty

    @TaskAction
    fun generate() {
        val out = outputDir.get().asFile.apply { mkdirs() }
        val lib = jniLibsDir.get().dir("arm64-v8a").file("libpmtiles_core.so").asFile
        execOps.exec {
            workingDir = coreDir.get().asFile
            commandLine = listOf(
                "cargo", "run", "--quiet", "--bin", "uniffi-bindgen", "--",
                "generate", "--library", lib.absolutePath,
                "--language", "kotlin", "--out-dir", out.absolutePath, "--no-format",
            )
        }
    }
}

androidComponents {
    onVariants(selector().withBuildType("release")) { variant ->
        val core = layout.projectDirectory.dir("../../core")

        val cargo = tasks.register<CargoNdkBuild>("cargoNdkBuild") {
            coreDir.set(core)
            abis.set(cargoAbis)
            (System.getenv("ANDROID_HOME") ?: System.getenv("ANDROID_SDK_ROOT"))?.let {
                ndkHome.set("$it/ndk/$ndkVer")
            }
        }
        val bindgen = tasks.register<UniffiBindgen>("uniffiBindgen") {
            coreDir.set(core)
            jniLibsDir.set(cargo.flatMap { it.outputDir })
        }

        // addGeneratedSourceDirectory registers each dir as the task's output, so
        // every consumer (compile, sources jar, lint, annotations…) auto-depends.
        variant.sources.jniLibs?.addGeneratedSourceDirectory(cargo, CargoNdkBuild::outputDir)
        variant.sources.java?.addGeneratedSourceDirectory(bindgen, UniffiBindgen::outputDir)
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.19.1@aar")
}

// The JNA dependency must resolve as the Android `aar` (it ships
// libjnidispatch.so) — but the `@aar` type is only recorded in the POM, not in
// Gradle Module Metadata. Disable module metadata so consumers use the POM and
// get the aar; otherwise they get the desktop jar and hit UnsatisfiedLinkError.
tasks.withType<GenerateModuleMetadata>().configureEach { enabled = false }

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
