set -xe
cargo build --target=./target.json
sudo qemu-system-arm -M vexpress-a15 -cpu cortex-a15 -monitor /dev/null -kernel ./target/target/debug/osdev.elf -netdev tap,id=net0,ifname=tap0,script=no,downscript=no,vnet_hdr=off -net nic,model=lan9118,netdev=net0 -nographic -m 1G

