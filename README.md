# CLAVIS
<p align="center">
<strong>CLAVIS BY</strong> <br/>
  <img src="frontend/public/img/katalyst_basic.png" alt="CLAVIS" width="70"/>
</p>

## Description

CLAVIS is a web application that allows users to change their passwords in Active Directory or Samba AD. Use StartTLS connection and allow self signed certificates (has disabled certificate validation).

## Background
I have a home lab with a Samba AD domain controller and I don't want to use terminal or windows tools to change manually the passwords of the users. So I decided to create a web application to change the passwords of the users.

## Features

- [x] Change password for users in Active Directory or Samba AD
- [x] Use StartTLS connection
- [x] Allow self signed certificates (has disabled certificate validation)

## Password Policy

- Minimum 8 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one number
- At least one special character

# Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| LDAP_URL | LDAP server URL | ldap://ldap.example.com |
| LDAP_BASE_DN | LDAP base DN | dc=example,dc=com |
| LDAP_SERVICE_USER | LDAP service user | cn=admin,dc=example,dc=com |
| LDAP_SERVICE_PASSWORD | LDAP service password | admin |
| RUST_LOG | Log level | info |
| BIND_ADDRESS | Bind address | 0.0.0.0:40000 |
| SMTP_HOST | SMTP server host | smtp.example.com |
| SMTP_PORT | SMTP server port | 587 |
| SMTP_USERNAME | SMTP username | username |
| SMTP_PASSWORD | SMTP password | password |
| SMTP_FROM | SMTP from | [EMAIL_ADDRESS] |

## Dependencies
It need `openssl` and `ca-certificates` for email transport other wise when try to send a message will fail.
To disable the SMPT don't use the SMTP environment vars and it will disable automatically.

## Build
For local build we have the makefile with some utils commands for building and run:

```bash
# build the service
make build
# build and run the service
make run
```

## Docker
There is a docker version with Dockerfile, it install all the dependencies and build the service inside the container and you can run it with docker-compose:

```bash
services:
  clavis:
    build: .
    ports:
      - "3000:3000"
    environment:
      - LDAP_URL=ldap://ip_or_hostname:389
      - LDAP_BASE_DN=DC=exampl,DC=local
      - LDAP_SERVICE_USER=CN=Administrator,CN=Users,DC=example,DC=local
      - LDAP_SERVICE_PASSWORD=password
      - RUST_LOG=info
      - BIND_ADDRESS=0.0.0.0:3000
      - SMTP_HOST=smtp
      - SMTP_PORT=587
      - SMTP_USERNAME=username@smtp.com
      - SMTP_PASSWORD=user_password
      - SMTP_FROM=from@email.com
    restart: always
```
### Internally
We have a image on nexus in `katalyst-docker/clavis` and we can pull it or use in ocker compose with:
```yaml
image: nexus.katalyst.con/katalyst-docker/clavis:latest

```
