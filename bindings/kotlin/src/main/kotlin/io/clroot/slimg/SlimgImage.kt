package io.clroot.slimg

import java.io.InputStream
import java.nio.ByteBuffer

/**
 * Fluent wrapper around [ImageData] for chaining image operations.
 *
 * ```kotlin
 * val result = SlimgImage.decode(bytes)
 *     .resize(width = 800)
 *     .crop(aspectRatio = 16 to 9)
 *     .encode(Format.WEB_P, quality = 85)
 * ```
 *
 * Each transformation returns a new [SlimgImage]; the original is never mutated.
 */
class SlimgImage private constructor(
    val imageData: ImageData,
    val format: Format?,
) {
    val width: Int get() = imageData.width.toInt()
    val height: Int get() = imageData.height.toInt()

    // ── Factory ─────────────────────────────────────────

    companion object {
        @Throws(SlimgException::class)
        fun decode(data: ByteArray): SlimgImage =
            io.clroot.slimg.decode(data).toSlimgImage()

        @Throws(SlimgException::class)
        fun decode(stream: InputStream): SlimgImage =
            io.clroot.slimg.decode(stream).toSlimgImage()

        @Throws(SlimgException::class)
        fun decode(buffer: ByteBuffer): SlimgImage =
            io.clroot.slimg.decode(buffer).toSlimgImage()

        @Throws(SlimgException::class)
        fun decodeFile(path: String): SlimgImage =
            io.clroot.slimg.decodeFile(path).toSlimgImage()

        fun from(imageData: ImageData, format: Format? = null): SlimgImage =
            SlimgImage(imageData, format)

        private fun DecodeResult.toSlimgImage() = SlimgImage(image, format)
    }

    // ── Resize ──────────────────────────────────────────

    /**
     * Resize the image.
     *
     * - `resize(width = 800)` — scale to width, preserve aspect ratio
     * - `resize(height = 600)` — scale to height, preserve aspect ratio
     * - `resize(width = 800, height = 600)` — exact dimensions (may distort)
     */
    @Throws(SlimgException::class)
    fun resize(width: Int? = null, height: Int? = null): SlimgImage {
        val mode = when {
            width != null && height != null -> ResizeMode.Exact(width.toUInt(), height.toUInt())
            width != null -> ResizeMode.Width(width.toUInt())
            height != null -> ResizeMode.Height(height.toUInt())
            else -> throw IllegalArgumentException("At least one of width or height must be specified")
        }
        return transformed(io.clroot.slimg.resize(imageData, mode))
    }

    /**
     * Fit within bounds, preserving aspect ratio.
     *
     * ```kotlin
     * image.resize(fit = 1200 to 1200)
     * ```
     */
    @Throws(SlimgException::class)
    fun resize(fit: Pair<Int, Int>): SlimgImage {
        val mode = ResizeMode.Fit(fit.first.toUInt(), fit.second.toUInt())
        return transformed(io.clroot.slimg.resize(imageData, mode))
    }

    /**
     * Scale by a factor (e.g. `0.5` = half size).
     */
    @Throws(SlimgException::class)
    fun resize(scale: Double): SlimgImage =
        transformed(io.clroot.slimg.resize(imageData, ResizeMode.Scale(scale)))

    // ── Crop ────────────────────────────────────────────

    /**
     * Extract a specific region.
     */
    @Throws(SlimgException::class)
    fun crop(x: Int, y: Int, width: Int, height: Int): SlimgImage {
        val mode = CropMode.Region(x.toUInt(), y.toUInt(), width.toUInt(), height.toUInt())
        return transformed(io.clroot.slimg.crop(imageData, mode))
    }

    /**
     * Crop to an aspect ratio (centered).
     *
     * ```kotlin
     * image.crop(aspectRatio = 16 to 9)
     * ```
     */
    @Throws(SlimgException::class)
    fun crop(aspectRatio: Pair<Int, Int>): SlimgImage {
        val mode = CropMode.AspectRatio(aspectRatio.first.toUInt(), aspectRatio.second.toUInt())
        return transformed(io.clroot.slimg.crop(imageData, mode))
    }

    // ── Extend ──────────────────────────────────────────

    /**
     * Extend the canvas to exact pixel dimensions (centered).
     */
    @Throws(SlimgException::class)
    fun extend(
        width: Int,
        height: Int,
        fill: FillColor = FillColor.Transparent,
    ): SlimgImage {
        val mode = ExtendMode.Size(width.toUInt(), height.toUInt())
        return transformed(io.clroot.slimg.extend(imageData, mode, fill))
    }

    /**
     * Extend the canvas to fit an aspect ratio (centered).
     *
     * ```kotlin
     * image.extend(aspectRatio = 16 to 9, fill = Slimg.solidColor(255, 255, 255))
     * ```
     */
    @Throws(SlimgException::class)
    fun extend(
        aspectRatio: Pair<Int, Int>,
        fill: FillColor = FillColor.Transparent,
    ): SlimgImage {
        val mode = ExtendMode.AspectRatio(aspectRatio.first.toUInt(), aspectRatio.second.toUInt())
        return transformed(io.clroot.slimg.extend(imageData, mode, fill))
    }

    // ── Encode ──────────────────────────────────────────

    /**
     * Encode the current image into the given [format].
     */
    @Throws(SlimgException::class)
    fun encode(format: Format, quality: Int = 80): PipelineResult =
        Slimg.convert(imageData, format, quality)

    /**
     * Re-encode in the original format detected during [decode].
     *
     * @throws IllegalStateException if the source format is unknown.
     */
    @Throws(SlimgException::class)
    fun optimize(quality: Int = 80): PipelineResult {
        val fmt = format
            ?: throw IllegalStateException("Source format unknown. Use encode(format, quality) instead.")
        return encode(fmt, quality)
    }

    private fun transformed(newData: ImageData) = SlimgImage(newData, format)
}
