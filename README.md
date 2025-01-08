# swiftfetch

**swiftfetch** is a fast and simple system information fetch tool written in Rust. It displays various system information like OS, CPU, RAM usage, uptime, and more in a neat, customizable format.

## Attention

⚠️ Breaking Change: The configuration format has been updated! The new version of swiftfetch will break compatibility with the old config.toml format. The previous configuration options (user_host_format, order, etc.) have been replaced with a new structure. Please update your configuration file by referring to the new format below.

## Features

- Displays Ascii art, essential system information like OS, kernel version, CPU, RAM usage, and more.
- Easy to configure and extend.
- Simple, fast, and lightweight.

## Installation

Using the AUR:

`yay -S swiftfetch` (or any other AUR helper)

To install swiftfetch, use the following command from the root of the project:

`cargo install --path .`

This command will install the program and automatically copy the default config.toml to ~/.config/swiftfetch/config.toml.

## Configuration

The configuration file can be found at `~/.config/swiftfetch/config.toml`, you can customize it to display or hide specific information based on your preferences.

If you did not set $EDITOR then it will return `nano` as default. Most DEs do that by default afaik, might have to add it yourself for WMs
You can find the default config right [here](/config/config.toml)

## Configuration Options

- `ascii_path`: This is currently the only way to display ascii art, I will add proper Ascii art for distros at some point.
- `ascii_color`: Sets the color of the ascii art.

- `items`: This section defines a list of key-value pairs for the items to be displayed. Each item can have three components:

  - **`key`**: This is the label for the information. For example, it could be "OS", "CPU", "RAM", or any other label you want to display for the corresponding value.
  - **`type`**: Defines the type of content to be displayed. You can choose from the following:
    - `default`: The value is dynamically fetched (e.g., OS name, kernel version).
    - `text`: A custom static value that you define.
    - `command`: Executes a shell command and displays the result.
  - **`value`**: This is the content associated with the key. The content can be a static text, a command to run, or a dynamic value, depending on the `type`.

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

```

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

```

## Contributing

If you'd like to contribute to swiftfetch, feel free to fork the repo and submit a pull request. Contributions are always welcome!

## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
