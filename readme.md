# The WASM Elevator

A simulated elevator implemented in Rust+Web-Assembly+Javascript. 

## Features
* **Interactive call buttons**: Click the call buttons to call the elevator to a floor
* **Emergency stop button**: Use the emergency stop button to halt the elevator
* **Direction awareness**: The elevator is direction aware, it will always serve calls in the going direction if possible
* **ETA**: When waiting for a call, see the Estimated Time of Arrival

## Interactive demo
An interactive demo is hosted at [andreaseg.github.io/wasm-lift](https://andreaseg.github.io/wasm-lift/)

## Structure

* **lift**:
A Rust _[no_std](https://docs.rust-embedded.org/book/intro/no-std.html)_ implementation of a simple lift controller. Exposes a generic sensor trait allowing for different implementors to benefit from the same controller. In this project it has been compiled to web-assembly and is displayed as an interactive webpage.

* **lift_wasm**:
Web-assembly implementation modelling a hypotethical lift. Acts as glue between the _lift_ and _www_ modules.

* **www**:
Javascript glue code to interact with the web-assembly module, the canvas and styling for display in addition to handlers for interactivity


Based on examples on [rustwasm.github.io](https://rustwasm.github.io/)