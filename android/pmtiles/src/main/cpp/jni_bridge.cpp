// jni_bridge.cpp — thin JNI layer. All real work is in the C-API.
#include <jni.h>
#include "pmtiles_c.h"

extern "C" {

JNIEXPORT jlong JNICALL
Java_com_mapeak_pmtiles_PMTilesReader_nativeOpen(JNIEnv *env, jobject, jstring jpath) {
  const char *path = env->GetStringUTFChars(jpath, nullptr);
  pmtiles_reader *r = pmtiles_open(path);
  env->ReleaseStringUTFChars(jpath, path);
  return reinterpret_cast<jlong>(r);
}

JNIEXPORT void JNICALL
Java_com_mapeak_pmtiles_PMTilesReader_nativeClose(JNIEnv *, jobject, jlong handle) {
  pmtiles_close(reinterpret_cast<pmtiles_reader *>(handle));
}

JNIEXPORT jbyteArray JNICALL
Java_com_mapeak_pmtiles_PMTilesReader_nativeGetTile(JNIEnv *env, jobject, jlong handle,
                                               jint z, jint x, jint y) {
  auto *r = reinterpret_cast<pmtiles_reader *>(handle);
  uint8_t *data = nullptr;
  size_t len = 0;
  if (pmtiles_get_tile(r, z, x, y, &data, &len) != 0 || data == nullptr) {
    return nullptr; // error or tile absent
  }
  jbyteArray out = env->NewByteArray(static_cast<jsize>(len));
  env->SetByteArrayRegion(out, 0, static_cast<jsize>(len),
                          reinterpret_cast<const jbyte *>(data));
  pmtiles_free(data);
  return out;
}

} // extern "C"
