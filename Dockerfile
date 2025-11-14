FROM rust:1.86-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    unzip \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    openjdk-17-jdk \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 20.x
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y nodejs && \
    rm -rf /var/lib/apt/lists/*

# Set up Android SDK
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_SDK_ROOT=${ANDROID_HOME}
ENV PATH="${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${PATH}"

# Install Android command line tools
RUN mkdir -p ${ANDROID_HOME}/cmdline-tools && \
    cd ${ANDROID_HOME}/cmdline-tools && \
    wget -q https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip && \
    unzip -q commandlinetools-linux-11076708_latest.zip && \
    mv cmdline-tools latest && \
    rm commandlinetools-linux-11076708_latest.zip

# Accept Android SDK licenses and install NDK
ENV ANDROID_NDK_VERSION=29.0.14033849
ENV ANDROID_NDK_HOME=${ANDROID_HOME}/ndk/${ANDROID_NDK_VERSION}
ENV ANDROID_NDK_ROOT=${ANDROID_NDK_HOME}
ENV ANDROID_NDK=${ANDROID_NDK_HOME}

RUN yes | sdkmanager --licenses && \
    sdkmanager --install \
    "ndk;${ANDROID_NDK_VERSION}" \
    "platform-tools" \
    "platforms;android-33" \
    "build-tools;33.0.2" && \
    rm -rf ${ANDROID_HOME}/.android

# Install Rust targets for Android
RUN rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android

# Install Rust targets for iOS
RUN rustup target add \
    aarch64-apple-ios \
    x86_64-apple-ios \
    aarch64-apple-ios-sim

# Install cargo-ndk
RUN cargo install cargo-ndk

# Set up Rust environment
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH="${CARGO_HOME}/bin:${PATH}"

# Create a non-root user for running builds
RUN useradd -m -u 1001 builder && \
    chown -R builder:builder ${ANDROID_HOME} ${CARGO_HOME} ${RUSTUP_HOME}

USER builder
WORKDIR /workspace

# Verify installations
RUN node --version && \
    npm --version && \
    rustc --version && \
    cargo --version && \
    java -version

CMD ["/bin/bash"]
