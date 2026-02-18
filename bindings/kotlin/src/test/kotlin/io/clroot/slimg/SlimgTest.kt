package io.clroot.slimg

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlin.test.assertFailsWith

class SlimgTest {

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
    fun `formatCanEncode returns false only for JXL`() {
        assertTrue(formatCanEncode(Format.JPEG))
        assertTrue(formatCanEncode(Format.PNG))
        assertTrue(formatCanEncode(Format.WEB_P))
        assertTrue(formatCanEncode(Format.AVIF))
        assertFalse(formatCanEncode(Format.JXL))
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
}
