package io.clroot.slimg

import java.io.ByteArrayInputStream
import java.nio.ByteBuffer
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlin.test.assertNull

class SlimgStreamsTest {

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

    private fun encodePng(image: ImageData): ByteArray =
        convert(image, PipelineOptions(
            format = Format.PNG,
            quality = 100u.toUByte(),
            resize = null,
            crop = null,
            extend = null,
            fillColor = null,
        )).data

    // ── decode(InputStream) ─────────────────────────────

    @Test
    fun `decode InputStream returns same result as decode ByteArray`() {
        val image = createTestImage(6u, 4u)
        val pngBytes = encodePng(image)

        val fromBytes = decode(pngBytes)
        val fromStream = decode(ByteArrayInputStream(pngBytes))

        assertEquals(fromBytes.format, fromStream.format)
        assertEquals(fromBytes.image.width, fromStream.image.width)
        assertEquals(fromBytes.image.height, fromStream.image.height)
    }

    // ── optimize(InputStream) ───────────────────────────

    @Test
    fun `optimize InputStream returns same result as optimize ByteArray`() {
        val image = createTestImage(8u, 8u)
        val pngBytes = encodePng(image)

        val fromBytes = optimize(pngBytes, 60u.toUByte())
        val fromStream = optimize(ByteArrayInputStream(pngBytes), 60u.toUByte())

        assertEquals(fromBytes.format, fromStream.format)
        assertTrue(fromStream.data.isNotEmpty())
    }

    // ── formatFromMagicBytes(InputStream) ───────────────

    @Test
    fun `formatFromMagicBytes InputStream detects PNG`() {
        val image = createTestImage(4u, 4u)
        val pngBytes = encodePng(image)

        val format = formatFromMagicBytes(ByteArrayInputStream(pngBytes))
        assertEquals(Format.PNG, format)
    }

    @Test
    fun `formatFromMagicBytes InputStream returns null for unknown data`() {
        val unknownBytes = byteArrayOf(0x00, 0x01, 0x02, 0x03)
        val format = formatFromMagicBytes(ByteArrayInputStream(unknownBytes))
        assertNull(format)
    }

    // ── decode(ByteBuffer) ──────────────────────────────

    @Test
    fun `decode ByteBuffer returns same result as decode ByteArray`() {
        val image = createTestImage(6u, 4u)
        val pngBytes = encodePng(image)

        val fromBytes = decode(pngBytes)
        val fromBuffer = decode(ByteBuffer.wrap(pngBytes))

        assertEquals(fromBytes.format, fromBuffer.format)
        assertEquals(fromBytes.image.width, fromBuffer.image.width)
        assertEquals(fromBytes.image.height, fromBuffer.image.height)
    }

    @Test
    fun `decode ByteBuffer does not modify buffer position`() {
        val pngBytes = encodePng(createTestImage(4u, 4u))
        val buffer = ByteBuffer.wrap(pngBytes)
        val positionBefore = buffer.position()

        decode(buffer)

        assertEquals(positionBefore, buffer.position())
    }

    @Test
    fun `decode direct ByteBuffer works`() {
        val pngBytes = encodePng(createTestImage(4u, 4u))
        val directBuffer = ByteBuffer.allocateDirect(pngBytes.size)
        directBuffer.put(pngBytes)
        directBuffer.flip()

        val result = decode(directBuffer)
        assertEquals(Format.PNG, result.format)
        assertEquals(4u, result.image.width)
    }

    // ── optimize(ByteBuffer) ────────────────────────────

    @Test
    fun `optimize ByteBuffer returns valid result`() {
        val pngBytes = encodePng(createTestImage(8u, 8u))

        val result = optimize(ByteBuffer.wrap(pngBytes), 60u.toUByte())

        assertEquals(Format.PNG, result.format)
        assertTrue(result.data.isNotEmpty())
    }

    // ── formatFromMagicBytes(ByteBuffer) ────────────────

    @Test
    fun `formatFromMagicBytes ByteBuffer detects PNG`() {
        val pngBytes = encodePng(createTestImage(4u, 4u))
        val format = formatFromMagicBytes(ByteBuffer.wrap(pngBytes))
        assertEquals(Format.PNG, format)
    }

    @Test
    fun `formatFromMagicBytes ByteBuffer returns null for unknown data`() {
        val format = formatFromMagicBytes(ByteBuffer.wrap(byteArrayOf(0x00, 0x01, 0x02, 0x03)))
        assertNull(format)
    }
}
