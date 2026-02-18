package io.clroot.slimg

import java.io.ByteArrayInputStream
import java.nio.ByteBuffer
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class SlimgObjectTest {

    private fun testImage(width: UInt = 8u, height: UInt = 8u): ImageData {
        val data = ByteArray(width.toInt() * height.toInt() * 4) { 0xFF.toByte() }
        return ImageData(width, height, data)
    }

    private fun pngBytes(image: ImageData = testImage()): ByteArray =
        Slimg.convert(image, Format.PNG).data

    // ── Decode ──────────────────────────────────────────

    @Test
    fun `decode ByteArray`() {
        val bytes = pngBytes()
        val result = Slimg.decode(bytes)
        assertEquals(Format.PNG, result.format)
        assertEquals(8u, result.image.width)
    }

    @Test
    fun `decode InputStream`() {
        val bytes = pngBytes()
        val result = Slimg.decode(ByteArrayInputStream(bytes))
        assertEquals(Format.PNG, result.format)
    }

    @Test
    fun `decode ByteBuffer`() {
        val bytes = pngBytes()
        val result = Slimg.decode(ByteBuffer.wrap(bytes))
        assertEquals(Format.PNG, result.format)
    }

    // ── Optimize ────────────────────────────────────────

    @Test
    fun `optimize with Int quality`() {
        val bytes = pngBytes()
        val result = Slimg.optimize(bytes, quality = 60)
        assertEquals(Format.PNG, result.format)
        assertTrue(result.data.isNotEmpty())
    }

    @Test
    fun `optimize default quality`() {
        val bytes = pngBytes()
        val result = Slimg.optimize(bytes)
        assertTrue(result.data.isNotEmpty())
    }

    @Test
    fun `optimize InputStream with Int quality`() {
        val bytes = pngBytes()
        val result = Slimg.optimize(ByteArrayInputStream(bytes), quality = 60)
        assertTrue(result.data.isNotEmpty())
    }

    @Test
    fun `optimize ByteBuffer with Int quality`() {
        val bytes = pngBytes()
        val result = Slimg.optimize(ByteBuffer.wrap(bytes), quality = 60)
        assertTrue(result.data.isNotEmpty())
    }

    @Test
    fun `optimize clamps out-of-range quality`() {
        val bytes = pngBytes()
        // Should not throw - values clamped to 0..100
        Slimg.optimize(bytes, quality = -10)
        Slimg.optimize(bytes, quality = 200)
    }

    // ── Convert ─────────────────────────────────────────

    @Test
    fun `convert with defaults`() {
        val image = testImage()
        val result = Slimg.convert(image, Format.WEB_P)
        assertEquals(Format.WEB_P, result.format)
        assertTrue(result.data.isNotEmpty())
    }

    @Test
    fun `convert with resize`() {
        val image = testImage(16u, 16u)
        val result = Slimg.convert(image, Format.PNG, resize = ResizeMode.Width(8u))
        assertEquals(Format.PNG, result.format)
    }

    @Test
    fun `convert with crop and quality`() {
        val image = testImage(16u, 16u)
        val result = Slimg.convert(
            image,
            Format.PNG,
            quality = 90,
            crop = CropMode.AspectRatio(1u, 1u),
        )
        val decoded = Slimg.decode(result.data)
        assertEquals(decoded.image.width, decoded.image.height)
    }

    // ── Image Operations ────────────────────────────────

    @Test
    fun `crop delegates correctly`() {
        val image = testImage(10u, 10u)
        val cropped = Slimg.crop(image, CropMode.Region(0u, 0u, 5u, 5u))
        assertEquals(5u, cropped.width)
        assertEquals(5u, cropped.height)
    }

    @Test
    fun `resize delegates correctly`() {
        val image = testImage(10u, 10u)
        val resized = Slimg.resize(image, ResizeMode.Width(5u))
        assertEquals(5u, resized.width)
    }

    @Test
    fun `extend with default fill`() {
        val image = testImage(8u, 8u)
        val extended = Slimg.extend(image, ExtendMode.Size(16u, 16u))
        assertEquals(16u, extended.width)
        assertEquals(16u, extended.height)
    }

    // ── solidColor ──────────────────────────────────────

    @Test
    fun `solidColor creates FillColor with Int values`() {
        val fill = Slimg.solidColor(255, 128, 0)
        assertEquals(FillColor.Solid(255u.toUByte(), 128u.toUByte(), 0u.toUByte(), 255u.toUByte()), fill)
    }

    @Test
    fun `solidColor with custom alpha`() {
        val fill = Slimg.solidColor(0, 0, 0, 128)
        assertEquals(FillColor.Solid(0u.toUByte(), 0u.toUByte(), 0u.toUByte(), 128u.toUByte()), fill)
    }

    @Test
    fun `extend with solidColor`() {
        val image = testImage(4u, 4u)
        val extended = Slimg.extend(
            image,
            ExtendMode.Size(8u, 8u),
            Slimg.solidColor(255, 0, 0),
        )
        assertEquals(8u, extended.width)
    }

    // ── Format Utilities ────────────────────────────────

    @Test
    fun `formatFromExtension via wrapper`() {
        assertEquals(Format.PNG, Slimg.formatFromExtension("image.png"))
        assertNull(Slimg.formatFromExtension("file.xyz"))
    }

    @Test
    fun `formatCanEncode via wrapper`() {
        assertTrue(Slimg.formatCanEncode(Format.PNG))
    }

    @Test
    fun `formatFromMagicBytes ByteBuffer via wrapper`() {
        val bytes = pngBytes()
        assertEquals(Format.PNG, Slimg.formatFromMagicBytes(ByteBuffer.wrap(bytes)))
    }
}
