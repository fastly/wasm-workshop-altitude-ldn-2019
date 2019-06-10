import 'allocator/arena';
export { memory };

import { bin2hex, hash } from '../../lib/wasm-crypto/assembly/crypto';
import { IO } from '../../lib/lucet/assemblyscript/modules/wasa/assembly/wasa';
import { STDIN, STDOUT, toUint8Array } from './helpers';

export function sha512(message: Uint8Array): string {
    return bin2hex(hash(message));
}

export function _start(): void {
    let message = toUint8Array(IO.readAll(STDIN)!);
    let hash = sha512(message);
    IO.writeString(STDOUT, hash, true);
}
