# JNA looks classes and fields up reflectively; keep them through R8/ProGuard in consuming apps.
-dontwarn java.awt.*
-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { public *; }
-keep class uniffi.yorion_engine.** { *; }
