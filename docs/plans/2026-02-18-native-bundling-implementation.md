# Native Library Bundling Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 5개 플랫폼의 네이티브 라이브러리를 Kotlin JAR에 번들링하고, 릴리즈 시 Maven Central에 자동 배포하는 CI 파이프라인을 구축한다.

**Architecture:** GitHub Actions 매트릭스 빌드로 5개 타겟을 크로스 컴파일하고, 생성된 네이티브 라이브러리를 수집하여 Gradle로 fat JAR를 패키징한 뒤 Maven Central에 배포한다.

**Tech Stack:** GitHub Actions, Rust cross-compilation, Gradle (signing + maven-publish), Maven Central (OSSRH)

---

## Task 1: Gradle에 signing 플러그인 및 Maven Central 저장소 설정 추가

**Files:**
- Modify: `bindings/kotlin/build.gradle.kts`
- Modify: `bindings/kotlin/gradle.properties`

**Step 1: gradle.properties에 Maven Central 설정 추가**

`bindings/kotlin/gradle.properties`를 다음으로 교체:

```properties
group=io.clroot.slimg
version=0.1.2
kotlin.code.style=official

# Maven Central (OSSRH) — values provided via env or ~/.gradle/gradle.properties
mavenCentralUsername=
mavenCentralPassword=

# GPG signing
signing.keyId=
signing.password=
signing.secretKeyRingFile=
```

**Step 2: build.gradle.kts에 signing + Maven Central repository 추가**

`bindings/kotlin/build.gradle.kts`를 다음으로 교체:

```kotlin
plugins {
    kotlin("jvm") version "2.1.0"
    `maven-publish`
    signing
}

group = property("group") as String
version = property("version") as String

repositories {
    mavenCentral()
}

dependencies {
    implementation("net.java.dev.jna:jna:5.16.0")
    testImplementation(kotlin("test"))
}

kotlin {
    jvmToolchain(17)
}

tasks.test {
    useJUnitPlatform()
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])

            pom {
                name.set("slimg-kotlin")
                description.set("Kotlin bindings for slimg image optimization library")
                url.set("https://github.com/clroot/slimg")

                licenses {
                    license {
                        name.set("MIT OR Apache-2.0")
                        url.set("https://github.com/clroot/slimg/blob/main/LICENSE")
                    }
                }

                developers {
                    developer {
                        id.set("clroot")
                        name.set("clroot")
                    }
                }

                scm {
                    connection.set("scm:git:git://github.com/clroot/slimg.git")
                    developerConnection.set("scm:git:ssh://github.com/clroot/slimg.git")
                    url.set("https://github.com/clroot/slimg")
                }
            }
        }
    }

    repositories {
        maven {
            name = "OSSRH"
            url = uri("https://s01.oss.sonatype.org/service/local/staging/deploy/maven2/")
            credentials {
                username = findProperty("mavenCentralUsername") as String? ?: System.getenv("MAVEN_CENTRAL_USERNAME") ?: ""
                password = findProperty("mavenCentralPassword") as String? ?: System.getenv("MAVEN_CENTRAL_PASSWORD") ?: ""
            }
        }
    }
}

signing {
    // CI에서는 환경 변수로, 로컬에서는 gradle.properties로 설정
    val signingKey = System.getenv("GPG_SIGNING_KEY")
    val signingPassword = System.getenv("GPG_SIGNING_PASSWORD")
    if (signingKey != null && signingPassword != null) {
        useInMemoryPgpKeys(signingKey, signingPassword)
    }
    sign(publishing.publications["maven"])
}

// signing은 publish할 때만 필요
tasks.withType<Sign>().configureEach {
    onlyIf { gradle.taskGraph.hasTask("publish") }
}
```

**Step 3: 빌드 확인**

Run: `cd bindings/kotlin && ./gradlew build`
Expected: BUILD SUCCESSFUL

**Step 4: Commit**

```bash
git add bindings/kotlin/build.gradle.kts bindings/kotlin/gradle.properties
git commit -m "feat(kotlin): add signing and Maven Central publishing config"
```

---

## Task 2: 릴리즈 워크플로우 생성 (kotlin-release.yml)

**Files:**
- Create: `.github/workflows/kotlin-release.yml`

**Step 1: 워크플로우 파일 생성**

`.github/workflows/kotlin-release.yml`:

```yaml
name: Kotlin Release

on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release (e.g. 0.1.2)'
        required: true
        type: string

permissions:
  contents: read

jobs:
  build-native:
    name: Build native (${{ matrix.target }})
    runs-on: ${{ matrix.runner }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: aarch64-apple-darwin
            runner: macos-latest
            lib: libslimg_ffi.dylib
            resource_dir: darwin-aarch64

          - target: x86_64-apple-darwin
            runner: macos-13
            lib: libslimg_ffi.dylib
            resource_dir: darwin-x86-64

          - target: x86_64-unknown-linux-gnu
            runner: ubuntu-latest
            lib: libslimg_ffi.so
            resource_dir: linux-x86-64

          - target: aarch64-unknown-linux-gnu
            runner: ubuntu-latest
            lib: libslimg_ffi.so
            resource_dir: linux-aarch64
            cross: true

          - target: x86_64-pc-windows-msvc
            runner: windows-latest
            lib: slimg_ffi.dll
            resource_dir: win32-x86-64

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install system dependencies (macOS)
        if: runner.os == 'macOS'
        run: brew install nasm dav1d

      - name: Install system dependencies (Linux)
        if: runner.os == 'Linux' && !matrix.cross
        run: sudo apt-get update && sudo apt-get install -y nasm

      - name: Install cross (Linux aarch64)
        if: matrix.cross
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Add NASM to PATH (Windows)
        if: runner.os == 'Windows'
        shell: bash
        run: echo "C:/Program Files/NASM" >> $GITHUB_PATH

      - name: Install NASM (Windows)
        if: runner.os == 'Windows'
        shell: bash
        run: choco install nasm -y

      - name: Build with cargo
        if: ${{ !matrix.cross }}
        run: cargo build --release -p slimg-ffi --target ${{ matrix.target }}

      - name: Build with cross
        if: ${{ matrix.cross }}
        run: cross build --release -p slimg-ffi --target ${{ matrix.target }}

      - name: Upload native library
        uses: actions/upload-artifact@v4
        with:
          name: native-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/${{ matrix.lib }}

  generate-bindings:
    name: Generate Kotlin bindings
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install system dependencies
        run: brew install nasm dav1d

      - name: Build slimg-ffi
        run: cargo build --release -p slimg-ffi

      - name: Generate Kotlin bindings
        run: |
          cargo run -p slimg-ffi --bin uniffi-bindgen generate \
            --library target/release/libslimg_ffi.dylib \
            --language kotlin \
            --out-dir generated-kotlin

      - name: Upload generated Kotlin
        uses: actions/upload-artifact@v4
        with:
          name: kotlin-bindings
          path: generated-kotlin/

  package-and-publish:
    name: Package and publish
    needs: [build-native, generate-bindings]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all native libraries
        uses: actions/download-artifact@v4
        with:
          path: native-libs/

      - name: Download generated Kotlin
        uses: actions/download-artifact@v4
        with:
          name: kotlin-bindings
          path: generated-kotlin/

      - name: Arrange native libraries into resources
        run: |
          # Copy generated Kotlin source
          cp -r generated-kotlin/* bindings/kotlin/src/main/kotlin/

          # Copy each platform's native library to the correct resource directory
          for target_dir in native-libs/native-*; do
            target=$(basename "$target_dir" | sed 's/native-//')
            case "$target" in
              aarch64-apple-darwin)    res_dir="darwin-aarch64" ;;
              x86_64-apple-darwin)     res_dir="darwin-x86-64" ;;
              x86_64-unknown-linux-gnu) res_dir="linux-x86-64" ;;
              aarch64-unknown-linux-gnu) res_dir="linux-aarch64" ;;
              x86_64-pc-windows-msvc)  res_dir="win32-x86-64" ;;
            esac
            mkdir -p "bindings/kotlin/src/main/resources/$res_dir"
            cp "$target_dir"/* "bindings/kotlin/src/main/resources/$res_dir/"
          done

          # List what we have
          find bindings/kotlin/src/main/resources -type f

      - name: Set up JDK 17
        uses: actions/setup-java@v4
        with:
          java-version: '17'
          distribution: 'temurin'

      - name: Setup Gradle
        uses: gradle/actions/setup-gradle@v4

      - name: Set version
        working-directory: bindings/kotlin
        run: |
          sed -i "s/^version=.*/version=${{ inputs.version }}/" gradle.properties

      - name: Build and test
        working-directory: bindings/kotlin
        run: ./gradlew build

      - name: Publish to Maven Central
        if: ${{ inputs.version != '' }}
        working-directory: bindings/kotlin
        env:
          MAVEN_CENTRAL_USERNAME: ${{ secrets.MAVEN_CENTRAL_USERNAME }}
          MAVEN_CENTRAL_PASSWORD: ${{ secrets.MAVEN_CENTRAL_PASSWORD }}
          GPG_SIGNING_KEY: ${{ secrets.GPG_SIGNING_KEY }}
          GPG_SIGNING_PASSWORD: ${{ secrets.GPG_SIGNING_PASSWORD }}
        run: ./gradlew publishMavenPublicationToOSSRHRepository

      - name: Upload JAR artifact
        uses: actions/upload-artifact@v4
        with:
          name: slimg-kotlin-${{ inputs.version }}
          path: bindings/kotlin/build/libs/*.jar
```

**Step 2: Commit**

```bash
git add .github/workflows/kotlin-release.yml
git commit -m "ci: add Kotlin release workflow with cross-platform native bundling"
```

---

## Task 3: cross 빌드 설정 (Linux aarch64용)

**Files:**
- Create: `Cross.toml`

**Step 1: Cross.toml 생성**

워크스페이스 루트에 `Cross.toml` 생성:

```toml
[target.aarch64-unknown-linux-gnu]
# Use default cross image with build essentials
# NASM is needed for mozjpeg
pre-build = ["apt-get update && apt-get install -y nasm"]
```

**Step 2: Commit**

```bash
git add Cross.toml
git commit -m "ci: add Cross.toml for Linux aarch64 cross-compilation"
```

---

## Task 4: kotlin-bindings.yml 업데이트 (UniFFI message 충돌 자동 패치)

현재 UniFFI가 생성한 코드에서 `SlimgException` 서브클래스의 `message` 파라미터가 `Throwable.message`와 충돌하는 문제가 있음. 릴리즈 워크플로우에서 생성할 때도 같은 문제가 발생하므로 자동 패치 스크립트가 필요.

**Files:**
- Create: `bindings/scripts/patch-generated-kotlin.sh`

**Step 1: 패치 스크립트 생성**

```bash
#!/usr/bin/env bash
# Patch UniFFI-generated Kotlin code for Kotlin 2.x compatibility.
# The generated SlimgException subclasses declare a `message` constructor
# parameter that conflicts with Throwable.message.
set -euo pipefail

GENERATED_FILE="$1"

if [[ ! -f "$GENERATED_FILE" ]]; then
    echo "Error: File not found: $GENERATED_FILE"
    exit 1
fi

# In SlimgException subclasses, rename constructor parameter `message` to `msg`
# and update the corresponding message override.
sed -i.bak -E '
    /class (Decode|Encode|Resize|Io|Image)\(/,/\}/ {
        s/val `message`: kotlin\.String/val `msg`: kotlin.String/
        s/get\(\) = "message=\$\{ `message` \}"/get() = "message=${ `msg` }"/
    }
' "$GENERATED_FILE"

# Also update FfiConverterTypeSlimgError references
sed -i.bak -E '
    s/SlimgException\.Decode\(`message`/SlimgException.Decode(`msg`/
    s/SlimgException\.Encode\(`message`/SlimgException.Encode(`msg`/
    s/SlimgException\.Resize\(`message`/SlimgException.Resize(`msg`/
    s/SlimgException\.Io\(`message`/SlimgException.Io(`msg`/
    s/SlimgException\.Image\(`message`/SlimgException.Image(`msg`/
' "$GENERATED_FILE"

rm -f "${GENERATED_FILE}.bak"
echo "Patched: $GENERATED_FILE"
```

**Step 2: kotlin-release.yml의 generate-bindings 잡에 패치 단계 추가**

`generate-bindings` 잡의 "Generate Kotlin bindings" 스텝 뒤에 추가:

```yaml
      - name: Patch generated Kotlin for Kotlin 2.x compatibility
        run: |
          chmod +x bindings/scripts/patch-generated-kotlin.sh
          bindings/scripts/patch-generated-kotlin.sh generated-kotlin/io/clroot/slimg/slimg_ffi.kt
```

**Step 3: Commit**

```bash
git add bindings/scripts/patch-generated-kotlin.sh
git commit -m "fix(kotlin): add patch script for UniFFI Kotlin 2.x message conflict"
```

---

## Summary

| Task | Description | Depends On |
|------|-------------|------------|
| 1 | Gradle signing + Maven Central config | - |
| 2 | kotlin-release.yml 워크플로우 | 1 |
| 3 | Cross.toml (Linux aarch64) | 2 |
| 4 | UniFFI 생성 코드 자동 패치 스크립트 | 2 |

## Required GitHub Secrets

릴리즈 전에 사용자가 GitHub repository settings에서 설정해야 할 시크릿:

| Secret | Description |
|--------|-------------|
| `MAVEN_CENTRAL_USERNAME` | Sonatype OSSRH 사용자명 |
| `MAVEN_CENTRAL_PASSWORD` | Sonatype OSSRH 비밀번호 |
| `GPG_SIGNING_KEY` | GPG 비밀키 (armor 형식) |
| `GPG_SIGNING_PASSWORD` | GPG 키 비밀번호 |
