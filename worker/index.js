// Import the wasm module and WASM binary
import init, { fetch } from './pkg/polyjuice_worker.js';
import wasmBinary from './pkg/polyjuice_worker_bg.wasm';

// Initialize WASM with the binary data
await init(wasmBinary);

// Export the fetch handler for Cloudflare Workers
export default {
  async fetch(request, env, ctx) {
    return await fetch(request, env, ctx);
  }
};

