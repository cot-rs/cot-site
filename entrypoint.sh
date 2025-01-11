#!/bin/sh

set -e

cot-site collect-static /app/static
cot-site "$@"
