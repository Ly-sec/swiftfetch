# swiftfetch

**swiftfetch** is a fast and simple system information fetch tool written in Rust. It displays various system information like OS, CPU, RAM usage, uptime, and more in a neat, customizable format.

## Attention

⚠️ Breaking Changes: `swiftfetch` is quite new so there will be some breaking changes from time to time, for example updates to the config / renaming of config options.
Please always check the default [config](/config/config.toml) for possible changes if something broke for you.

## Features

- Displays Ascii art, essential system information like OS, kernel version, CPU, RAM usage, and more.
- Optional Kitty image mode to replace ASCII art with full-color logos.
- Easy to configure and extend.
- Simple, fast, and lightweight.

## Installation

Using the AUR:

`yay -S swiftfetch` (or any other AUR helper)

To install swiftfetch straight from the repo, use the following command from the root of the project:

`cargo install --path .`

This command will install the program and automatically copy the default config.toml to ~/.config/swiftfetch/config.toml.

# 🧊 Using `swiftfetch` as a Nix Flake

You can use `swiftfetch` as a Nix flake input in your **system configuration**, **Home Manager setup**, or simply run it with `nix run`.

---

## 1. Add `swiftfetch` as a flake input

In your top-level `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    swiftfetch = {
      url = "github:ly-sec/swiftfetch";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, swiftfetch, ... }: {
    # For example, you can forward swiftfetch outputs if needed:
    packages = {
      inherit (swiftfetch.packages.${system}) swiftfetch;
    };
  };
}
```

---

## 2. Use in Home Manager

To add `swiftfetch` to your Home Manager config:

```nix
{
  programs.home-manager.enable = true;

  home.packages = [
    inputs.swiftfetch.packages.${pkgs.system}.swiftfetch
  ];
}
```

> Tip: You may need to pass `inputs` and `pkgs` into your Home Manager module depending on how you’ve structured your flake.

---

## 3. Run directly

You can run `swiftfetch` directly with:

```sh
nix run github:ly-sec/swiftfetch
```

---

## 4. Add to systemPackages

If you’re using NixOS and want to install `swiftfetch` system-wide:

```nix
{
  environment.systemPackages = with pkgs; [
    swiftfetch
  ];
}
```

> Make sure you’ve added the flake input and forwarded the `swiftfetch` package as shown above.

## Configuration

The configuration file can be found at `~/.config/swiftfetch/config.toml`, you can customize it to display or hide specific information based on your preferences.

If you did not set $EDITOR then it will return `nano` as default. Most DEs do that by default afaik, might have to add it yourself for WMs
You can find the default config right [here](/config/config.toml)

## Configuration Options

- `display_mode`: Selects how the left column is rendered. Accepts `ascii` (default) or `image`.
- `image_path`: Path to a PNG/JPEG/WebP file to transmit when `display_mode = "image"`.
- `image_width` / `image_height`: Optional resize hints (pixels). When only one is set the other dimension is calculated to keep the aspect ratio.
- `image_padding_columns` *(optional)*: Override the automatically calculated horizontal spacing (derived from the image width plus a small gap, using your terminal's reported cell size when available). Most users can leave this unset.
- `image_rows` *(optional)*: Override the number of terminal rows the image should occupy. Handy when automatic height detection (based on pixel height ÷ character height) doesn’t match your font size.
- `image_horizontal_offset` *(optional)*: Move the image left/right relative to the system info (measured in terminal columns). Positive values shift it to the right, negative to the left.
- `image_vertical_offset` *(optional)*: Move the image up/down relative to the system info (measured in terminal rows). Positive values shift it downward.
- `ascii_path`: Override the ASCII art file. When unset we fall back to a distro-specific default if available.
- `ascii_color`: Sets the color of the ascii art (or padding spaces when an image is shown).

- `items`: This section defines a list of key-value pairs for the items to be displayed. Each item can have three components:

  - **`key`**: This is the label for the information. For example, it could be "OS", "CPU", "RAM", or any other label you want to display for the corresponding value.
  - **`type`**: Defines the type of content to be displayed. You can choose from the following:
    - `default`: The value is dynamically fetched (e.g., OS name, kernel version).
    - `text`: A custom static value that you define.
    - `command`: Executes a shell command and displays the result.
  - **`value`**: This is the content associated with the key. The content can be a static text, a command to run, or a dynamic value, depending on the `type`.
  - `color`: Sets the color of the key.
  - `value_color`: Sets the color of the value color (aka the part after the separator).

### Kitty image mode

When `display_mode = "image"` the logo is transmitted through the Kitty graphics protocol before the system information is printed. Your terminal must support the protocol (Kitty, WezTerm, Ghostty, Foot, etc.). We auto-detect support (including Ghostty sessions that use `TERM=xterm-256color`), but you can override the check with `SWIFTFETCH_FORCE_KITTY=1`. If transmission fails we quietly fall back to ASCII mode.

Place the image wherever you like (e.g. `~/.config/swiftfetch/logo.png`) and update the path/target size. Spacing between the logo and the text is computed automatically from the rendered width, while the image is nudged down slightly so it lines up with your stats — you can still tweak horizontal spacing with `image_padding_columns` and fine-tune positions with `image_horizontal_offset` / `image_vertical_offset`.

  ### Example of `text` type

  If you want to display a custom message, you can use the `text` type. Here, the `key` will be the label and the `value` will be the custom text:

  ```toml
  [[display.items]]
  key = "Message"
  type = "text"
  value = "Hello, world!"
  ```

  This will output:

  ```
  Message: Hello, world!
  ```

  ### Example of `command` type

  If you want to run a command and display its output, you can use the `command` type. Here, the `key` will be the label and the `value` will be the shell command to execute:

  ```toml
  [[display.items]]
  key = "Free Memory"
  type = "command"
  value = "free -h"
  ```

  This will output the result of the `free -h` command:

  ```
  Free Memory: (output of free -h)
  ```

  ### Empty `key` and `value`

  If you want to add a blank line, leave both the `key` and `value` empty. This will simply create an empty line in the output:

  ```toml
  [[display.items]]
  key = ""
  type = "text"
  value = ""
  ```

  This will output a blank line.

## Example Output

`````

                  -`
                 .o+`                  lysec@archlinux
                `ooo/                  ┌────────────────── System Information ───────────────────┐
               `+oooo:                            󰣇 ‣ os: Arch Linux
              `+oooooo:                           󰍛 ‣ kernel: 6.12.8-2-cachyos
              -+oooooo+:                           ‣ wm: Hyprland
            `/:-:++oooo+:                          ‣ editor: nano
           `/++++/+++++++:                         ‣ shell: fish
          `/++++++++++++++:                        ‣ term: ghostty
         `/+++ooooooooooooo/`                      ‣ pkgs: 1058
        ./ooosssso++osssssso+`                     ‣ flat: 0
       .oossssso-````/ossssss+`        ├───────────────── Hardware Information ─────────────────┤
      -osssssso.      :ssssssso.                  󰍛 ‣ cpu: AMD Ryzen 7 7800X3D 8-Core Processor
     :osssssss/        osssso+++.                 󰓅 ‣ ram: 5.56 GB / 30.51 GB
    /ossssssss/        +ssssooo/-      ├───────────────── Uptime Information ───────────────────┤
  `/ossssso+/:-        -:/+osssso+-                ‣ uptime: 3h 02m
 `+sso+:-`                 `.-/+oso:               ‣ age: 20 days
`++:.                           `-/+/  └────────────────────────────────────────────────────────┘

`````

## Contributing

If you'd like to contribute to swiftfetch, feel free to fork the repo and submit a pull request. Contributions are always welcome!

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
