# TODO(ryan): I've changed as/ld to native cross compiler ones.
# Could this cause dependency problems in the future?
AS=as -march=i386 --32
LD=ld -melf_i386 -nostdlib
QEMU=qemu-system-i386
TARGET=i686-unknown-linux-gnu
QEMUARGS=-device rtl8139,vlan=0 -net user,id=net0,vlan=0 -net dump,vlan=0,file=/tmp/rustos-dump.pcap -d int,cpu_reset

.PHONY: all
all: boot.bin

.PHONY: run
run: boot.bin
	$(QEMU) $(QEMUARGS) -kernel $<

.PHONY: debug
debug: boot.bin
	$(QEMU) $(QEMUARGS) -S -gdb tcp::3333 -kernel $< &
	termite -e 'gdb $< -ex "target remote :3333" -ex "break _start" -ex "c"'

.PHONY: vb
vb: boot.iso
	virtualbox --debug --startvm rustos


.PHONY: target/$(TARGET)/debug/librustos*.a
target/$(TARGET)/debug/librustos*.a: Cargo.toml
	cargo build --target $(TARGET) --verbose

boot.bin: src/arch/x86/link.ld \
		target/$(TARGET)/debug/deps/boot.o \
		target/$(TARGET)/debug/librustos*.a \
		target/$(TARGET)/debug/deps/context.o \
		target/$(TARGET)/debug/deps/_context.o
	$(LD) --gc-sections -o $@ -T $^

boot.iso: boot.bin
	cp boot.bin src/isodir/boot/
	grub-mkrescue -o boot.iso src/isodir

target/$(TARGET)/debug/deps/:
	mkdir -p $@

target/$(TARGET)/debug/deps/%.s: ../rust/src/rt/arch/i386/%.S target/$(TARGET)/debug/deps/
	$(CPP) -o $@ $<

target/$(TARGET)/debug/deps/%.o: src/arch/x86/%.s target/$(TARGET)/debug/deps/
	$(AS)  -o $@ $<

.PHONY: clean
clean:
	cargo clean
