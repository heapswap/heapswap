import init from "./pkg/heapswap_browser.js"
export * from "./pkg/heapswap_browser.js"
export { init }

const encoder = new TextEncoder()
export const fromString = (str: string) => encoder.encode(str)
const decoder = new TextDecoder()
export const toString = (buf: Uint8Array) => decoder.decode(buf)
