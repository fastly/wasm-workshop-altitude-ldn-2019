import 'allocator/arena';
export { memory };

import { bin2hex, hash } from "../../lib/wasm-crypto/assembly/crypto";
import { IO } from "../../lib/lucet/assemblyscript/modules/wasa/assembly/wasa";

export function sha512(message: Uint8Array): string {
    return bin2hex(hash(message));
}

const stdin = 0;
const stdout = 1;

export function _start(): void {
    IO.writeString(stdout, '👋', true);
}

function toUint8Array(array: Array<u8>): Uint8Array {
    let typedArray = new Uint8Array(array.length);
    for (let i = 0; i < array.length; i++) {
        typedArray[i] = unchecked(array[i]);
    }
    return typedArray;
}
