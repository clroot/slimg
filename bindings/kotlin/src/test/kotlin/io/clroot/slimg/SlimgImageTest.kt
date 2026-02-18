package io.clroot.slimg

import java.io.ByteArrayInputStream
import java.nio.ByteBuffer
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class SlimgImageTest {

    private fun testImage(width: Int = 8, height: Int = 8): ImageData {
        val data = ByteArray(width * height * 4) { 0xFF.toByte() }
        return ImageData(width.toUInt(), height.toUInt(), data)
    }

    private fun pngBytes(width: Int = 8, height: Int = 8): ByteArray =
        Slimg.convert(testImage(width, height), Format.PNG).data

    // ── Factory ─────────────────────────────────────────

    @Test
    fun `decode ByteArray preserves format`() {
        val img = SlimgImage.decode(pngBytes())
        assertEquals(Format.PNG, img.format)
        assertEquals(8, img.width)
        assertEquals(8, img.height)
    }

    @Test
    fun `decode InputStream`() {
        val img = SlimgImage.decode(ByteArrayInputStream(pngBytes()))
        assertEquals(Format.PNG, img.format)
    }

    @Test
    fun `decode ByteBuffer`() {
        val img = SlimgImage.decode(ByteBuffer.wrap(pngBytes()))
        assertEquals(Format.PNG, img.format)
    }

    @Test
    fun `from wraps existing ImageData`() {
        val data = testImage()
        val img = SlimgImage.from(data, Format.WEB_P)
        assertEquals(Format.WEB_P, img.format)
        assertEquals(8, img.width)
    }

    @Test
    fun `from without format`() {
        val img = SlimgImage.from(testImage())
        assertEquals(null, img.format)
    }

    // ── Resize ──────────────────────────────────────────

    @Test
    fun `resize by width`() {
        val img = SlimgImage.decode(pngBytes(16, 16))
            .resize(width = 8)
        assertEquals(8, img.width)
        assertEquals(8, img.height)
    }

    @Test
    fun `resize by height`() {
        val img = SlimgImage.decode(pngBytes(16, 16))
            .resize(height = 8)
        assertEquals(8, img.width)
        assertEquals(8, img.height)
    }

    @Test
    fun `resize exact`() {
        val img = SlimgImage.decode(pngBytes(16, 16))
            .resize(width = 12, height = 8)
        assertEquals(12, img.width)
        assertEquals(8, img.height)
    }

    @Test
    fun `resize fit`() {
        val img = SlimgImage.decode(pngBytes(16, 8))
            .resize(fit = 10 to 10)
        assertTrue(img.width <= 10)
        assertTrue(img.height <= 10)
    }

    @Test
    fun `resize scale`() {
        val img = SlimgImage.decode(pngBytes(16, 16))
            .resize(scale = 0.5)
        assertEquals(8, img.width)
        assertEquals(8, img.height)
    }

    @Test
    fun `resize no args throws`() {
        val img = SlimgImage.decode(pngBytes())
        assertFailsWith<IllegalArgumentException> {
            img.resize()
        }
    }

    // ── Crop ────────────────────────────────────────────

    @Test
    fun `crop region`() {
        val img = SlimgImage.decode(pngBytes(16, 16))
            .crop(x = 0, y = 0, width = 8, height = 4)
        assertEquals(8, img.width)
        assertEquals(4, img.height)
    }

    @Test
    fun `crop aspect ratio`() {
        val img = SlimgImage.decode(pngBytes(16, 8))
            .crop(aspectRatio = 1 to 1)
        assertEquals(img.width, img.height)
    }

    // ── Extend ──────────────────────────────────────────

    @Test
    fun `extend to size`() {
        val img = SlimgImage.decode(pngBytes(8, 8))
            .extend(16, 16)
        assertEquals(16, img.width)
        assertEquals(16, img.height)
    }

    @Test
    fun `extend to aspect ratio`() {
        val img = SlimgImage.decode(pngBytes(8, 8))
            .extend(aspectRatio = 2 to 1)
        assertEquals(2.0, img.width.toDouble() / img.height, 0.01)
    }

    @Test
    fun `extend with solid color`() {
        val img = SlimgImage.decode(pngBytes(4, 4))
            .extend(8, 8, fill = Slimg.solidColor(255, 0, 0))
        assertEquals(8, img.width)
    }

    // ── Encode ──────────────────────────────────────────

    @Test
    fun `encode to format`() {
        val result = SlimgImage.decode(pngBytes())
            .encode(Format.WEB_P, quality = 85)
        assertEquals(Format.WEB_P, result.format)
        assertTrue(result.data.isNotEmpty())
    }

    @Test
    fun `optimize re-encodes in source format`() {
        val result = SlimgImage.decode(pngBytes())
            .optimize(quality = 60)
        assertEquals(Format.PNG, result.format)
    }

    @Test
    fun `optimize throws when format unknown`() {
        val img = SlimgImage.from(testImage())
        assertFailsWith<IllegalStateException> {
            img.optimize()
        }
    }

    // ── Chaining ────────────────────────────────────────

    @Test
    fun `full pipeline chain`() {
        val result = SlimgImage.decode(pngBytes(16, 16))
            .resize(width = 12)
            .crop(aspectRatio = 1 to 1)
            .encode(Format.WEB_P, quality = 85)

        assertEquals(Format.WEB_P, result.format)
        assertTrue(result.data.isNotEmpty())
    }

    @Test
    fun `chain preserves source format`() {
        val img = SlimgImage.decode(pngBytes(16, 16))
            .resize(width = 8)
            .crop(aspectRatio = 1 to 1)

        assertEquals(Format.PNG, img.format)
    }

    @Test
    fun `chain does not mutate original`() {
        val original = SlimgImage.decode(pngBytes(16, 16))
        val resized = original.resize(width = 8)

        assertEquals(16, original.width)
        assertEquals(8, resized.width)
    }

    @Test
    fun `decode stream resize extend encode`() {
        val result = SlimgImage.decode(ByteArrayInputStream(pngBytes(10, 10)))
            .resize(width = 8)
            .extend(aspectRatio = 16 to 9, fill = Slimg.solidColor(0, 0, 0))
            .encode(Format.PNG, quality = 90)

        val decoded = Slimg.decode(result.data)
        assertEquals(Format.PNG, decoded.format)
    }
}
