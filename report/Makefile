# Mileage may vary
TEX_PATH=pdflatex
TEX_PARAMS=-interaction=nonstopmode -file-line-error
TEX=$(TEX_PATH) $(TEX_PARAMS)

# expected tex file name, i.e. SRC_FOLDER/main.tex
PACKAGE_NAME=main

# preferred pdf viewer
BROWSER=firefox

SRC_FOLDER=./src
BUILD_FOLDER=./build

##############
# Begin actual Code, usually no need to edit it...

all: clean build

clean:
	rm -rf $(BUILD_FOLDER)

build: clean
	mkdir $(BUILD_FOLDER) && \
	cd $(BUILD_FOLDER) && \
	cp -r ../$(SRC_FOLDER)/* . && \
	$(TEX) $(PACKAGE_NAME).tex && \
	$(TEX) $(PACKAGE_NAME).tex && \
	cd .. && \
	cp $(BUILD_FOLDER)/$(PACKAGE_NAME).pdf .
	echo "🥳 PDF successfully built! 🥳"

show: build
	$(BROWSER) $(PACKAGE_NAME).pdf &

# lets not think about that
.phony: all clean build show
