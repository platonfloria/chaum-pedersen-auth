# Chaum Pedersen authentication

## Build image

```bash
make build
```
or
```bash
make build service=tests
```

## Test image

```bash
make test
```

## Run command inside the container

```bash
make terminal
```

## Run stack

```bash
make run
```
or
```bash
make run_local
```

## Smoke testing
Install grpcurl to hit api endpoints from the command line.
https://github.com/fullstorydev/grpcurl#installation

### Register
```bash
grpcurl -plaintext \
    --d '{
        "user": "testuser",
        "y1": 0,
        "y2": 0
    }' \
    localhost:50051 zkp_auth.Auth.Register
```

### Create authentication challenge
```bash
grpcurl -plaintext \
    --d '{
        "user": "testuser",
        "r1": 0,
        "r2": 0
    }' \
    localhost:50051 zkp_auth.Auth.CreateAuthenticationChallenge
```

### Verify authentication
```bash
grpcurl -plaintext \
    --d '{
        "auth_id": "",
        "s": 0
    }' \
    localhost:50051 zkp_auth.Auth.VerifyAuthentication
```
