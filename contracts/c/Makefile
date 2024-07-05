TARGET := riscv64-unknown-linux-gnu
CC := $(TARGET)-gcc
LD := $(TARGET)-gcc
OBJCOPY := $(TARGET)-objcopy
CFLAGS := -fPIC -nostdinc -nostdlib -nostartfiles -fvisibility=hidden -I deps/ckb-c-stdlib/libc -I deps/ckb-c-stdlib/molecule -I deps/ckb-c-stdlib -I ../libs/types/c/src -Wall -Werror -Wno-nonnull -Wno-nonnull-compare -Wno-unused-function -g
LDFLAGS := -Wl,-static -fdata-sections -ffunction-sections -Wl,--gc-sections
DEBUG_FLAGS = -DCKB_C_STDLIB_PRINTF

NET_TYPE ?= mainnet
ifeq ($(NET_TYPE), testnet)
	CFLAGS += -DCKB_TESTNET
else ifeq ($(NET_TYPE), mainnet)
	CFLAGS += -DCKB_MAINNET
else
	$(error "Unknown NET_TYPE: $(NET_TYPE)")
endif
#BUILDER_DOCKER := dotbitteam/ckb-dev-all-in-one:0.0.1
BUILDER_DOCKER := nervos/ckb-riscv-gnu-toolchain:gnu-focal-20230214

CODE_DIR_DOCKER := /code
OUTPUT_DIR := ./build
define DOCKER_RUN_CMD
	docker run --rm \
	-v `pwd`:/code \
	-v ./build:/build \
	-v ~/.gitconfig:/root/.gitconfig:ro \
	-v ~/.cargo:/root/.cargo \
	-v ./../../libs:/libs \
	${BUILDER_DOCKER} bash -c
endef

via-docker:
	@${DOCKER_RUN_CMD} "cd ${CODE_DIR_DOCKER} && make $(ARGS) CFLAGS='$(CFLAGS)'"

build/debug/%.so: %.c
	$(CC) $(CFLAGS) $(LDFLAGS) $(DEBUG_FLAGS) -shared -o $@ $<

build/release/%.so: %.c
	$(CC) $(CFLAGS) $(LDFLAGS) -shared -o $@ $<
	$(OBJCOPY) --strip-debug --strip-all $@ $@

clean:
	rm -rf $(OUTPUT_DIR)/*.so $(OUTPUT_DIR)/*.so.debug *.so *.so.debug always_success
