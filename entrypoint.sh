#!/bin/bash
/app/server &
/app/worker &

wait -n

exit $?
