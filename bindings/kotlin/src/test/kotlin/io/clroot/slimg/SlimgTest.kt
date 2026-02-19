package io.clroot.slimg

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlin.test.assertFailsWith
import kotlin.test.assertContentEquals

class SlimgTest {

    // ── Helper ──────────────────────────────────────────

    /**
     * Create a test ImageData (RGBA, 4 bytes per pixel).
     * Each pixel: R = row index, G = col index, B = 0xFF, A = 0xFF.
     */
    private fun createTestImage(width: UInt, height: UInt): ImageData {
        val data = ByteArray(width.toInt() * height.toInt() * 4)
        for (row in 0 until height.toInt()) {
            for (col in 0 until width.toInt()) {
                val offset = (row * width.toInt() + col) * 4
                data[offset] = row.toByte()
                data[offset + 1] = col.toByte()
                data[offset + 2] = 0xFF.toByte()
                data[offset + 3] = 0xFF.toByte()
            }
        }
        return ImageData(width, height, data)
    }

    private fun pixelAt(image: ImageData, col: Int, row: Int): List<Int> {
        val offset = (row * image.width.toInt() + col) * 4
        return (0 until 4).map { image.data[offset + it].toInt() and 0xFF }
    }

    // ── Format Utility Tests ────────────────────────────

    @Test
    fun `formatExtension returns correct extensions`() {
        assertEquals("jpg", formatExtension(Format.JPEG))
        assertEquals("png", formatExtension(Format.PNG))
        assertEquals("webp", formatExtension(Format.WEB_P))
        assertEquals("avif", formatExtension(Format.AVIF))
        assertEquals("jxl", formatExtension(Format.JXL))
        assertEquals("qoi", formatExtension(Format.QOI))
    }

    @Test
    fun `formatCanEncode returns true for all formats`() {
        assertTrue(formatCanEncode(Format.JPEG))
        assertTrue(formatCanEncode(Format.PNG))
        assertTrue(formatCanEncode(Format.WEB_P))
        assertTrue(formatCanEncode(Format.AVIF))
        assertTrue(formatCanEncode(Format.JXL))
        assertTrue(formatCanEncode(Format.QOI))
    }

    @Test
    fun `formatFromExtension detects known formats`() {
        assertEquals(Format.JPEG, formatFromExtension("photo.jpg"))
        assertEquals(Format.JPEG, formatFromExtension("photo.jpeg"))
        assertEquals(Format.PNG, formatFromExtension("image.png"))
        assertEquals(Format.WEB_P, formatFromExtension("image.webp"))
        assertEquals(Format.AVIF, formatFromExtension("image.avif"))
        assertEquals(Format.JXL, formatFromExtension("image.jxl"))
        assertEquals(Format.QOI, formatFromExtension("image.qoi"))
    }

    @Test
    fun `formatFromExtension returns null for unknown`() {
        assertNull(formatFromExtension("file.bmp"))
        assertNull(formatFromExtension("noext"))
    }

    @Test
    fun `formatFromMagicBytes detects JPEG`() {
        val header = byteArrayOf(0xFF.toByte(), 0xD8.toByte(), 0xFF.toByte(), 0xE0.toByte())
        assertEquals(Format.JPEG, formatFromMagicBytes(header))
    }

    @Test
    fun `formatFromMagicBytes detects PNG`() {
        val header = byteArrayOf(
            0x89.toByte(), 0x50, 0x4E, 0x47,
            0x0D, 0x0A, 0x1A, 0x0A
        )
        assertEquals(Format.PNG, formatFromMagicBytes(header))
    }

    @Test
    fun `formatFromMagicBytes returns null for unknown`() {
        assertNull(formatFromMagicBytes(byteArrayOf(0x00, 0x00, 0x00, 0x00)))
    }

    @Test
    fun `outputPath changes extension`() {
        val result = outputPath("/tmp/photo.jpg", Format.WEB_P, null)
        assertEquals("/tmp/photo.webp", result)
    }

    @Test
    fun `outputPath with explicit output`() {
        val result = outputPath("/tmp/photo.jpg", Format.PNG, "/out/result.png")
        assertEquals("/out/result.png", result)
    }

    @Test
    fun `decode rejects unknown format`() {
        assertFailsWith<SlimgException.UnknownFormat> {
            decode(byteArrayOf(0x00, 0x00, 0x00, 0x00))
        }
    }

    // ── Crop Tests ──────────────────────────────────────

    @Test
    fun `crop region returns correct dimensions`() {
        val image = createTestImage(10u, 8u)
        val result = crop(image, CropMode.Region(2u, 1u, 5u, 4u))
        assertEquals(5u, result.width)
        assertEquals(4u, result.height)
    }

    @Test
    fun `crop region preserves pixel data`() {
        val image = createTestImage(10u, 8u)
        val result = crop(image, CropMode.Region(2u, 1u, 3u, 2u))
        // Top-left of cropped = original pixel (col=2, row=1)
        val pixel = pixelAt(result, 0, 0)
        assertEquals(1, pixel[0]) // R = row 1
        assertEquals(2, pixel[1]) // G = col 2
        assertEquals(0xFF, pixel[2]) // B
        assertEquals(0xFF, pixel[3]) // A
    }

    @Test
    fun `crop aspect ratio square from landscape`() {
        val image = createTestImage(10u, 6u)
        val result = crop(image, CropMode.AspectRatio(1u, 1u))
        assertEquals(6u, result.width)
        assertEquals(6u, result.height)
    }

    @Test
    fun `crop aspect ratio 16 to 9`() {
        val image = createTestImage(100u, 100u)
        val result = crop(image, CropMode.AspectRatio(16u, 9u))
        assertEquals(100u, result.width)
        val ratio = result.width.toDouble() / result.height.toDouble()
        assertTrue(ratio in 1.7..1.8, "Expected ~16:9 ratio, got $ratio")
    }

    @Test
    fun `crop region out of bounds throws error`() {
        val image = createTestImage(10u, 8u)
        assertFailsWith<SlimgException.Crop> {
            crop(image, CropMode.Region(8u, 0u, 5u, 4u))
        }
    }

    // ── Extend Tests ────────────────────────────────────

    @Test
    fun `extend aspect ratio to square from landscape`() {
        val image = createTestImage(10u, 6u)
        val result = extend(image, ExtendMode.AspectRatio(1u, 1u), FillColor.Transparent)
        assertEquals(10u, result.width)
        assertEquals(10u, result.height)
    }

    @Test
    fun `extend aspect ratio to square from portrait`() {
        val image = createTestImage(6u, 10u)
        val result = extend(image, ExtendMode.AspectRatio(1u, 1u), FillColor.Transparent)
        assertEquals(10u, result.width)
        assertEquals(10u, result.height)
    }

    @Test
    fun `extend with solid fill color`() {
        val image = createTestImage(4u, 4u)
        val red = FillColor.Solid(
            0xFFu.toUByte(), 0x00u.toUByte(), 0x00u.toUByte(), 0xFFu.toUByte()
        )
        val result = extend(image, ExtendMode.Size(6u, 6u), red)
        assertEquals(6u, result.width)
        assertEquals(6u, result.height)
        // Top-left (0,0) should be the red fill
        val pixel = pixelAt(result, 0, 0)
        assertEquals(0xFF, pixel[0]) // R
        assertEquals(0x00, pixel[1]) // G
        assertEquals(0x00, pixel[2]) // B
        assertEquals(0xFF, pixel[3]) // A
    }

    @Test
    fun `extend with transparent fill`() {
        val image = createTestImage(4u, 4u)
        val result = extend(image, ExtendMode.Size(6u, 6u), FillColor.Transparent)
        // Top-left should be fully transparent
        val pixel = pixelAt(result, 0, 0)
        assertEquals(listOf(0, 0, 0, 0), pixel)
    }

    @Test
    fun `extend preserves original pixel data`() {
        val image = createTestImage(4u, 4u)
        val result = extend(image, ExtendMode.Size(6u, 6u), FillColor.Transparent)
        // 4x4 centered in 6x6 → offset (1,1)
        // Result pixel (1,1) = original pixel (0,0) = R=0, G=0, B=0xFF, A=0xFF
        val pixel = pixelAt(result, 1, 1)
        assertEquals(0, pixel[0])
        assertEquals(0, pixel[1])
        assertEquals(0xFF, pixel[2])
        assertEquals(0xFF, pixel[3])
    }

    @Test
    fun `extend noop when already matching aspect`() {
        val image = createTestImage(10u, 10u)
        val result = extend(image, ExtendMode.AspectRatio(1u, 1u), FillColor.Transparent)
        assertEquals(10u, result.width)
        assertEquals(10u, result.height)
        assertContentEquals(image.data, result.data)
    }

    @Test
    fun `extend size smaller than image throws error`() {
        val image = createTestImage(10u, 8u)
        assertFailsWith<SlimgException.Extend> {
            extend(image, ExtendMode.Size(5u, 8u), FillColor.Transparent)
        }
    }

    // ── Resize Tests ────────────────────────────────────

    @Test
    fun `resize by width preserves aspect ratio`() {
        val image = createTestImage(10u, 8u)
        val result = resize(image, ResizeMode.Width(5u))
        assertEquals(5u, result.width)
        assertEquals(4u, result.height) // 10:8 → 5:4
    }

    @Test
    fun `resize by height preserves aspect ratio`() {
        val image = createTestImage(10u, 8u)
        val result = resize(image, ResizeMode.Height(4u))
        assertEquals(5u, result.width)
        assertEquals(4u, result.height)
    }

    @Test
    fun `resize exact allows non-proportional`() {
        val image = createTestImage(10u, 8u)
        val result = resize(image, ResizeMode.Exact(20u, 20u))
        assertEquals(20u, result.width)
        assertEquals(20u, result.height)
    }

    @Test
    fun `resize fit stays within bounds`() {
        val image = createTestImage(10u, 8u)
        val result = resize(image, ResizeMode.Fit(5u, 5u))
        assertTrue(result.width <= 5u)
        assertTrue(result.height <= 5u)
    }

    @Test
    fun `resize scale doubles dimensions`() {
        val image = createTestImage(10u, 8u)
        val result = resize(image, ResizeMode.Scale(2.0))
        assertEquals(20u, result.width)
        assertEquals(16u, result.height)
    }

    @Test
    fun `resize produces valid RGBA data`() {
        val image = createTestImage(10u, 8u)
        val result = resize(image, ResizeMode.Width(5u))
        assertEquals(result.width.toInt() * result.height.toInt() * 4, result.data.size)
    }

    // ── Convert (Pipeline) Tests ────────────────────────

    @Test
    fun `convert to PNG produces valid result`() {
        val image = createTestImage(4u, 4u)
        val options = PipelineOptions(
            format = Format.PNG,
            quality = 80u.toUByte(),
            resize = null,
            crop = null,
            extend = null,
            fillColor = null,
        )
        val result = convert(image, options)
        assertEquals(Format.PNG, result.format)
        assertTrue(result.data.isNotEmpty())
        // PNG magic: 0x89 P N G
        assertEquals(0x89.toByte(), result.data[0])
        assertEquals(0x50.toByte(), result.data[1])
    }

    @Test
    fun `convert with resize changes dimensions`() {
        val image = createTestImage(10u, 8u)
        val options = PipelineOptions(
            format = Format.PNG,
            quality = 80u.toUByte(),
            resize = ResizeMode.Width(5u),
            crop = null,
            extend = null,
            fillColor = null,
        )
        val result = convert(image, options)
        val decoded = decode(result.data)
        assertEquals(5u, decoded.image.width)
        assertEquals(4u, decoded.image.height)
    }

    @Test
    fun `convert full pipeline crop then extend`() {
        val image = createTestImage(100u, 60u)
        val options = PipelineOptions(
            format = Format.PNG,
            quality = 80u.toUByte(),
            resize = null,
            crop = CropMode.AspectRatio(16u, 9u),
            extend = ExtendMode.AspectRatio(1u, 1u),
            fillColor = FillColor.Solid(
                0xFFu.toUByte(), 0xFFu.toUByte(), 0xFFu.toUByte(), 0xFFu.toUByte()
            ),
        )
        val result = convert(image, options)
        val decoded = decode(result.data)
        // After 16:9 crop then 1:1 extend → square
        assertEquals(decoded.image.width, decoded.image.height)
    }

    @Test
    fun `convert to WebP produces valid result`() {
        val image = createTestImage(8u, 8u)
        val options = PipelineOptions(
            format = Format.WEB_P,
            quality = 75u.toUByte(),
            resize = null,
            crop = null,
            extend = null,
            fillColor = null,
        )
        val result = convert(image, options)
        assertEquals(Format.WEB_P, result.format)
        // WebP magic: "RIFF"
        assertEquals(0x52.toByte(), result.data[0]) // 'R'
        assertEquals(0x49.toByte(), result.data[1]) // 'I'
    }

    // ── Optimize Tests ──────────────────────────────────

    @Test
    fun `optimize re-encodes in same format`() {
        val image = createTestImage(8u, 8u)
        val encoded = convert(image, PipelineOptions(
            format = Format.PNG,
            quality = 80u.toUByte(),
            resize = null,
            crop = null,
            extend = null,
            fillColor = null,
        ))

        val optimized = optimize(encoded.data, 60u.toUByte())
        assertEquals(Format.PNG, optimized.format)
        assertTrue(optimized.data.isNotEmpty())
    }

    // ── Decode Tests ────────────────────────────────────

    @Test
    fun `decode and roundtrip PNG`() {
        val original = createTestImage(6u, 4u)
        val encoded = convert(original, PipelineOptions(
            format = Format.PNG,
            quality = 100u.toUByte(),
            resize = null,
            crop = null,
            extend = null,
            fillColor = null,
        ))

        val decoded = decode(encoded.data)
        assertEquals(Format.PNG, decoded.format)
        assertEquals(6u, decoded.image.width)
        assertEquals(4u, decoded.image.height)
    }
}
