docker run \
--rm \
-ti \
-p 3333:3333 \
--mount type=bind,source=/home/yatekii/repos/probe-rs/perf/data,target=/app/data \
perfbot:latest