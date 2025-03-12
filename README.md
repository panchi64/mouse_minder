# MouseMinder

MouseMinder is a simple utility that tracks your mouse position and allows you to restore it to a saved location with a hotkey. Perfect for screen recording, video editing, or any workflow where you need to return to specific screen positions quickly.

<!-- ![MouseMinder UI](I have to upload the image on GitHub) -->

## Inspiration

This project was inspired by this [YouTube video](https://youtu.be/vlXdUU5pd_0?si=i9fi6_rJrbT00S3j) from [Bog](https://www.youtube.com/@bogxd), who attempted to create a similar application to streamline his screen recording workflow. While Bog's brave attempt to code a Python version hit some roadblocks (check out the video), this Rust implementation aims to provide a relatively reliable solution to the same problem.

## Features

- **Automatic Position Tracking**: Saves mouse position after 2 seconds of inactivity
- **Hotkey Restoration**: Press Ctrl+Shift+R (Windows/Linux) or Cmd+Shift+R (macOS) to restore your cursor to its last saved position
- **Simple Interface**: Clean and simple UI that stays out of your way
- **Cross-Platform**: Works on Windows, macOS, and Linux

## Use Cases

- **Screen Recording**: When recording tutorials or demos, restore your cursor to the exact same position after cutting out mistakes
- **Precision Work**: When working with detailed designs or small UI elements, quickly return to your area of focus
- **Multitasking**: Jump back to important screen positions while working across multiple applications

## Installation

### From Releases

1. Download the appropriate version for your operating system from the [Releases](https://github.com/yourusername/mouse_minder/releases) page
2. Run the executable

### From Source

1. Clone this repository
2. Make sure you have Rust installed
3. Run `cargo build --release`
4. The executable will be available in the `target/release` directory

## Permissions

MouseMinder requires accessibility permissions to:

- Track mouse movements
- Control cursor position
- Detect keyboard shortcuts

You should be prompted to grant these permissions when first running the application.

## Development

Built with the following Rust dependencies:

- [egui](https://github.com/emilk/egui) for UI
- [device_query](https://github.com/ostrosco/device_query) for mouse tracking
- [global-hotkey](https://github.com/tauri-apps/global-hotkey) for keyboard shortcuts
- [enigo](https://github.com/enigo-rs/enigo) for mouse control

## Contributing

Contributions are welcome! Feel free to submit pull requests or open issues for any bugs or feature requests.
