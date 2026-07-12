# Reproducible build environment for the FRANK Scanlation Android APK.
#
# Usage:
#   docker build -t frank-scanlation-android-builder -f mobile-build/Dockerfile .
#   docker run --rm -v "$PWD":/work -w /work frank-scanlation-android-builder \
#       ./mobile-build/build.sh
#
# Produces APKs under desktop/src-tauri/gen/android/app/build/outputs/apk.

FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive \
    ANDROID_HOME=/opt/android-sdk \
    ANDROID_SDK_ROOT=/opt/android-sdk \
    NDK_VERSION=28.2.13676358 \
    JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64 \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH="/usr/local/cargo/bin:/usr/local/rustup/bin:${PATH}"

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        unzip \
        git \
        build-essential \
        pkg-config \
        libssl-dev \
        openjdk-17-jdk \
        clang \
        llvm \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://www.python.org/ftp/python/3.11.9/Python-3.11.9.tgz -o /tmp/python.tgz \
    && tar -xzf /tmp/python.tgz -C /tmp \
    && cd /tmp/Python-3.11.9 && ./configure --quiet && make -j"$(nproc)" install \
    && rm -rf /tmp/python.tgz /tmp/Python-3.11.9

RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal \
    --default-toolchain stable \
    && rustup target add aarch64-linux-android armv7-linux-androideabi \
        i686-linux-android x86_64-linux-android

RUN mkdir -p "${ANDROID_HOME}/cmdline-tools" \
    && curl -fsSL https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip -o /tmp/cmdline.zip \
    && unzip -q /tmp/cmdline.zip -d "${ANDROID_HOME}/cmdline-tools" \
    && mv "${ANDROID_HOME}/cmdline-tools/cmdline-tools" \
        "${ANDROID_HOME}/cmdline-tools/latest" \
    && rm /tmp/cmdline.zip \
    && yes | "${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager" \
        --licenses > /dev/null 2>&1 \
    && "${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager" \
        "platform-tools" \
        "platforms;android-36" \
        "build-tools;36.0.0" \
        "ndk;${NDK_VERSION}"

ENV ANDROID_NDK_HOME="${ANDROID_HOME}/ndk/${NDK_VERSION}" \
    NDK_HOME="${ANDROID_HOME}/ndk/${NDK_VERSION}"

RUN curl -fsSL https://bun.sh/install | bash \
    && mv /root/.bun /usr/local/bun \
    && ln -s /usr/local/bun/bin/bun /usr/local/bin/bun

RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /work
