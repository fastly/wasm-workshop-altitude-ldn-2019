import 'allocator/arena';
export { memory };

import { bin2hex, hash } from "../../../lib/wasm-crypto/assembly/crypto";

export function sha512(message: Uint8Array): string {
    return bin2hex(hash(message));
}

export function hello(): i32 {
    return '👋'.codePointAt(0);
}
