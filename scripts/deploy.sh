ssh nebuchadnezar 'docker stop perfbot'
docker save perfbot | bzip2 | ssh nebuchadnezar 'bunzip2 | docker load'
ssh nebuchadnezar 'docker run --rm -d -p 3333:3333 --mount type=bind,source=/home/perfbot,target=/app/data --name perfbot perfbot:latest'