package io.clroot.slimg

import java.io.InputStream
import java.nio.ByteBuffer

// ── InputStream overloads ───────────────────────

/**
 * Decode an image from an [InputStream].
 *
 * Reads all bytes from the stream and delegates to [decode].
 * The caller is responsible for closing the stream.
 */
@Throws(SlimgException::class)
fun decode(stream: InputStream): DecodeResult =
    decode(stream.readBytes())

/**
 * Re-encode an image read from an [InputStream] at the given [quality].
 *
 * Reads all bytes from the stream and delegates to [optimize].
 * The caller is responsible for closing the stream.
 */
@Throws(SlimgException::class)
fun optimize(stream: InputStream, quality: UByte): PipelineResult =
    optimize(stream.readBytes(), quality)

/**
 * Detect the image format of data provided by an [InputStream] using magic bytes.
 *
 * Reads all bytes from the stream and delegates to [formatFromMagicBytes].
 * The caller is responsible for closing the stream.
 */
fun formatFromMagicBytes(stream: InputStream): Format? =
    formatFromMagicBytes(stream.readBytes())

// ── ByteBuffer overloads ────────────────────────

/**
 * Decode an image from a [ByteBuffer].
 *
 * Reads the remaining bytes (position to limit) without modifying the buffer's state.
 */
@Throws(SlimgException::class)
fun decode(buffer: ByteBuffer): DecodeResult =
    decode(buffer.toByteArray())

/**
 * Re-encode an image from a [ByteBuffer] at the given [quality].
 *
 * Reads the remaining bytes (position to limit) without modifying the buffer's state.
 */
@Throws(SlimgException::class)
fun optimize(buffer: ByteBuffer, quality: UByte): PipelineResult =
    optimize(buffer.toByteArray(), quality)

/**
 * Detect the image format of data in a [ByteBuffer] using magic bytes.
 *
 * Reads the remaining bytes (position to limit) without modifying the buffer's state.
 */
fun formatFromMagicBytes(buffer: ByteBuffer): Format? =
    formatFromMagicBytes(buffer.toByteArray())

private fun ByteBuffer.toByteArray(): ByteArray {
    val buf = duplicate()
    val bytes = ByteArray(buf.remaining())
    buf.get(bytes)
    return bytes
}
