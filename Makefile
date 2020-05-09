# Simple, stupid makefile to make phd

TARGET = phd
RELEASE = target/release/$(TARGET)
DEBUG = target/debug/$(TARGET)
SOURCES = $(wildcard src/*.rs src/**/*.rs)
PREFIX = $(DESTDIR)/usr/local
BINDIR = $(PREFIX)/bin

.PHONY: release debug install uninstall clean

# Default target. Build release binary.
release: $(RELEASE)

# Binary with debugging info.
debug: $(DEBUG)

# Install locally.
install: $(RELEASE)
	install $(RELEASE) $(BINDIR)/$(TARGET)

# Uninstall locally.
uninstall: $(RELEASE)
	-rm $(BINDIR)/$(TARGET)

# Remove build directory.
clean:
	-rm -rf target

# Build the release version
$(RELEASE): $(SOURCES)
	cargo build --release

# Build the debug version
$(DEBUG): $(SOURCES)
	cargo build

# Build manual
.PHONY: manual
manual: doc/phd.1

doc/phd.1: doc/phd.1.md scdoc
	scdoc < doc/phd.1.md > doc/phd.1

# Must have scdoc installed to build manual.
scdoc:
	@which scdoc || (echo "scdoc(1) not found."; \
		echo "please install to build the manual: https://repology.org/project/scdoc"; exit 1)
