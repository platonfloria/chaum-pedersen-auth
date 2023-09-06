# Chaum Pedersen authentication
This project implements a simple server/client authentication using Chaum Pedersen zero knowledge sigma protocol.
There are two variants of this protocol implemented, one is using exponentiation, the other one is using k256 elliptic curve.

When run in docker-compose or locally, by default, the grcp server is exposed on http://localhost:50051 and web client is exposed on http://localhost:8080.


## Build images
```bash
make build
```

## Run unit tests
```bash
make test
```

## Run stack in docker compose
```bash
make run
```

## Run server locally
```bash
make run_local_server
```

## Run client locally
```bash
make run_local_client
```

## Smoke testing
Install grpcurl to hit api endpoints from the command line.
https://github.com/fullstorydev/grpcurl#installation

### Register
```bash
grpcurl -plaintext \
    --d '{
        "user": "testuser",
        "y1": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAL3GqnpYYAs=",
        "y2": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA2XK7fK8mKg="
    }' \
    localhost:50051 zkp_auth.Auth.Register
```

### Create authentication challenge
```bash
grpcurl -plaintext \
    --d '{
        "user": "testuser",
        "r1": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWvHBn7ds4M=",
        "r2": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA0FAkZel/Y8="
    }' \
    localhost:50051 zkp_auth.Auth.CreateAuthenticationChallenge
```

### Verify authentication
```bash
grpcurl -plaintext \
    --d '{
        "auth_id": "11fc1350-1288-4bfd-9322-f0e9d491cd77",
        "s": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAl8UA="
    }' \
    localhost:50051 zkp_auth.Auth.VerifyAuthentication
```
