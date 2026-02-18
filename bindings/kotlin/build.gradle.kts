plugins {
    kotlin("jvm") version "2.1.0"
    id("com.vanniktech.maven.publish") version "0.30.0"
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

    // Point JNA to the Cargo build output so tests can load libslimg_ffi
    val cargoTargetDir = rootProject.projectDir.resolve("../../target")
    val libPaths = listOf(
        cargoTargetDir.resolve("debug"),
        cargoTargetDir.resolve("release"),
    ).joinToString(File.pathSeparator) { it.absolutePath }
    systemProperty("jna.library.path", libPaths)
}

mavenPublishing {
    publishToMavenCentral(com.vanniktech.maven.publish.SonatypeHost.CENTRAL_PORTAL, automaticRelease = true)
    signAllPublications()

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
