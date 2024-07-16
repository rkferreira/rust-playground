#.SILENT

build:
	@echo "#######################"
	@echo "Building and testing..."
	@echo "#######################"
	@echo ""
	@cargo test
	@cargo build --release
	@echo ""
	@echo "Done."
	@echo ""


install:
	@echo "#######################"
	@echo "Installing..."
	@echo "#######################"
	@echo ""
	sudo install target/release/platform-cli /usr/local/bin
	@echo ""
	@echo "Done."
	@echo ""

checkdeps:
	@echo "#######################"
	@echo "Checking dependencies..."
	@echo "#######################"
	@echo ""
	@echo "Check if terraform is present"
	@echo ""
	terraform --version
	@echo ""
	@echo "Done"
	@echo ""

all: build install checkdeps
