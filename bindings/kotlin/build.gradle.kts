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
    val signingKey = System.getenv("GPG_SIGNING_KEY")
    val signingPassword = System.getenv("GPG_SIGNING_PASSWORD")
    if (signingKey != null && signingPassword != null) {
        useInMemoryPgpKeys(signingKey, signingPassword)
    }
    sign(publishing.publications["maven"])
}

tasks.withType<Sign>().configureEach {
    onlyIf { gradle.taskGraph.hasTask("publish") }
}
