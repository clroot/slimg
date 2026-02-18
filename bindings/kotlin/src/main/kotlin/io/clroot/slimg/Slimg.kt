package io.clroot.slimg

import java.io.InputStream
import java.nio.ByteBuffer

/**
 * Ergonomic wrapper around the slimg FFI functions.
 *
 * Eliminates [UByte]/[UInt] boilerplate and provides sensible defaults.
 *
 * ```kotlin
 * val result = Slimg.decode(imageBytes)
 * val optimized = Slimg.optimize(imageBytes, quality = 80)
 * val converted = Slimg.convert(result.image, Format.WEB_P, quality = 85)
 * ```
 */
object Slimg {

    // ── Decode ──────────────────────────────────────────

    @Throws(SlimgException::class)
    fun decode(data: ByteArray): DecodeResult =
        io.clroot.slimg.decode(data)

    @Throws(SlimgException::class)
    fun decode(stream: InputStream): DecodeResult =
        io.clroot.slimg.decode(stream)

    @Throws(SlimgException::class)
    fun decode(buffer: ByteBuffer): DecodeResult =
        io.clroot.slimg.decode(buffer)

    @Throws(SlimgException::class)
    fun decodeFile(path: String): DecodeResult =
        io.clroot.slimg.decodeFile(path)

    // ── Optimize ────────────────────────────────────────

    @Throws(SlimgException::class)
    fun optimize(data: ByteArray, quality: Int = 80): PipelineResult =
        io.clroot.slimg.optimize(data, quality.toQuality())

    @Throws(SlimgException::class)
    fun optimize(stream: InputStream, quality: Int = 80): PipelineResult =
        io.clroot.slimg.optimize(stream, quality.toQuality())

    @Throws(SlimgException::class)
    fun optimize(buffer: ByteBuffer, quality: Int = 80): PipelineResult =
        io.clroot.slimg.optimize(buffer, quality.toQuality())

    // ── Convert ─────────────────────────────────────────

    /**
     * Encode [image] into [format] with optional transformations.
     *
     * ```kotlin
     * Slimg.convert(image, Format.WEB_P, quality = 85, resize = ResizeMode.Width(800u))
     * ```
     */
    @Throws(SlimgException::class)
    fun convert(
        image: ImageData,
        format: Format,
        quality: Int = 80,
        resize: ResizeMode? = null,
        crop: CropMode? = null,
        extend: ExtendMode? = null,
        fillColor: FillColor? = null,
    ): PipelineResult = io.clroot.slimg.convert(
        image,
        PipelineOptions(format, quality.toQuality(), resize, crop, extend, fillColor),
    )

    // ── Image Operations ────────────────────────────────

    @Throws(SlimgException::class)
    fun crop(image: ImageData, mode: CropMode): ImageData =
        io.clroot.slimg.crop(image, mode)

    @Throws(SlimgException::class)
    fun resize(image: ImageData, mode: ResizeMode): ImageData =
        io.clroot.slimg.resize(image, mode)

    @Throws(SlimgException::class)
    fun extend(
        image: ImageData,
        mode: ExtendMode,
        fill: FillColor = FillColor.Transparent,
    ): ImageData = io.clroot.slimg.extend(image, mode, fill)

    // ── Format Utilities ────────────────────────────────

    fun formatExtension(format: Format): String =
        io.clroot.slimg.formatExtension(format)

    fun formatCanEncode(format: Format): Boolean =
        io.clroot.slimg.formatCanEncode(format)

    fun formatFromExtension(path: String): Format? =
        io.clroot.slimg.formatFromExtension(path)

    fun formatFromMagicBytes(data: ByteArray): Format? =
        io.clroot.slimg.formatFromMagicBytes(data)

    fun formatFromMagicBytes(stream: InputStream): Format? =
        io.clroot.slimg.formatFromMagicBytes(stream)

    fun formatFromMagicBytes(buffer: ByteBuffer): Format? =
        io.clroot.slimg.formatFromMagicBytes(buffer)

    fun outputPath(input: String, format: Format, output: String? = null): String =
        io.clroot.slimg.outputPath(input, format, output)

    // ── Helpers ─────────────────────────────────────────

    /**
     * Create a solid [FillColor] from integer RGBA values (0–255).
     */
    fun solidColor(r: Int, g: Int, b: Int, a: Int = 255): FillColor =
        FillColor.Solid(r.toUByte(), g.toUByte(), b.toUByte(), a.toUByte())
}

private fun Int.toQuality(): UByte = coerceIn(0, 100).toUByte()
