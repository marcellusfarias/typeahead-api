# =======================================================================
# BUILD STAGE
# =======================================================================

FROM rust:latest AS builder 
ARG APP_NAME=type-ahead-api

WORKDIR /usr/src/${APP_NAME}

COPY ./src ./src
COPY ./Cargo.toml .
COPY ./Cargo.lock .
COPY ./log4rs.yml .
COPY ./names.json .

# debug information. Run with --progress=plain to see the result of this command.
# RUN ls -l 

RUN cargo install --path .

# Change the name of the application so that the remaininder of the 
# Dockerfile is generic 
RUN mv /usr/local/cargo/bin/${APP_NAME} /app-exe

# =======================================================================
# FINAL STAGE - Copy only the application into a minimal container
# The image size was 93 MB 
# =======================================================================
# Using debian since the official Rust example is doing the same. There are some pros and cons about using Alpine.
FROM debian:buster-slim
ARG APP_NAME=type-ahead-api

# Update all packages in Debian to the latest version. Need to do this to ensure
# all security patches etc are up to date
RUN apt-get update && apt-get install -y && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app-exe .
COPY --from=builder /usr/src/${APP_NAME}/log4rs.yml .
COPY --from=builder /usr/src/${APP_NAME}/names.json .

# debug information. Run with --progress=plain to see the result of this command.
#RUN ls  -F 

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

ENV FILE_NAME ./names.json

# Avoid changing this too; it will expose the port so
# other containers can connect to your app.
EXPOSE $PORT

# Change to the app-user to run the application this stops running the app 
# with root privs
USER app-user

CMD ["./app-exe"]



