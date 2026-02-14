# StickHeroo 🚹️ - Mojo Demonstration

This project serves as a practical demonstration of the [mojo-rust-sdk](https://github.com/inspi-writer001/mojo-rust-sdk-v0), showcasing its capabilities within a simple, turn-based RPG game.

The application is built with Rust and the Leptos framework, and it runs entirely in the browser by compiling to WebAssembly. The primary goal is to illustrate how the SDK can be used to interact with the Solana blockchain for building on-chain or chain-aware applications.

## Core Features

- **SDK Integration:** Demonstrates how to use the `mojo-rust-sdk` for blockchain interactions.
- **Turn-Based Battle:** A simple combat system against an AI opponent.
- **Solana Wallet:** Connect a Phantom wallet to handle transactions.
- **Reactive UI:** The frontend is built with Leptos, a modern Rust framework for reactive web applications.

## Technology Stack

- **Primary SDK:** [mojo-rust-sdk](https://github.com/inspi-writer001/mojo-rust-sdk-v0)
- **Frontend:** Rust compiled to WebAssembly
- **Framework:** [Leptos](https://leptos.dev/)
- **Blockchain:** Solana
- **Build Tool:** [Trunk](https://trunkrs.dev/)

## Getting Started

To run this project locally, you will need Rust and the `trunk` build tool installed.

1.  **Install Rust:**
    Follow the official instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).

2.  **Install Trunk:**

    ```bash
    cargo install --locked trunk
    ```

3.  **Install wasm-bindgen-cli:**

    ```bash
    cargo install wasm-bindgen-cli
    ```

4.  **Add Wasm Target:**

    ```bash
    rustup target add wasm32-unknown-unknown
    ```

5.  **Clone the Repository:**

    ```bash
    git clone <repository-url>
    cd web_demo
    ```

6.  **Run the Development Server:**

    ```bash
    trunk serve
    ```

7.  **Open in Browser:**
    Navigate to `http://127.0.0.1:8080` to view the application.

## Project Structure

- **`Cargo.toml`**: Defines project dependencies, most importantly the `mojo-rust-sdk`.
- **`index.html`**: The main entry point for the web application.
- **`Trunk.toml`**: Configuration for the `trunk` build tool.
- **`src/`**: Contains the main application source code.
  - **`main.rs`**: The entry point of the Rust application.
  - **`app.rs`**: Defines the main application component, including routing and global state.
  - **`game_state.rs`**: Contains the core game logic and state definitions.
  - **`solana_bridge.rs`**: Handles communication between the Rust/WASM code and the browser's Solana wallet.
  - **`wallet.rs`**: Provides wallet-related utilities.
  - **`components/`**: Reusable UI components.
  - **`pages/`**: The different views or pages of the application.
- **`style/`**: Contains the CSS stylesheets.
