plugins {
    java
    id("com.github.johnrengelman.shadow") version "8.1.1" apply false
    id("io.papermc.paperweight.patcher") version "1.5.4"
}

val paperMavenPublicUrl = "https://repo.papermc.io/repository/maven-public/"

repositories {
    mavenCentral()
    maven(paperMavenPublicUrl) {
        content {
            onlyForConfigurations(configurations.paperclip.name)
        }
    }
}

dependencies {
    decompiler("net.minecraftforge:forgeflower:2.0.627.2")
    remapper("net.fabricmc:tiny-remapper:0.8.6:fat")
    paperclip("io.papermc:paperclip:3.0.3")
}

allprojects {
    apply(plugin = "java")

    java {
        toolchain {
            languageVersion.set(JavaLanguageVersion.of(17))
        }
    }
}

subprojects {
    repositories {
        mavenCentral()
        maven(paperMavenPublicUrl)
    }

    tasks {
        withType<JavaCompile>().configureEach {
            options.encoding = Charsets.UTF_8.name()
            options.release.set(17)
            options.compilerArgs.add("--enable-preview")
        }

        withType<Javadoc>().configureEach {
            options.encoding = Charsets.UTF_8.name()
        }

        @Suppress("UnstableApiUsage")
        withType<ProcessResources>().configureEach {
            filteringCharset = Charsets.UTF_8.name()
        }
    }
}

paperweight {
    serverProject.set(project(":cardstock-server"))

    decompileRepo.set(paperMavenPublicUrl)
    remapRepo.set(paperMavenPublicUrl)

    usePaperUpstream(providers.gradleProperty("paperRef")) {
        withPaperPatcher {
            val projectDir = layout.projectDirectory

            apiPatchDir.set(projectDir.dir("patches/api"))
            apiOutputDir.set(projectDir.dir("cardstock-api"))

            serverPatchDir.set(projectDir.dir("patches/server"))
            serverOutputDir.set(projectDir.dir("cardstock-server"))
        }
    }
}
