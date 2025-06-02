/**
 * Convert a base64 string to an ArrayBuffer
 * @param {ArrayBuffer} buffer
 * @returns {string}
 */
export function arrayBufferToBase64(buffer) {
  var binary = "";
  var bytes = new Uint8Array(buffer);
  for (var i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}


/**
 * converts SharedArrayBuffer to ArrayBuffer  
 * @param {ArrayBufferLike} bufferLike
 * @returns {ArrayBuffer}
 */
export function toArrayBuffer(bufferLike) {
  return bufferLike instanceof ArrayBuffer
    ? bufferLike
    : bufferLike instanceof SharedArrayBuffer
      ? new Uint8Array(bufferLike).slice().buffer
      : (() => {
        throw new Error("Unsupported buffer type");
      })();
}