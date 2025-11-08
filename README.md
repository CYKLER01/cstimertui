# Rust CSTimer

A terminal-based speedcubing timer written in Rust, inspired by cstimer.net.

## Features

*   **Accurate Timer:** A precise, millisecond-resolution timer for tracking solve times.
*   **Two Timer Modes:**
    *   **Default:** Press Space to start and stop the timer.
    *   **Hold:** Hold Space until the indicator turns yellow, then release to start the timer. Press Space again to stop. (DOES NOT WORK!!)
*   **Session Statistics:** Automatically calculates and displays key statistics for your current session:
    *   Average of 5 (Ao5)
    *   Average of 12 (Ao12)
    *   Best of 12 (Bo12)
    *   Overall Best time
*   **Customizable Display:** Choose from three different visual styles for the timer: `Text`, `Boxes`, or `BoxesRounded`.
*   **Configuration Menu:** An interactive menu to easily change settings on the fly.
*   **Solve History:** View a list of your times from the current session.

## Building and Running

### Building from source

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/cykler01/rustcstimer.git
    cd rustcstimer
    ```

2.  **Build the project:**
    ```bash
    cargo build --release
    ```

3.  **Run the application:**
    *   To start the timer:
        ```bash
        ./target/release/rustcstimer
        ```
    *   To open the settings menu:
        ```bash
        ./target/release/rustcstimer -m
        ```
    *   To see the help message:
        ```bash
        ./target/release/rustcstimer -h
        ```

### Installation

You can install the application to make it available system-wide.

1.  **Install the binary:**
    ```bash
    cargo install --path .
    ```
    This will install the binary to `~/.cargo/bin/rustcstimer`.

2.  **Ensure `~/.cargo/bin` is in your `PATH`:**
    Add the following line to your shell's configuration file (e.g., `~/.bashrc`, `~/.zshrc`):
    ```bash
    export PATH="$HOME/.cargo/bin:$PATH"
    ```

3.  **Run the application:**
    You can now run the application from anywhere using the `rustcstimer` command:
    ```bash
    rustcstimer
    rustcstimer -m
    ```

## How to Use

### Timer Screen
*   Use the `Spacebar` to start and stop the timer according to the selected `Run Option`.
*   Press `Esc` to exit the timer and return to your terminal.
*   Your solve history and statistics are displayed on the left.

### Settings Menu

You can access the settings menu by running the application with the `-m` or `--menu` flag.

```bash
    cargo run -- -m
```

In the menu, you can:

*   Navigate between options using the `Up` and `Down` arrow keys.
*   Change the values of the selected option by pressing `Enter`.
*   Select "Start Timer" to save your changes and begin a session.
*   Select "Exit" or press `Esc` to quit the application.

### Available Settings

*   **Style:** `Text`, `Boxes`, or `BoxesRounded`.
*   **Run Option:** `Default` or `Hold`.
