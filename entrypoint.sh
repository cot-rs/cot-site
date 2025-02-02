#!/bin/sh

set -e

cot-site collect-static /app/static
exec cot-site "$@"
