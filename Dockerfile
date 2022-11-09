################################################
# Example Dockerfile for a common Node.js file #
################################################

# You can change this file if you need but read this first
# https://github.com/matilda-applicants/common-tasks-instructions/wiki/Docker-on-your-task

# =======================================================================
# BUILD STAGE
# =======================================================================
#
FROM rust AS builder 
ARG APP_NAME=type-ahead-api
# Update the packages used to build so that they are the latest ones which
# will be incorporated in the exe

# RUN apk upgrade --update-cache && \
#     apk add --no-cache musl-dev build-base cmake && \
#     rm -rf /var/cache/apk/*

WORKDIR /usr/src/${APP_NAME}
# RUN rustup target add x86_64-unknown-linux-musl
COPY ./src ./src
COPY ./Cargo.toml .
COPY ./Cargo.lock .
COPY ./log4rs.yml .
COPY ./names.json .
RUN ls -l
# RUN USER=root cargo new ${APP_NAME}

# COPY Cargo.toml Cargo.lock ./
# COPY log4rs.yml ./ 
# COPY names.json ./ 
# WORKDIR /usr/src/${APP_NAME}

# Build the application with cross platform support so it can be run on
# a different version of linux

# COPY ./src ./src

# RUN cargo install --target x86_64-unknown-linux-musl --path .
RUN cargo install --path .

# RUN cargo build --release
# RUN rm -rf /build/src
# RUN rm -rf /build/target/release/.fingerprint/${APP_NAME}-*

# Change the name of the application so that the remaininder of the 
# Dockerfile is generic 
RUN mv /usr/local/cargo/bin/${APP_NAME} /app-exe

# Move the log4rs configuration file to a generic place so the rest of the Dockerfile
# does not need to know what application it is
# RUN mv /usr/src/${APPLICATION}/log4rs.yml /log4rs.yml

# =======================================================================
# FINAL STAGE - Copy only the application into a minimal container
# The image size was 17.7 MB 
# =======================================================================
#
FROM debian:buster-slim
ARG APP_NAME=type-ahead-api
# Update all packages in Alpine to the latest version. Need to do this to ensure
# all security patches etc are up to date
# RUN apk upgrade --update-cache && \
#     rm -rf /var/cache/apk/*

RUN apt-get update && apt-get install -y && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app-exe .
COPY --from=builder /usr/src/${APP_NAME}/log4rs.yml .
COPY --from=builder /usr/src/${APP_NAME}/names.json .
RUN ls  -F

# Create the app-user and set the privs for the dir and its contents
RUN addgroup --gid 1000 app-user
RUN adduser --disabled-password --shell /bin/sh --uid 1000 --ingroup app-user app-user
RUN chown -R app-user:app-user /app

# This will export the PORT environment variable to your application.
# It has 12345 as default value, but when running the container we might pass
# any other port. You shouldn't change this unless you really know what you are doing.
ENV PORT 12345

# This will export the SUGGESTION_NUMBER environment variable to your application.
# It has 10 as default value, but when running the container we might pass
# any other value
ENV SUGGESTION_NUMBER 10

ENV HOST 0.0.0.0

# Avoid changing this too; it will expose the port so
# other containers can connect to your app.
EXPOSE $PORT

# Change to the app-user to run the application this stops running the app 
# with root privs
# USER app-user

CMD ["./app-exe"]



