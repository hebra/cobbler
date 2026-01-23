.PHONY: all build run test tidy fmt clean

BINARY := cobbler

all: build

build:
	go build -o bin/$(BINARY) ./cmd/$(BINARY)

run:
	go run ./cmd/$(BINARY)

test:
	go test ./...

tidy:
	go mod tidy

fmt:
	go fmt ./...

clean:
	rm -rf bin
