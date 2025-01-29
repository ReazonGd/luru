# Variabel
BINARY_NAME := luru
INSTALL_DIR := /usr/local/bin
TARGET := release/$(BINARY_NAME)
# TARGET := target/release/$(BINARY_NAME)

# Default target
.PHONY: all
all: build

# Build proyek menggunakan cargo
.PHONY: build
build:
	cargo build --release
	sudo cp target/release/$(BINARY_NAME) release/


# Install biner ke /usr/local/bin
.PHONY: install
install: 
	@echo "Installing $(BINARY_NAME) to $(INSTALL_DIR)..."
	@sudo install -Dm755 $(TARGET) $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Installation complete!"

# Uninstall biner dari /usr/local/bin
.PHONY: uninstall
uninstall:
	@echo "Uninstalling $(BINARY_NAME) from $(INSTALL_DIR)..."
	@sudo rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Uninstallation complete!"

# Membersihkan file build
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
