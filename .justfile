user    := "atareao"
name    := `basename ${PWD}`
version := `git tag -l  | tail -n1`

build:
    echo {{version}}
    echo {{name}}
    podman build -t {{user}}/{{name}}:{{version}} \
                 -t {{user}}/{{name}}:latest \
                 .

tag:
    podman tag {{user}}/{{name}}:{{version}} {{user}}/{{name}}:latest

push:
    podman push {{user}}/{{name}} --all-tags

run:
    podman run --rm \
               --init \
               --name aopodcast \
               --volume $PWD/config.yml:/app/config.yml \
               --volume $PWD/data:/app/data \
               --volume $PWD/assets:/app/assets \
               --volume $PWD/templates:/app/templates \
               --volume $PWD/posts:/app/posts \
               --volume $PWD/pages:/app/pages \
               {{user}}/{{name}}

test:
    echo {{version}}
    echo {{name}}
    podman build -t {{user}}/{{name}}:test \
                 .
    podman push {{user}}/{{name}}:test

