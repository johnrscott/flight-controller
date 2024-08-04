# Install the STM32 programmer cli tools and st-link before running this script
arm-none-eabi-objcopy --output-target binary target/thumbv7em-none-eabi/debug/firmware firmware.bin
st-flash write firmware.bin 0x08000000
