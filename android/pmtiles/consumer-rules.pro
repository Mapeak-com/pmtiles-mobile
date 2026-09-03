# Shipped inside the AAR and applied to any consuming app that minifies with
# R8/ProGuard.
#
# Almost none of the FFI surface is reachable through ordinary Java references:
# JNA wires it up at runtime from the *names* of classes, methods and fields.
# Anything renamed or stripped here fails at runtime, not at build time.

# --- JNA ---------------------------------------------------------------------

# JNA resolves its own machinery reflectively (Native, Structure, the type
# mappers, CallbackReference, com.sun.jna.internal.Cleaner, ...).
-keep class com.sun.jna.** { *; }
-keep interface com.sun.jna.** { *; }

# com.sun.jna.Native$AWT references java.awt, which does not exist on Android.
# That code path is never taken here.
-dontwarn java.awt.**

# --- UniFFI bindings ---------------------------------------------------------

# UniffiLib and IntegrityCheckingUniffiLib use JNA direct mapping
# (Native.register), binding each `external fun` to the native symbol of the
# same name in libpmtiles_core.so, so their method names must survive.
-keep class com.mapeak.pmtiles.UniffiLib { *; }
-keep class com.mapeak.pmtiles.IntegrityCheckingUniffiLib { *; }

# JNA lays each Structure out from the field names listed in its
# @Structure.FieldOrder annotation (read at runtime), and instantiates the
# nested ByValue/ByReference subclasses through their no-arg constructors.
# Covers RustBuffer, ForeignBytes, UniffiRustCallStatus and the
# UniffiForeignFutureResult* family.
-keepattributes *Annotation*
-keep class * extends com.sun.jna.Structure { *; }

# Rust calls back into these through a JNA proxy, which locates the single
# callback method by reflection.
-keep interface * extends com.sun.jna.Callback { *; }
-keepclassmembers class * implements com.sun.jna.Callback { <methods>; }
