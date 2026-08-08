#include <jni.h>
#include <string>

extern "C" JNIEXPORT jstring JNICALL
Java_com_nishanth_1kj_internetdownloader_MainActivity_stringFromJNI(
        JNIEnv* env,
        jobject /* this */) {
    std::string hello = "Hello from C++ Native Code!";
    return env->NewStringUTF(hello.c_str());
}
