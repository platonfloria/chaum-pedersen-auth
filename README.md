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
        "y1": 53417206097666059,
        "y2": 244824878090066088
    }' \
    localhost:50051 zkp_auth.Auth.Register
```

### Create authentication challenge
```bash
grpcurl -plaintext \
    --d '{
        "user": "testuser",
        "r1": 102394247258157955,
        "r2": 234539649658649999
    }' \
    localhost:50051 zkp_auth.Auth.CreateAuthenticationChallenge
```

### Verify authentication
```bash
grpcurl -plaintext \
    --d '{
        "auth_id": "bb7e1fc6-ea9d-4285-95b7-e8c69cfc5704",
        "s": 2486592
    }' \
    localhost:50051 zkp_auth.Auth.VerifyAuthentication
```
