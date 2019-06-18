# WebAssembly outside the web

Workshop presented at Altitude London 2019.

WebAssembly is a W3C standard for compiling languages like C, C++, and Rust into modules that can run in web browsers at near-native speed. In order to safely run untrusted code in browsers, WebAssembly was designed from the start to be memory-safe and sandboxed, as well as to only use resources explicitly provided by the runtime environment. In this workshop, we'll see how Fastly's open-source Lucet project can provide similar benefits for server-side applications.

After introducing the core principles of WebAssembly, we'll build an application that uses WebAssembly to cryptographically hash messages in the browser. Then, we'll use the same WebAssembly cryptography module in a server-like command-line environment using Lucet and the POSIX-like WebAssembly System Interface (WASI). We'll show both how the WebAssembly module is implemented using AssemblyScript (a dialect of TypeScript), and how that module fits into usual client- and server-side development workflows.

## SHA512 Examples

The goal of each example in this workshop is to run the SHA512 cryptographic hashing algorithm in WebAssembly on some input message. The examples differ in what environment the WebAssembly runs in, and how the message and the resulting hash move in between the host and the WebAssembly sandbox.

In each example, the implementation of SHA512 is from Frank Denis's [`wasm-crypto` library](https://github.com/jedisct1/wasm-crypto), which is implemented in [AssemblyScript](https://assemblyscript.org/), a dialect of TypeScript.

By default, each example only prints the character `👋`. The step-by-step instructions below explain how to complete the program to correctly compute the hash, and there are also files like `index.solution.js` and `main.solution.rs` that contain the completed programs.

### sha512-web

This example is a Webpack project that embeds the SHA512 program in the browser. Please do not judge the UI too harshly; I am only a compiler engineer 😉.

### sha512-lucet

This example embeds SHA512 as a command-line program using the Lucet runtime's API in Rust. The message is read from standard input by the Rust standard library, which also prints the resulting hash.

### sha512-wasi

This example also embeds SHA512 as a command-line program, but the input/output is handled within the WebAssembly sandbox via the [WebAssembly System Interface (WASI)](https://wasi.dev).

## Prerequisites

This repository contains submodules, so make sure to clone it with the `--recursive` flag:

```shell
$ git clone --recursive https://github.com/fastly/wasm-workshop-altitude-ldn-2019.git
```

If you've already cloned without that option, you can instead initialize the submodules recursively.

```shell
$ git submodule update --init --recursive
```

For best results running the examples, use the Docker image that accompanies this workshop. From the root of this repository, run:

```shell
$ docker pull acfoltzer/wasm-workshop-altitude-ldn-2019:latest
$ ./run_devenv.sh
```

This will give you a terminal with Rust and the Lucet command-line tools preinstalled, along with Node and NPM for the web example.

## Step-by-step

First, start the workshop Docker container if you haven't already:

```shell
$ ./run_devenv.sh
```

### sha512-web

Let's start with the web example by starting a development server from within the Docker container:

```shell
# cd /workshop/sha512-web
# npm install
# npm run start-docker
```

Then, in a browser on your host machine, open <http://localhost:8080>. You should see two text areas, the bottom of which should contain `👋`.

This Webpack/NPM project contains both the typical web assets for this page, like `src/index.js` and `src/index.html`, but also the AssemblyScript SHA512 implementation in `src/assembly`. Take a look at `src/index.js` to see how we are downloading and invoking the `hello()` function exported by `src/assembly/index.ts`.

Webpack provides us with the path to the compiled `module.wasm` file. We load it asynchronously using `instantiateStreaming()` for streaming compilation, and then call `hello()` whenever the input in the upper text area changes.

It's worth noting that we also have to pass `importObject` when creating the instance in order to satisfy the requirements of the WebAssembly module. Since this is a fairly simple program, the only things we need to provide are a `WebAssembly.Memory` instance for the program heap, and an `abort` function which will be called if something unexpected happens, like the WebAssembly program running out of memory.

With the instance created, our task is to take the contents of the upper text area and pass the UTF-8 encoding of that string into the `sha512()` function exposed by the WebAssembly module. Then, we need to get the `string` result of that function, and display it in the lower text area.

Unfortunately, it is much simpler to pass a simple scalar value (like the character code for 👋) to and from WebAssembly than it is to handle compound types like arrays, strings, and objects. The only primitive types in WebAssembly are 32- and 64-bit integers (signed and unsigned) and floating-point values. The definition of any more complicated types are left up to the language compiler's WebAssembly target.

Fortunately, AssemblyScript includes a loader library which helps us move these types back and forth. It is already being used in `src/index.js` to instantiate the WebAssembly module, but we will use it to move the message into the WebAssembly heap, and to retrieve the string containing the final hash. In `updateHash()`, which already has `message` decoded into an array of bytes, we copy the array into the WebAssembly heap:

```js
const msg = new TextEncoder().encode(msgInput.value);
const msgPtr = instance.newArray(msg);
```

Note that `newArray()` doesn't return a JavaScript array, instead it returns a pointer into the WebAssembly heap, represented in JavaScript as a number. Next, we need to invoke the hash function, which looks like:

```typescript
export function sha512(message: Uint8Array): string {
    return bin2hex(hash(message));
}
```

When an AssemblyScript function takes a compound type like `Uint8Array` as an argument, the compiled code actually takes a pointer to that type when we invoke them from the host. So, we can pass the pointer we just received into the hash function:

```js
const hashStrPtr = instance.sha512(msgPtr);
```

As you might have guessed, when returning compound types to the host, we are actually returning a pointer to that value, in this case the string containing the hex digits of the hash. So, we need to call another helper function from the loader to actually convert that pointer into a JavaScript string we can put into a text area:

```js
const hashStr = instance.getString(hashStrPtr);
hashOutput.value = hashStr;
```

If you save `src/index.js` with these changes, your browser window should reload with the results of the hash function in place.

As you can see, dealing with marshaling like this, rather than more seamlessly using the host's native types, is one of the biggest pain points for dealing with WebAssembly at the moment. Using WebIDL to generate JavaScript code to handle this is a topic of ongoing work in the WebAssembly CG.

### sha512-lucet

Next, we'll go outside the browser by compiling the same WebAssembly module to native code using Lucet. We'll use the Lucet runtime library from Rust to load the module, pass in arguments, run the hash function, and read the result. But first, we wave hello again.

**Note**: make sure you have followed the steps for `sha512-web` first, or have at least run `npm run asbuild` in that directory.

The interface to our program will be similar to other command line tools like `sha512sum`: we'll take in a message from standard input, and then output its hash to the console. We pipe in `echo -n ''` to give the program an empty message to start with:

```shell
# cd /workshop/sha512-lucet
# echo -n '' | cargo run
...
👋
```

Taking a look at `src/main.rs`, we see much of the same basic structure as in our JavaScript embedding. We start by reading in the contents of standard input, then load the compiled Lucet module `module.so` from disk. We create an `mmap`-backed memory region for the WebAssembly heap, stack, and the Lucet metadata, and then instantiate the loaded module in that region. Again, we run the `hello()` function, and then print the decoded character. We've made the Lucet compilation process automatic for this example, but if you are curious how we use the Lucet compiler to produce `module.so`, see `build.rs`. It can also be used from the command line, which we'll see for `sha512-wasi`.

Like when we instantiate the module in JavaScript, we have to provide all of the functions the module expects from its environment. We've implicitly provided the memory by using `MmapRegion`, but the `abort` must be explicitly provided as an exported, dynamic symbol. When Lucet compiles the module, it uses the mapping in `bindings.json` to determine which symbol each WebAssembly import name maps to; in this case `env::abort` maps to `__as_abort`, which we define at the bottom of `main.rs`. Implementing new hostcalls like this one are beyond the scope of this workshop, but this is the mechanism that allows Lucet hosts to offer advanced functionality to WebAssembly programs; think of them like system calls in a traditional operating system.

To complete our program, though, we again have to deal with the fact that we can't straightforwardly move complex types between the host environment and the WebAssembly sandbox. We've written the marshaling functions needed for this program in `src/assemblyscript.rs`. Take a look for an example of how the Lucet APIs can be used by the host to manipulate WebAssembly's memory, though know that this code is simplified and lacks proper error handling.

Again, we begin modifying the program by copying the message bytes into the WebAssembly instance, which again yields a WebAssembly pointer:

```rust
let message_ptr = inst.put_byte_slice(&message);
```

We pass this pointer to the hash function, which requires a bit more typing than JavaScript because we have to be explicit about the types:

```rust
let hash_str_ptr = inst
    .run("sha512", &[Val::GuestPtr(message_ptr)])
    .unwrap()
    .into();
```

Finally, we read the hash string out of the instance using the pointer returned by `sha512()`, and print it to the console:

```rust
let hash_str = inst.get_string(hash_str_ptr);
println!("{}", hash_str);
```

Now if we run the program, we should get the hash of the empty message:

```shell
# echo -n '' | cargo run
...
cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e
```

### sha512-wasi

The `sha512-web` and `sha512-lucet` examples both have the same basic structure. The host language is used to collect the message input and display the results; the WebAssembly module is just used to run the hash function, and convert the result to a string. In this example, we use the [WebAssembly System Interface (WASI)](https://wasi.dev) to write all of the program logic in WebAssembly instead.

WASI is an experimental standard interface for C-like WebAssembly programs that is inspired by POSIX. It defines a set of hostcalls for operations on files, directories, and streams, as well as command-line arguments, environment variables, clocks, and random number generation functions. Language implementors can port their standard libraries to use these hostcalls, allowing many command-line programs to work without modification in WebAssembly.

For this example, we are only making use of WASI's ability to read from and write to streams, including standard input and output. In `assembly/index.ts`, the entrypoint for our program is `_start()`; think of this like `main()` in a normal C program. In `_start()`, we use the `IO` class from the AssemblyScript WASI bindings to write a string to `stdout`.

To run this program, we don't create a new project that uses the Lucet runtime library. Instead, since WASI is a standardized set of system calls, we can use a single host program, `lucet-wasi`, to run any WASI program.

First, we will compile the AssemblyScript project into a `.wasm` file, and then use `lucetc`, the Lucet compiler, to compile that into native code:

```shell
# cd /workshop/sha512-wasi
# npm install
# npm run asbuild
# lucetc dist/module.wasm -o dist/module.so --bindings bindings.json
# echo -n 'Hello, Altitude London!' | lucet-wasi dist/module.so
👋
```

Once again, we'll replace the emoji with actually running `sha512()`. This time, though, we don't need to deal with pointers and copying into and out of the WebAssembly sandbox. We only need to convert an `Array<u8>` into a `Uint8Array`, and then run the hash function directly and write out the resulting string

```typescript
export function _start(): void {
    let message = toUint8Array(IO.readAll(STDIN)!);
    let hash = sha512(message);
    IO.writeString(STDOUT, hash, true);
}
```

Now when we recompile and run, we get the hash we expect:

```shell
# npm run asbuild
# lucetc dist/module.wasm -o dist/module.so --bindings bindings.json
# echo -n 'Hello, Altitude London!' | lucet-wasi dist/module.so
c12496513baeb2b100473a92d73832837e2be07e07d5d9f5449f9a9767d5e90ce81db4db254aff7bfa64fca290ddfd701f36ac96d9815b8b1b49bdf537d2acd8
```

## Next steps

If you're interested in learning more, check out the [Lucet repository on GitHub](https://github.com/fastly/lucet). If you've followed the step-by-step instructions, you already have the repository checked out in `/workshop/lib/lucet`. It contains the sources for `lucetc`, `lucet-wasi`, and the runtime libraries for C and Rust hosts. There is also a growing wiki with documentation, and example projects and benchmarking suites.

If you have questions, please drop us a line at <labs@fastly.com> or open an issue on this or the Lucet repository. 👋
