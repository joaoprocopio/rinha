# rinha

## dev setup

```sh
docker compose -f docker/compose.processor.yml up -d
docker compose up -d
```

## devenv teardown

```sh
docker compose -f docker/compose.processor.yml down
docker compose down
docker volume prune --all --force
```

## build & push

```sh
export TIMESTAMP=$(date '+%Y%m%d%H%M%S')
export ARCH="amd64"
export PLATFORM="linux/$ARCH"
export IMAGE="joaoprocopio/rinha:$ARCH-$TIMESTAMP"

docker login
docker build --platform $PLATFORM --tag $IMAGE .
docker push $IMAGE
```
