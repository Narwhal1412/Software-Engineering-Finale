#!/usr/bin/env bash
set -e

echo "Running Go tests..."
cd "$(dirname "$0")/hello-service"
go test ./...
