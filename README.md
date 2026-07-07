# see

A *very* lightweight terminal image viewer for terminals that support real graphics protocols.

`see` renders images using terminal graphics protocols such as Kitty graphics, iTerm2 inline images, and SIXEL. It does not use ASCII, braille, or block-character rendering.

## Installation

```sh
cargo install see-cli --locked
```

The default install only includes PNG and JPEG support. Other image formats can be enabled via feature flags.

## Usage

```sh
see image.png
```

Fit to terminal width and allow vertical scrolling:

```sh
see image.png --full-width
```

Display at original image size:

```sh
see image.png --original
```

Resize by width, preserving aspect ratio:
```sh
see image.png --width 800
```

Resize by height, preserving aspect ratio:

```sh
see image.png --height 600
```

Fit inside a pixel box:

```sh
see image.png --width 800 --height 600
```

Displaying multiple images:

```sh
see *.png
```
or
```sh
see a.png b.jpg
```

## Supported Terminals

Support depends on the terminal’s graphics protocol support. The current goal is to support terminals implementing Kitty graphics, iTerm2 inline images, or SIXEL, and the protocol detection is done automatically. If your terminal doesn't seem to work but supports one of these protocols, just create an issue!

## Future Plans

None. This crate is done, I don't plan to add any other features other than support fixes.

## Alternatives

* [viu](https://github.com/atanunq/viu)
* [timg](https://github.com/hzeller/timg)
