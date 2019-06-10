export { STDIN, STDOUT, toUint8Array }

const STDIN = 0;
const STDOUT = 1;

// Unfortunately there's not yet an AssemblyScript library function for converting between arrays
// and typed arrays.
function toUint8Array(array: Array<u8>): Uint8Array {
    let typedArray = new Uint8Array(array.length);
    for (let i = 0; i < array.length; i++) {
        typedArray[i] = unchecked(array[i]);
    }
    return typedArray;
}
