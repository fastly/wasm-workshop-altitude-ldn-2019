import asLoader from 'assemblyscript/lib/loader';
import wasmModulePath from './assembly/module.wasm';

const importObject = {
    env: {
        abort: function(_msg, _file, line, column) {
            console.error("abort at " + line + ":" + column);
        },
        memory: new WebAssembly.Memory({ initial: 1})
    }
};

async function loadModule(path, importObject) {
    return asLoader.instantiateStreaming(
        fetch(wasmModulePath),
        importObject
    );
}

const msgInput = document.getElementById('inputString');

const hashOutput = document.getElementById('outputHash');
hashOutput.readOnly = true;

async function updateHash() {
    return loadModule(wasmModulePath, importObject).then(instance => {
        const msg = new TextEncoder().encode(msgInput.value);

        hashOutput.value = String.fromCodePoint(instance.hello());
    });
}

messageInput.addEventListener("input", updateHash);
updateHash();
