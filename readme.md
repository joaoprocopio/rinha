# rinha

## dev local setup

```sh
cargo run
```

## dev docker setup

```sh
docker compose -f compose.processor.yml up -d
docker compose up -d
```

## dev docker teardown

```sh
docker compose -f compose.processor.yaml down
docker compose down
docker volume prune --all --force
```

## build & push

```sh
docker login

export TIMESTAMP=$(date '+%Y%m%d%H%M%S')
export ARCH="amd64"
export PLATFORM="linux/$ARCH"
export IMAGE="joaoprocopio/rinha:$ARCH-$TIMESTAMP"

docker build --platform $PLATFORM --tag $IMAGE .
docker push $IMAGE
```
