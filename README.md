# mozconfig

Terminal configuration switcher for Mozilla build trees.

## General

This tool assumes that you have a list of available configurations in the root of your mozilla build directory, e.g.:

```
~/code/mozilla/.mozconfig-debug
~/code/mozilla/.mozconfig-release
~/code/mozilla/.mozconfig-js
```

Using `mozconfig` will symlink a specified configuration to `~/code/mozilla/.mozconfig` which is the default used by `mach`.

## Usage

```bash
# Switch to a configuration
mozconfig release

# Show the current configuration
mozconfig

# Show a list of available configurations
mozconfig --list
```

## oh-my-zsh plugin

This repo contains a plugin for [oh-my-zsh](https://ohmyz.sh/) which will can show the currently set config in the prompt.

Symlink the `oh-my-zsh/mozconfig` to your plugins directory.

```bash
ln -s $(pwd)/oh-my-zsh/mozconfig ~/.oh-my-zsh/plugins/mozconfig
```

Then add the following to your `.zshrc` file:

```bash
PROMPT+='$(prompt_mozconfig)'
```
