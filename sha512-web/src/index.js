import asLoader from 'assemblyscript/lib/loader';
import wasmModulePath from './assembly/module.wasm';

const importObject = {
    env: {
        abort: function(msg, file, line, column) {
            console.error("abort at " + file + ":" + line + ":" + column + ": " + msg);
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

const messageInput = document.getElementById('inputString');

const hashOutput = document.getElementById('outputHash');
hashOutput.readOnly = true;

async function updateHash() {
    return loadModule(wasmModulePath, importObject).then(instance => {
        const message = new TextEncoder().encode(messageInput.value);
        const messagePtr = instance.newArray(message);
        
        const hashStrPtr = instance.sha512(messagePtr);
        instance.freeArray(messagePtr);
        
        hashOutput.value = instance.getString(hashStrPtr);
    });
}

messageInput.addEventListener("input", updateHash);
updateHash();
