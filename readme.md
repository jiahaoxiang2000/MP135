# MP135

this project is cross compile project for arm. the MP135 is the chip name stm32MP135.

## environment

because the board had the linux operation system, so we can use the cross compile tool to compile the code.

### cross compile tool

our env is on the MacOS 15.1
    
```shell
brew tap messense/macos-cross-toolchains
brew install arm-unknown-linux-gnueabihf
echo 'export PATH="/opt/homebrew/opt/arm-unknown-linux-gnueabihf/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
brew install arm-linux-gnueabihf-binutils

```