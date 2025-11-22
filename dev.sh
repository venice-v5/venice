arm-none-eabi-objcopy -O binary $(zoxide query venice/target)/armv7a-vex-v5/debug/venice $(zoxide query org.venice.venice-cli)/venice-v0.1.0.bin

# cargo v5 rm user/venice-v0.1.0.bin

$(zoxide query venice-cli)/target/debug/venice_cli run
