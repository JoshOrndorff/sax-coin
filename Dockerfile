# Simple dockerfile to make the node available to meetup participants

# Choose the base image
FROM ubuntu:20.04

# Copy the node into the image
COPY target/release/sax-coin .

# Open some ports
EXPOSE 30333 9933 9944

# Specifying an ENTRYPOINT rather than a CMD allows me to pass args to the node
# https://stackoverflow.com/a/29661891/4184410
ENTRYPOINT ["./sax-coin"]
